//! Authenticated owner reads, revisioned preferences and permission-checked public projections.

use crate::{
    store::player_history::{self, Error, Query, Subject},
    AppState,
};
use axum::{
    extract::{ConnectInfo, Request, State},
    http::{Method, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{net::SocketAddr, time::Duration};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Preference {
    visibility: String,
}

pub(crate) async fn handler(
    State(app): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    request: Request,
) -> Response {
    let result = handle(&app, peer, request).await;
    let mut response = match result {
        Ok(value) => Json(value).into_response(),
        Err((status, code)) => (status, Json(json!({"error":code}))).into_response(),
    };
    response.headers_mut().insert(
        "cache-control",
        axum::http::HeaderValue::from_static("private, no-store"),
    );
    response
}

type ApiError = (StatusCode, &'static str);
fn failure(error: Error) -> ApiError {
    match error {
        Error::Invalid => (StatusCode::BAD_REQUEST, "invalid_query"),
        Error::NotFound => (StatusCode::NOT_FOUND, "not_found"),
        Error::Private => (StatusCode::FORBIDDEN, "history_private"),
        Error::Unauthorized => (StatusCode::UNAUTHORIZED, "auth_required"),
        Error::Revision => (StatusCode::CONFLICT, "privacy_revision"),
        _ => (StatusCode::SERVICE_UNAVAILABLE, "history_unavailable"),
    }
}

async fn handle(app: &AppState, peer: SocketAddr, request: Request) -> Result<Value, ApiError> {
    if crate::api::limited(app, crate::ws::client_ip(peer, request.headers())) {
        return Err((StatusCode::TOO_MANY_REQUESTS, "rate_limited"));
    }
    if !app.config.history_api {
        return Err((StatusCode::SERVICE_UNAVAILABLE, "history_unavailable"));
    }
    let store = app
        .store
        .as_ref()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "history_unavailable"))?;
    let path = request.uri().path().to_owned();
    let headers = request.headers().clone();
    let owner = path.starts_with("/api/me/");
    let binding = if owner {
        Some(
            crate::auth::http::history_binding(app, &headers)
                .map_err(|_| (StatusCode::UNAUTHORIZED, "auth_required"))?,
        )
    } else {
        None
    };
    if !owner && !app.config.history_public {
        return Err((StatusCode::FORBIDDEN, "history_private"));
    }
    let subject = match &binding {
        Some(b) => Subject::Owner {
            account: b.account,
            name: b.profile.name.clone(),
        },
        None => {
            let name = path
                .strip_prefix("/api/players/")
                .and_then(|s| s.split('/').next())
                .filter(|n| tbg_core::account::valid_name_shape(n))
                .ok_or((StatusCode::NOT_FOUND, "not_found"))?;
            if app
                .config
                .auth_clock
                .now()
                .is_ok_and(|now| app.config.moderation.hidden(name, now))
            {
                return Err((StatusCode::NOT_FOUND, "not_found"));
            }
            Subject::Public(name.into())
        }
    };
    let query = Query::parse(request.uri().query()).map_err(failure)?;
    let preference = path == "/api/me/preferences/history";
    let entry = path.split("/matches/").nth(1).map(str::to_owned);
    let save = if request.method() == Method::PATCH {
        if !preference {
            return Err((StatusCode::NOT_FOUND, "not_found"));
        }
        if headers.get_all("if-match").iter().count() != 1 {
            return Err((StatusCode::BAD_REQUEST, "privacy_revision"));
        }
        let revision = headers
            .get("if-match")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix('"'))
            .and_then(|v| v.strip_suffix('"'))
            .and_then(|v| v.parse::<i64>().ok())
            .filter(|v| *v >= 0)
            .ok_or((StatusCode::BAD_REQUEST, "privacy_revision"))?;
        let body = axum::body::to_bytes(request.into_body(), 1024)
            .await
            .map_err(|_| (StatusCode::BAD_REQUEST, "invalid_body"))?;
        let p: Preference =
            serde_json::from_slice(&body).map_err(|_| (StatusCode::BAD_REQUEST, "invalid_body"))?;
        if p.visibility == "public" && !app.config.history_public {
            return Err((StatusCode::SERVICE_UNAVAILABLE, "history_unavailable"));
        }
        Some((p.visibility, revision))
    } else {
        None
    };
    let permit = app
        .history_reads
        .clone()
        .try_acquire_owned()
        .map_err(|_| (StatusCode::TOO_MANY_REQUESTS, "rate_limited"))?;
    let account = binding.as_ref().map(|b| b.account);
    let queued_binding = binding.clone();
    let clock = app.config.auth_clock.clone();
    let result = tokio::time::timeout(
        Duration::from_secs(2),
        store.release(move |conn| {
            // The permit remains held by the actual queued SQL work, even after HTTP timeout.
            let _permit = permit;
            if queued_binding
                .as_ref()
                .is_some_and(|b| !clock.now().is_ok_and(|now| b.valid(now)))
            {
                return Err(Error::Unauthorized);
            }
            if preference {
                let account = account.ok_or(Error::NotFound)?;
                match save {
                    Some((visibility, revision)) => {
                        player_history::set_preference(conn, account, &visibility, revision)
                    }
                    None => player_history::preference(conn, account),
                }
            } else if let Some(entry) = entry {
                player_history::detail(conn, &subject, &entry)
            } else {
                player_history::list(conn, &subject, &query)
            }
        }),
    )
    .await
    .map_err(|_| (StatusCode::SERVICE_UNAVAILABLE, "history_unavailable"))?
    .ok_or((StatusCode::SERVICE_UNAVAILABLE, "history_unavailable"))?
    .map_err(failure)?;
    if let Some(old) = binding {
        let current = crate::auth::http::history_binding(app, &headers)
            .map_err(|_| (StatusCode::UNAUTHORIZED, "auth_required"))?;
        if current.account != old.account || current.family != old.family {
            return Err((StatusCode::UNAUTHORIZED, "auth_required"));
        }
    }
    Ok(result)
}

/// Apply to legacy profile replies and extraction errors too; shared caches cannot
/// retain old history while the owner is switching the current preference.
pub(crate) async fn no_store(request: Request, next: axum::middleware::Next) -> Response {
    let guarded = request.uri().path().starts_with("/api/players/")
        || request.uri().path().starts_with("/api/me/");
    let presence = matches!(
        request.uri().path(),
        "/api/live" | "/api/online" | "/api/internal/presence"
    );
    let mut response = next.run(request).await;
    if guarded || presence && !response.status().is_success() {
        response.headers_mut().insert(
            "cache-control",
            axum::http::HeaderValue::from_static("private, no-store"),
        );
        response.headers_mut().remove("etag");
    }
    response
}
