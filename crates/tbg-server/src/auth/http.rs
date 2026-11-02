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
    #[serde(default)]
    email: String,
    locale: String,
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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RecoveryRequest {
    name: String,
    email: String,
    locale: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Recover {
    name: String,
    code: String,
    device_id: String,
    persistent: bool,
    registration_retry_key: String,
}

enum Output {
    Accepted,
    Grant(Grant),
    Recovered(Grant),
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

/// P66: each release slot keeps its own Access index, so a request naming
/// another slot was misrouted. Single-server mode and unnamed requests pass.
fn slot_matches(headers: &HeaderMap, mine: Option<&str>) -> Result<(), Error> {
    let mut named = headers.get_all("x-tbg-slot").iter();
    let Some(first) = named.next() else {
        return Ok(());
    };
    if named.next().is_some() {
        return Err(Error::BadRequest);
    }
    match mine {
        Some(slot) if first.as_bytes() != slot.as_bytes() => Err(Error::Unavailable),
        _ => Ok(()),
    }
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
    slot_matches(&headers, app.release.slot())?;
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
        "recovery-request" => {
            let i: RecoveryRequest = parse(body)?;
            auth.recovery_request(i.name, i.email, i.locale, ip).await;
            Ok(Output::Accepted)
        }
        "recover" => {
            let i: Recover = parse(body)?;
            Ok(Output::Recovered(
                auth.recover(
                    i.name,
                    i.code,
                    &i.device_id,
                    i.persistent,
                    &i.registration_retry_key,
                    channel,
                    ip,
                )
                .await?,
            ))
        }
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
                    input.email,
                    input.locale,
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
    let recovered = matches!(&output, Output::Recovered(_));
    match output {
        Output::Accepted => Ok((StatusCode::ACCEPTED, Json(json!({"ok":true}))).into_response()),
        Output::Grant(grant) | Output::Recovered(grant) => {
            if now >= grant.expires_at || now >= grant.family.expires_at {
                return Err(Error::Unauthorized);
            }
            let mut value = json!({"access_token":grant.access,"access_expires_at":grant.expires_at,"session_expires_at":grant.family.expires_at,"session_id":tokens::hex(&grant.family.id),"server_time":now,"profile":grant.profile,"persistent":grant.persistent});
            if let Some(key) = grant.account_key {
                value["account_key"] = Value::String(key);
                value["email_state"] =
                    Value::String(if recovered { "verified" } else { "unverified" }.into());
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
    if let Error::Suspended { until, reason } = error {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error":"account_suspended","until":until,"reason":reason.as_str()})),
        )
            .into_response();
    }
    let (status, code) = match error {
        Error::Suspended { .. } => (StatusCode::FORBIDDEN, "account_suspended"),
        Error::CodeInvalid => (StatusCode::UNAUTHORIZED, "code_invalid"),
        Error::RegistrationClosed => (StatusCode::SERVICE_UNAVAILABLE, "registration_closed"),
        Error::EmailLimit => (StatusCode::CONFLICT, "email_limit"),
        Error::EmailBlocked => (StatusCode::FORBIDDEN, "email_blocked"),
        Error::EmailInvalid => (StatusCode::BAD_REQUEST, "email_invalid"),
        Error::EmailRequired => (StatusCode::BAD_REQUEST, "email_required"),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn p66_auth_requests_are_served_only_by_the_named_generation() {
        let mut headers = HeaderMap::new();
        assert_eq!(slot_matches(&headers, Some("a")), Ok(()), "unnamed request");
        headers.insert("x-tbg-slot", "b".parse().unwrap());
        assert_eq!(slot_matches(&headers, None), Ok(()), "single-server mode");
        assert_eq!(slot_matches(&headers, Some("b")), Ok(()));
        assert_eq!(slot_matches(&headers, Some("a")), Err(Error::Unavailable));
        headers.append("x-tbg-slot", "b".parse().unwrap());
        assert_eq!(slot_matches(&headers, Some("b")), Err(Error::BadRequest));
    }
}

/// P73 owner HTTP uses the same Access index, channel/origin policy and slot fence.
pub(crate) fn history_binding(
    app: &AppState,
    headers: &HeaderMap,
) -> Result<super::Binding, Error> {
    let channel = if headers.get("x-tbg-client").is_some_and(|v| v == "native") {
        Channel::Native
    } else {
        Channel::Browser
    };
    origin::validate(channel, headers, &app.config.auth_origins)?;
    slot_matches(headers, app.release.slot())?;
    let raw = bearer(headers)?.ok_or(Error::Unauthorized)?;
    app.auth.authenticate(tokens::decode::<32>(&raw)?)
}

/// CE-01 changes only the permitted media type; the exact Origin, custom header, cookie and slot fences stay identical.
pub(crate) fn image_binding(app: &AppState, headers: &HeaderMap) -> Result<super::Binding, Error> {
    if headers.get_all("content-type").iter().count() != 1
        || !headers
            .get("content-type")
            .is_some_and(|v| matches!(v.to_str(), Ok("image/png" | "image/gif" | "image/webp")))
    {
        return Err(Error::BadRequest);
    }
    let mut envelope = headers.clone();
    envelope.insert(
        "content-type",
        axum::http::HeaderValue::from_static("application/json"),
    );
    history_binding(app, &envelope)
}
