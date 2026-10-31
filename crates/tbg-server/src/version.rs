//! Public client release discovery; no account data or matchmaking changes.

use axum::{http::header, response::IntoResponse, Json};
use serde_json::json;

pub async fn get() -> impl IntoResponse {
    let latest = option_env!("TBG_LATEST_CLIENT").unwrap_or("0.1.0-native.2");
    let releases = "https://github.com/AlexandreZanata/TBG-Tupi-Battle-Grounds/releases";
    (
        [(header::CACHE_CONTROL, "no-cache, max-age=0")],
        Json(json!({
            "protocol": tbg_core::protocol::PROTOCOL_VERSION,
            "latest_client": latest,
            "downloads": {"linux":releases,"windows":releases,"itch":null}
        })),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn discovery_is_public_uncached_and_names_the_current_protocol() {
        let response = get().await.into_response();
        assert_eq!(
            response.headers()[header::CACHE_CONTROL],
            "no-cache, max-age=0"
        );
        let body = axum::body::to_bytes(response.into_body(), 4096)
            .await
            .unwrap();
        let value: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(value["protocol"], tbg_core::protocol::PROTOCOL_VERSION);
        assert!(value["latest_client"].as_str().is_some());
        assert!(value["downloads"]["linux"]
            .as_str()
            .unwrap()
            .starts_with("https://github.com/"));
        assert!(value.get("accounts").is_none());
    }
}
