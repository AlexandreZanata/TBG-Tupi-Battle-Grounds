//! Bounded authenticated emoji library/upload HTTP; decode jobs retain their semaphore on timeout.
use crate::{
    store::emoji::{self, Error},
    AppState,
};
use axum::{
    extract::{Request, State},
    http::{Method, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
pub(crate) async fn handler(State(app): State<AppState>, req: Request) -> Response {
    let mut r =
        match tokio::time::timeout(std::time::Duration::from_secs(5), execute(app, req)).await {
            Ok(Ok(r)) => r,
            Ok(Err(e)) => error(e),
            Err(_) => error(Error::Unavailable),
        };
    r.headers_mut()
        .insert("cache-control", "no-store".parse().unwrap());
    r
}
async fn execute(app: AppState, req: Request) -> Result<Response, Error> {
    let raw = req.method() == Method::POST && req.uri().path() == "/api/me/emoji";
    if raw
        && !req
            .headers()
            .get("content-type")
            .is_some_and(|v| matches!(v.to_str(), Ok("image/png" | "image/gif" | "image/webp")))
    {
        return Err(Error::Unsupported);
    }
    let binding = if raw {
        crate::auth::http::image_binding(&app, req.headers())
    } else {
        crate::auth::http::history_binding(&app, req.headers())
    }
    .map_err(|_| Error::Restricted)?;
    let account = binding.account;
    let now = app.auth.now().map_err(|_| Error::Unavailable)?;
    let permit = app
        .history_reads
        .clone()
        .try_acquire_owned()
        .map_err(|_| Error::Unavailable)?;
    let path = req.uri().path().to_owned();
    let method = req.method().clone();
    let label = url::form_urlencoded::parse(req.uri().query().unwrap_or("").as_bytes())
        .find(|(k, _)| k == "label")
        .map(|(_, v)| v.into_owned())
        .unwrap_or_default();
    let mime = req
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_owned();
    let store = app.store.as_ref().ok_or(Error::Unavailable)?;
    let upload = method == Method::POST && path == "/api/me/emoji";
    let bytes = axum::body::to_bytes(req.into_body(), if upload { 1048576 } else { 4096 })
        .await
        .map_err(|_| Error::TooLarge)?;
    let sheet = if upload {
        store
            .release(move |db| emoji::allowed(db, account, now))
            .await
            .ok_or(Error::Unavailable)??;
        let work = app
            .emoji_work
            .clone()
            .try_acquire_owned()
            .map_err(|_| Error::Unavailable)?;
        let source = bytes.clone();
        let job = tokio::task::spawn_blocking(move || {
            let _permit = work;
            super::normalize::normalize(&source, &mime)
        });
        let sheet = tokio::time::timeout(std::time::Duration::from_secs(2), job)
            .await
            .map_err(|_| Error::Unavailable)?
            .map_err(|_| Error::Unavailable)?
            .map_err(|e| match e {
                super::normalize::Error::Type => Error::Unsupported,
                super::normalize::Error::Size => Error::TooLarge,
                _ => Error::Image(e.detail()),
            })?;
        Some(sheet)
    } else {
        None
    };
    let result = store
        .release(move |db| {
            let _permit = permit;
            if binding.revoked() {
                return Err(Error::Restricted);
            }
            if let Some(sheet) = sheet {
                let id = emoji::upload(db, account, &label, sheet, now)?;
                return Ok((
                    StatusCode::CREATED,
                    Json(json!({"id":id,"status":"pending"})),
                )
                    .into_response());
            }
            match (method.as_str(), path.as_str()) {
                ("GET", "/api/me/emoji") => Ok(Json(emoji::library(db, account)?).into_response()),
                ("PUT", "/api/me/emoji/slots") => Ok(Json(emoji::equip(
                    db,
                    account,
                    serde_json::from_slice(&bytes).map_err(|_| Error::Invalid)?,
                    now,
                )?)
                .into_response()),
                _ => {
                    let id = path
                        .split('/')
                        .nth(4)
                        .and_then(|v| v.parse().ok())
                        .ok_or(Error::Invalid)?;
                    if method == Method::DELETE {
                        emoji::withdraw(db, account, id, now)?;
                        Ok(Json(json!({"ok":true})).into_response())
                    } else if method == Method::GET && path.ends_with("/sheet.png") {
                        Ok((
                            [("content-type", "image/png")],
                            emoji::private(db, account, id)?,
                        )
                            .into_response())
                    } else {
                        Err(Error::Invalid)
                    }
                }
            }
        })
        .await
        .ok_or(Error::Unavailable)?;
    result
}
fn error(e: Error) -> Response {
    let detail = if let Error::Image(detail) = e {
        Some(detail)
    } else {
        None
    };
    let (status, code) = match e {
        Error::TooLarge => (413, "too_large"),
        Error::Unsupported => (415, "unsupported_type"),
        Error::Image(_) => (422, "image_invalid"),
        Error::Invalid => (422, "image_invalid"),
        Error::Unverified => (403, "unverified"),
        Error::Restricted => (403, "restricted"),
        Error::Disabled => (503, "uploads_disabled"),
        Error::QueueFull => (503, "queue_full"),
        Error::Limited => (429, "rate_limited"),
        Error::NotFound => (404, "not_found"),
        Error::Unavailable => (503, "busy"),
    };
    let mut r = (
        StatusCode::from_u16(status).unwrap_or(StatusCode::SERVICE_UNAVAILABLE),
        Json(json!({"error":code,"detail":detail})),
    )
        .into_response();
    if status == 429 || status == 503 {
        r.headers_mut().insert("retry-after", "60".parse().unwrap());
    }
    r
}
