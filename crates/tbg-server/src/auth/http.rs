//! Separate native/browser POST routes; cookie credentials never enter browser JSON.

use super::*;
use crate::AppState;
use axum::{
    extract::{ConnectInfo, Path, Request, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::net::SocketAddr;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Login {
    account_key: String,
    device_id: String,
    persistent: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Register {
    name: String,
    device_id: String,
    persistent: bool,
    registration_retry_key: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BrowserRefresh {
    operation_id: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeRefresh {
    operation_id: String,
    refresh_token: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Empty {}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeCredential {
    refresh_token: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Recovery {
    account_key: String,
}

enum Output {
    Grant(Grant),
    Confirmed(FamilyId),
    LoggedOut { clear_cookie: bool },
}

fn parse<T: serde::de::DeserializeOwned>(body: &[u8]) -> Result<T, Error> {
    serde_json::from_slice(body).map_err(|_| Error::BadRequest)
}

fn bearer(headers: &HeaderMap) -> Result<Option<String>, Error> {
    if headers.get_all("authorization").iter().count() > 1 {
        return Err(Error::Unauthorized);
    }
    let Some(header) = headers.get("authorization") else {
        return Ok(None);
    };
    let header = header.to_str().map_err(|_| Error::Unauthorized)?;
    let value = header.strip_prefix("Bearer ").ok_or(Error::Unauthorized)?;
    tokens::decode::<32>(value)?;
    Ok(Some(value.to_owned()))
}

pub(crate) async fn handler(
    State(app): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    Path((channel, operation)): Path<(String, String)>,
    request: Request,
) -> Response {
    let result = prepare(app, peer, channel, operation, request).await;
    let mut response = match result {
        Ok(response) => response,
        Err(error) => failure(error),
    };
    response.headers_mut().insert(
        "cache-control",
        axum::http::HeaderValue::from_static("no-store"),
    );
    response
        .headers_mut()
        .insert("pragma", axum::http::HeaderValue::from_static("no-cache"));
    response.headers_mut().insert(
        "x-content-type-options",
        axum::http::HeaderValue::from_static("nosniff"),
    );
    response
}

async fn prepare(
    app: AppState,
    peer: SocketAddr,
    channel: String,
    operation: String,
    request: Request,
) -> Result<Response, Error> {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    let (parts, body) = request.into_parts();
    let headers = parts.headers;
    let channel = match channel.as_str() {
        "browser" => Channel::Browser,
        "native" => Channel::Native,
        _ => return Err(Error::Forbidden),
    };
    origin::validate(channel, &headers, &app.config.auth_origins)?;
    let cookie = if channel == Channel::Browser {
        origin::cookie(&headers)?
    } else {
        None
    };
    let access = bearer(&headers)?;
    let ip = crate::ws::client_ip(peer, &headers);
    let permit = app
        .auth
        .work
        .clone()
        .try_acquire_owned()
        .map_err(|_| Error::Unavailable)?;
    let body = tokio::time::timeout_at(deadline, axum::body::to_bytes(body, 4096))
        .await
        .map_err(|_| Error::Unavailable)?
        .map_err(|_| Error::BadRequest)?;
    let auth = app.auth.clone();
    // Dropping the HTTP waiter does not cancel a committed rotation/registration.
    let task = tokio::spawn(async move {
        let _permit = permit;
        execute(
            &auth,
            &operation,
            channel,
            &body,
            cookie.as_deref(),
            access.as_deref(),
            ip,
        )
        .await
    });
    let output = tokio::time::timeout_at(deadline, task)
        .await
        .map_err(|_| Error::Unavailable)?
        .map_err(|_| Error::Unavailable)??;
    render(output, channel, app.auth.now()?)
}

async fn execute(
    auth: &Service,
    operation: &str,
    channel: Channel,
    body: &[u8],
    cookie: Option<&str>,
    bearer: Option<&str>,
    ip: IpAddr,
) -> Result<Output, Error> {
    match operation {
        "login" => {
            let input: Login = parse(body)?;
            Ok(Output::Grant(
                auth.login(
                    &input.account_key,
                    &input.device_id,
                    input.persistent,
                    channel,
                    ip,
                )
                .await?,
            ))
        }
        "register" => {
            let input: Register = parse(body)?;
            Ok(Output::Grant(
                auth.register(
                    input.name,
                    &input.device_id,
                    input.persistent,
                    &input.registration_retry_key,
                    channel,
                    ip,
                )
                .await?,
            ))
        }
        "refresh" => {
            let (operation, refresh) = match channel {
                Channel::Browser => {
                    let input: BrowserRefresh = parse(body)?;
                    (
                        input.operation_id,
                        cookie.ok_or(Error::Unauthorized)?.to_owned(),
                    )
                }
                Channel::Native => {
                    let input: NativeRefresh = parse(body)?;
                    (input.operation_id, input.refresh_token)
                }
            };
            Ok(Output::Grant(
                auth.refresh(&refresh, &operation, channel, ip).await?,
            ))
        }
        "confirm" => {
            let refresh = credential(body, channel, cookie)?.ok_or(Error::Unauthorized)?;
            Ok(Output::Confirmed(
                auth.confirm(&refresh, channel, ip).await?,
            ))
        }
        "logout" => {
            let refresh = credential(body, channel, cookie)?;
            auth.logout(channel, refresh.as_deref(), bearer, ip).await?;
            Ok(Output::LoggedOut { clear_cookie: true })
        }
        "logout-all" => {
            let input: Recovery = parse(body)?;
            let clear_cookie = auth
                .logout_all(&input.account_key, bearer, cookie, ip)
                .await?;
            Ok(Output::LoggedOut { clear_cookie })
        }
        _ => Err(Error::BadRequest),
    }
}

fn credential(
    body: &[u8],
    channel: Channel,
    cookie: Option<&str>,
) -> Result<Option<String>, Error> {
    match channel {
        Channel::Browser => {
            let _: Empty = parse(body)?;
            Ok(cookie.map(str::to_owned))
        }
        Channel::Native => {
            let input: NativeCredential = parse(body)?;
            Ok(input.refresh_token)
        }
    }
}

fn render(output: Output, channel: Channel, now: i64) -> Result<Response, Error> {
    match output {
        Output::Grant(grant) => {
            if now >= grant.expires_at || now >= grant.family.expires_at {
                return Err(Error::Unauthorized);
            }
            let mut value = json!({"access_token":grant.access,"access_expires_at":grant.expires_at,"session_expires_at":grant.family.expires_at,"session_id":tokens::hex(&grant.family.id),"server_time":now,"profile":grant.profile,"persistent":grant.persistent});
            if let Some(key) = grant.account_key {
                value["account_key"] = Value::String(key);
            }
            if channel == Channel::Native {
                value["refresh_token"] = Value::String(grant.refresh.clone());
            }
            let mut response = Json(value).into_response();
            if channel == Channel::Browser {
                let age = if grant.persistent {
                    format!("; Max-Age={}", grant.family.expires_at - now)
                } else {
                    String::new()
                };
                set_cookie(
                    &mut response,
                    format!(
                        "__Host-tbg_refresh={}; Path=/; Secure; HttpOnly; SameSite=Strict{age}",
                        grant.refresh
                    ),
                )?;
            }
            Ok(response)
        }
        Output::Confirmed(id) => Ok(Json(json!({"session_id":tokens::hex(&id)})).into_response()),
        Output::LoggedOut { clear_cookie } => {
            let mut response = Json(json!({"ok":true})).into_response();
            if channel == Channel::Browser && clear_cookie {
                set_cookie(
                    &mut response,
                    "__Host-tbg_refresh=; Path=/; Secure; HttpOnly; SameSite=Strict; Max-Age=0"
                        .into(),
                )?;
            }
            Ok(response)
        }
    }
}

fn set_cookie(response: &mut Response, cookie: String) -> Result<(), Error> {
    response.headers_mut().insert(
        "set-cookie",
        cookie.parse().map_err(|_| Error::Unavailable)?,
    );
    Ok(())
}

fn failure(error: Error) -> Response {
    let (status, code) = match error {
        Error::Unauthorized => (StatusCode::UNAUTHORIZED, "auth_required"),
        Error::Forbidden => (StatusCode::FORBIDDEN, "forbidden"),
        Error::BadRequest => (StatusCode::BAD_REQUEST, "bad_request"),
        Error::Unavailable => (StatusCode::SERVICE_UNAVAILABLE, "unavailable"),
        Error::RateLimited => (StatusCode::TOO_MANY_REQUESTS, "rate_limited"),
        Error::Conflict => (StatusCode::CONFLICT, "conflict"),
        Error::RecoveryRequired => (StatusCode::CONFLICT, "recovery_required"),
        Error::NameInvalid => (StatusCode::BAD_REQUEST, "name_invalid"),
        Error::NameTaken => (StatusCode::CONFLICT, "name_taken"),
        Error::FamilyLimit => (StatusCode::CONFLICT, "family_limit"),
        Error::Capacity => (StatusCode::SERVICE_UNAVAILABLE, "capacity"),
        Error::InMatch => (StatusCode::CONFLICT, "in_match"),
    };
    let mut response = (status, Json(json!({"error":code}))).into_response();
    if status == StatusCode::TOO_MANY_REQUESTS {
        response
            .headers_mut()
            .insert("retry-after", axum::http::HeaderValue::from_static("60"));
    }
    response
}
