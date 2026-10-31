//! Public, immutable release notes compiled from the reviewed bilingual catalog.
use axum::{
    extract::Path,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::Value;
use std::sync::OnceLock;

fn catalog() -> &'static Value {
    static CATALOG: OnceLock<Value> = OnceLock::new();
    CATALOG.get_or_init(|| {
        serde_json::from_str(include_str!("../../../docs/patches/catalog.json"))
            .expect("validated repository patch catalog")
    })
}

pub async fn list() -> impl IntoResponse {
    (
        [(header::CACHE_CONTROL, "public, max-age=60")],
        Json(catalog().clone()),
    )
}

pub async fn detail(Path(id): Path<String>) -> Response {
    if id.len() > 64 || !id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') {
        return StatusCode::NOT_FOUND.into_response();
    }
    match catalog()["notes"].as_array().and_then(|notes| {
        notes
            .iter()
            .find(|note| note["id"].as_str() == Some(id.as_str()))
    }) {
        Some(note) => (
            [(header::CACHE_CONTROL, "public, max-age=60")],
            Json(note.clone()),
        )
            .into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn public_list_and_details_are_bounded_bilingual_and_read_only() {
        let response = list().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers()[header::CACHE_CONTROL],
            "public, max-age=60"
        );
        let bytes = axum::body::to_bytes(response.into_body(), 262144)
            .await
            .unwrap();
        let payload: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(payload["schema"], 1);
        for note in payload["notes"].as_array().unwrap() {
            for lang in ["en", "pt-BR"] {
                assert!(!note["title"][lang].as_str().unwrap().is_empty());
            }
            assert_eq!(
                detail(Path(note["id"].as_str().unwrap().into()))
                    .await
                    .status(),
                StatusCode::OK
            );
        }
        assert_eq!(
            detail(Path("missing".into())).await.status(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            detail(Path("../private".into())).await.status(),
            StatusCode::NOT_FOUND
        );
    }
}
