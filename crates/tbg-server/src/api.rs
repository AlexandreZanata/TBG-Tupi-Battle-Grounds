//! Read-only JSON API (P13): the leaderboard and player profiles. The
//! leaderboard never touches SQL (in-memory snapshot); profiles run at
//! most three indexed queries. No account ids, key hashes, IPs, or
//! sub-day timestamps ever leave the server.

use std::net::{IpAddr, SocketAddr};
use std::time::Instant;

use axum::extract::{ConnectInfo, Path, RawQuery, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;
use tbg_core::FACTIONS;

use crate::AppState;

/// Paging params without a `serde` derive (the server has no `serde`
/// dependency): `offset` default 0 (clamped to 100 000), `limit` default
/// 50 (clamped to 1–100).
fn paging(raw: Option<String>) -> (usize, usize) {
    let mut offset = 0u32;
    let mut limit = 50u32;
    if let Some(query) = raw {
        for pair in query.split('&') {
            let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
            match (k, v.parse::<u32>()) {
                ("offset", Ok(n)) => offset = n.min(100_000),
                ("limit", Ok(n)) => limit = n.clamp(1, 100),
                _ => {}
            }
        }
    }
    (offset as usize, limit as usize)
}

fn cache_headers() -> [(axum::http::HeaderName, &'static str); 1] {
    [(header::CACHE_CONTROL, "public, max-age=10")]
}

fn limited(app: &AppState, ip: IpAddr) -> bool {
    let Ok(mut limits) = app.api_limits.lock() else {
        return false;
    };
    let now = Instant::now();
    limits.sweep(now);
    !limits.check_api(ip, app.config.api_per_ip_min as u32, now)
}

/// `GET /api/leaderboard?offset=0&limit=50`: rank order with 1-based ranks.
pub async fn leaderboard(
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    State(app): State<AppState>,
    RawQuery(query): RawQuery,
) -> impl IntoResponse {
    let ip = crate::ws::client_ip(peer, &headers);
    if limited(&app, ip) {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({"error": "rate_limited"})),
        )
            .into_response();
    }
    let Some(store) = &app.store else {
        return (StatusCode::NOT_FOUND, Json(json!({"error": "not_found"}))).into_response();
    };
    let snap = store.snapshot();
    let (offset, limit) = paging(query);
    let rows: Vec<_> = snap
        .rows
        .iter()
        .skip(offset)
        .take(limit)
        .map(|r| {
            json!({
                "rank": r.rank, "name": r.name, "rating": r.rating, "games": r.games,
                "wins": r.wins, "losses": r.losses, "draws": r.draws, "win_rate": r.win_rate(),
            })
        })
        .collect();
    (
        cache_headers(),
        Json(json!({
            "total": snap.total, "updated_at": snap.updated_at, "rows": rows,
        })),
    )
        .into_response()
}

/// `GET /api/players/{name}`: public profile (BR-P05) with faction shares
/// and the last 10 ranked matches. Bad shapes 404 without touching the DB.
pub async fn player(
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    State(app): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let ip = crate::ws::client_ip(peer, &headers);
    if limited(&app, ip) {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({"error": "rate_limited"})),
        )
            .into_response();
    }
    if !tbg_core::account::valid_name_shape(&name) {
        return (StatusCode::NOT_FOUND, Json(json!({"error": "not_found"}))).into_response();
    }
    let Some(store) = &app.store else {
        return (StatusCode::NOT_FOUND, Json(json!({"error": "not_found"}))).into_response();
    };
    let name_lower = name.to_ascii_lowercase();
    let Some(stats) = store.player_stats(name_lower.clone()).await else {
        return (StatusCode::NOT_FOUND, Json(json!({"error": "not_found"}))).into_response();
    };
    let rank = store.snapshot().rank_of(&name_lower);
    // First maximum wins ties; shares are finite percentages, never NaN.
    let mut favorite = 0;
    for i in 1..stats.factions.len() {
        if stats.factions[i] > stats.factions[favorite] {
            favorite = i;
        }
    }
    let mut shares = serde_json::Map::new();
    for f in FACTIONS {
        shares.insert(f.name().into(), json!(stats.factions[f.index()]));
    }
    let recent: Vec<_> = stats
        .recent
        .iter()
        .map(|m| {
            json!({
                "date": m.date, "opponent": m.opponent, "result": m.result,
                "delta": m.delta, "reason": m.reason, "duration_s": m.duration_s,
            })
        })
        .collect();
    (
        cache_headers(),
        Json(json!({
            "name": stats.name, "rating": stats.rating, "peak_rating": stats.peak_rating,
            "rank": rank, "games": stats.games, "wins": stats.wins, "losses": stats.losses,
            "draws": stats.draws, "win_rate": win_rate_of(stats.wins, stats.games),
            "joined": stats.joined_day, "factions": shares,
            "favorite_faction": FACTIONS[favorite].name(), "recent": recent,
        })),
    )
        .into_response()
}

fn win_rate_of(wins: u32, games: u32) -> f64 {
    if games == 0 {
        0.0
    } else {
        (f64::from(wins) * 1000.0 / f64::from(games)).round() / 10.0
    }
}
