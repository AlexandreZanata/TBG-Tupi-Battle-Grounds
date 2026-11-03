//! Player report HTTP admission; evidence and target selection remain store-authoritative.
use crate::{
    store::reports::{self, Error},
    AppState,
};
use axum::{
    extract::{Request, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
pub(crate) async fn handler(State(app): State<AppState>, req: Request) -> Response {
    let mut response =
        match tokio::time::timeout(std::time::Duration::from_secs(5), execute(app, req)).await {
            Ok(Ok(id)) => (StatusCode::CREATED, Json(json!({"id":id}))).into_response(),
            Ok(Err(e)) => failure(e),
            Err(_) => failure(Error::Unavailable),
        };
    response
        .headers_mut()
        .insert("cache-control", "no-store".parse().unwrap());
    response
}
async fn execute(app: AppState, req: Request) -> Result<i64, Error> {
    let binding =
        crate::auth::http::history_binding(&app, req.headers()).map_err(|_| Error::Forbidden)?;
    let permit = app
        .history_reads
        .clone()
        .try_acquire_owned()
        .map_err(|_| Error::Unavailable)?;
    let bytes = axum::body::to_bytes(req.into_body(), 4096)
        .await
        .map_err(|_| Error::Invalid)?;
    let input = serde_json::from_slice(&bytes).map_err(|_| Error::Invalid)?;
    let store = app.store.as_ref().ok_or(Error::Unavailable)?;
    let now = app.auth.now().map_err(|_| Error::Unavailable)?;
    store
        .release(move |db| {
            let _permit = permit;
            if binding.revoked() {
                return Err(Error::Forbidden);
            }
            reports::file(db, binding.account, input, now)
        })
        .await
        .ok_or(Error::Unavailable)?
}
fn failure(error: Error) -> Response {
    let (status, code) = match error {
        Error::Invalid => (400, "bad_request"),
        Error::NotEligible => (422, "not_eligible"),
        Error::Duplicate => (409, "duplicate"),
        Error::Limited => (429, "rate_limited"),
        Error::Disabled => (503, "reports_disabled"),
        Error::Forbidden => (403, "forbidden"),
        Error::Unavailable => (503, "unavailable"),
    };
    let mut r = (
        StatusCode::from_u16(status).unwrap_or(StatusCode::SERVICE_UNAVAILABLE),
        Json(json!({"error":code})),
    )
        .into_response();
    if status == 429 {
        r.headers_mut()
            .insert("retry-after", "86400".parse().unwrap());
    }
    r
}
