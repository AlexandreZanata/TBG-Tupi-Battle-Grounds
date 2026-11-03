//! Approved-only immutable public sheets, with bounded bytes and no original-upload access.
use crate::{
    store::{emoji, reports::hex16},
    AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
pub(crate) async fn handler(State(app): State<AppState>, Path(asset): Path<String>) -> Response {
    let Some(id) = asset.strip_suffix(".png").and_then(hex16) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Some(store) = &app.store else {
        return StatusCode::NOT_FOUND.into_response();
    };
    if !matches!(
        store.release(move |db| emoji::approved(db, &id)).await,
        Some(Ok(true))
    ) {
        return StatusCode::NOT_FOUND.into_response();
    }
    let bytes = if let Some(bytes) = app.emoji_cache.get(&id) {
        bytes
    } else {
        let Some(Ok(bytes)) = store.release(move |db| emoji::public(db, &id)).await else {
            return StatusCode::NOT_FOUND.into_response();
        };
        app.emoji_cache.insert(id, bytes.clone());
        std::sync::Arc::new(bytes)
    };
    (
        [
            ("content-type", "image/png"),
            ("cache-control", "public, max-age=31536000, immutable"),
            ("x-content-type-options", "nosniff"),
        ],
        bytes.as_ref().clone(),
    )
        .into_response()
}
