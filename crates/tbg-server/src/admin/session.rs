//! Operator HTTP claim/sign-in flow and Secure cookie rendering.
use crate::store::operators::{self as db, Error};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use serde_json::json;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Email {
    email: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Claim {
    email: String,
    code: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Key {
    admin_key: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Verify {
    attempt: String,
    code: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Empty {}
fn parse<T: serde::de::DeserializeOwned>(b: &[u8]) -> Result<T, Error> {
    serde_json::from_slice(b).map_err(|_| Error::Invalid)
}
pub(crate) fn execute(
    conn: &mut rusqlite::Connection,
    path: &str,
    bytes: &[u8],
    token: &str,
    ip: &str,
    request: &str,
    attempt: &str,
    now: i64,
    second: bool,
) -> Result<Response, Error> {
    match path {
        "claim/request" => {
            let i: Email = parse(bytes)?;
            let _ = db::claim::request(conn, &i.email, ip, now);
            Ok((StatusCode::ACCEPTED, Json(json!({"ok":true}))).into_response())
        }
        "claim/complete" => {
            let i: Claim = parse(bytes)?;
            Ok(Json(
                json!({"admin_key":db::claim::complete(conn,&i.email,&i.code,ip,request,now)?}),
            )
            .into_response())
        }
        "session/start" => {
            let i: Key = parse(bytes)?;
            if let Ok(Some(grant)) =
                db::sessions::start(conn, &i.admin_key, attempt, ip, now, second)
            {
                return render(grant);
            }
            Ok((StatusCode::ACCEPTED, Json(json!({"attempt":attempt}))).into_response())
        }
        "session/verify" => {
            let i: Verify = parse(bytes)?;
            render(db::sessions::verify(conn, &i.attempt, &i.code, ip, now)?)
        }
        "session" => {
            let p = db::sessions::authenticate(conn, token, None, now)?;
            Ok(Json(p.dto(&db::sessions::csrf(token))).into_response())
        }
        "session/reauth" => {
            let i: Key = parse(bytes)?;
            db::sessions::reauth(conn, token, &i.admin_key, request, now)?;
            Ok(Json(json!({"ok":true})).into_response())
        }
        "session/logout" => {
            let _: Empty = parse(bytes)?;
            db::sessions::logout(conn, token, request, now)?;
            let mut r = Json(json!({"ok":true})).into_response();
            r.headers_mut().insert(
                "set-cookie",
                "__Host-tbg_admin=; Max-Age=0; Path=/; Secure; HttpOnly; SameSite=Strict"
                    .parse()
                    .unwrap(),
            );
            Ok(r)
        }
        _ => Err(Error::Invalid),
    }
}
fn render(grant: db::sessions::Granted) -> Result<Response, Error> {
    let mut r = Json(grant.principal.dto(&grant.csrf)).into_response();
    r.headers_mut().insert(
        "set-cookie",
        format!(
            "__Host-tbg_admin={}; Path=/; Secure; HttpOnly; SameSite=Strict",
            grant.token
        )
        .parse()
        .map_err(|_| Error::Unavailable)?,
    );
    Ok(r)
}
pub fn error(error: Error) -> Response {
    let (status, code) = match error {
        Error::CodeInvalid => (401, "code_invalid"),
        Error::Unauthorized => (401, "auth_required"),
        Error::Forbidden => (403, "forbidden"),
        Error::Invalid => (400, "bad_request"),
        Error::Conflict => (409, "conflict"),
        Error::Reauth => (403, "reauth_required"),
        Error::Limited => (429, "rate_limited"),
        Error::Unavailable => (503, "unavailable"),
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
