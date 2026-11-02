//! Bounded authenticated email operations using the existing Access/origin/slot policy.
use crate::{
    store::recovery::{service, Error},
    AppState,
};
use axum::{
    extract::{ConnectInfo, Request, State},
    http::{Method, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use serde_json::json;
use std::{net::SocketAddr, time::Duration};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Code {
    code: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Change {
    account_key: String,
    email: String,
    locale: String,
    #[serde(default)]
    lost_old: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Confirm {
    #[serde(default)]
    old_code: String,
    new_code: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Cancel {
    old_code: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Empty {}
fn parse<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, Error> {
    serde_json::from_slice(bytes).map_err(|_| Error::EmailInvalid)
}
fn response(error: Error) -> Response {
    let (status, code) = match error {
        Error::Denied => (401, "code_invalid"),
        Error::EmailInvalid => (400, "email_invalid"),
        Error::EmailBlocked => (403, "email_blocked"),
        Error::EmailLimit => (409, "email_limit"),
        Error::RateLimited => (429, "rate_limited"),
        Error::Conflict => (409, "conflict"),
        _ => (503, "unavailable"),
    };
    let mut r = (
        StatusCode::from_u16(status).unwrap_or(StatusCode::SERVICE_UNAVAILABLE),
        Json(json!({"error":code})),
    )
        .into_response();
    if status == 429 {
        r.headers_mut()
            .insert("retry-after", "3600".parse().unwrap());
    }
    r
}
pub(crate) async fn handler(
    State(app): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    req: Request,
) -> Response {
    let mut result =
        match tokio::time::timeout(Duration::from_secs(5), execute(app, peer, req)).await {
            Ok(Ok(r)) => r,
            Ok(Err(e)) => response(e),
            Err(_) => response(Error::Unavailable),
        };
    result
        .headers_mut()
        .insert("cache-control", "no-store".parse().unwrap());
    result
}
async fn execute(app: AppState, peer: SocketAddr, req: Request) -> Result<Response, Error> {
    let binding =
        crate::auth::http::history_binding(&app, req.headers()).map_err(|_| Error::Denied)?;
    let _permit = app
        .history_reads
        .clone()
        .try_acquire_owned()
        .map_err(|_| Error::Unavailable)?;
    let ip = crate::ws::client_ip(peer, req.headers()).to_string();
    let path = req.uri().path().to_owned();
    let get = req.method() == Method::GET;
    let bytes = axum::body::to_bytes(req.into_body(), 4096)
        .await
        .map_err(|_| Error::EmailInvalid)?;
    let store = app.store.as_ref().ok_or(Error::Unavailable)?;
    let account = binding.account;
    let now = app.auth.now().map_err(|_| Error::Unavailable)?;
    store
        .release(move |db| -> Result<Response, Error> {
            match path.as_str() {
                "/api/me/account" if get => Ok(Json(service::panel(db, account)?).into_response()),
                "/api/me/email/verify" => {
                    let c: Code = parse(&bytes)?;
                    service::verify(db, account, &c.code, now)?;
                    Ok(Json(json!({"email_state":"verified"})).into_response())
                }
                "/api/me/email/resend" => {
                    let _: Empty = parse(&bytes)?;
                    service::resend(db, account, &ip, now)?;
                    Ok((StatusCode::ACCEPTED, Json(json!({"ok":true}))).into_response())
                }
                "/api/me/email" => {
                    let c: Change = parse(&bytes)?;
                    let step = service::change(
                        db,
                        account,
                        &c.account_key,
                        &c.email,
                        &c.locale,
                        c.lost_old,
                        now,
                    )?;
                    Ok((StatusCode::ACCEPTED, Json(json!({"step":step}))).into_response())
                }
                "/api/me/email/change/confirm" => {
                    let c: Confirm = parse(&bytes)?;
                    let done = service::confirm(db, account, &c.old_code, &c.new_code, now)?;
                    Ok(
                        Json(json!({"email_state":if done{"verified"}else{"hold"}}))
                            .into_response(),
                    )
                }
                "/api/me/email/change/cancel" => {
                    let c: Cancel = parse(&bytes)?;
                    service::cancel(db, account, c.old_code.as_deref(), now)?;
                    Ok(Json(json!({"ok":true})).into_response())
                }
                _ => Err(Error::Denied),
            }
        })
        .await
        .ok_or(Error::Unavailable)?
}
