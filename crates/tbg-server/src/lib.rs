//! `tbg-server`: authoritative game server. Runs `tbg-core` matches at a
//! fixed tick and talks to browsers over WebSocket (docs/ARCHITECTURE.md §5).

use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex};

use axum::extract::{ConnectInfo, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use tokio::net::TcpListener;
use tokio::sync::mpsc;

pub mod botclient;
mod chan;
pub mod config;
mod lobby;
mod match_task;
pub mod metrics;
mod names;
mod ws;

pub use config::Config;
use metrics::Metrics;

/// Shared by every request handler.
#[derive(Clone)]
pub(crate) struct AppState {
    pub lobby: mpsc::Sender<chan::LobbyCmd>,
    pub metrics: Arc<Metrics>,
    pub config: Arc<Config>,
    pub ips: Arc<Mutex<HashMap<IpAddr, usize>>>,
    pub next_conn_id: Arc<AtomicU64>,
}

/// Serve until the listener fails. Tests bind `127.0.0.1:0` and call this.
pub async fn run(config: Config, listener: TcpListener) -> std::io::Result<()> {
    let metrics = Arc::new(Metrics::default());
    let config = Arc::new(config);
    let (lobby_tx, lobby_rx) = mpsc::channel(chan::LOBBY_QUEUE);
    tokio::spawn(lobby::run(
        lobby_rx,
        lobby_tx.clone(),
        config.clone(),
        metrics.clone(),
    ));
    let state = AppState {
        lobby: lobby_tx,
        metrics,
        config: config.clone(),
        ips: Arc::default(),
        next_conn_id: Arc::new(AtomicU64::new(1)),
    };
    let mut app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/ws", get(ws::handler))
        .route("/metrics", get(metrics_handler))
        .with_state(state);
    if let Some(dir) = &config.static_dir {
        app = app.fallback_service(tower_http::services::ServeDir::new(dir));
    }
    let service = app.into_make_service_with_connect_info::<SocketAddr>();
    axum::serve(listener, service).await
}

/// Live counters, only for local requests that did not come through a proxy.
async fn metrics_handler(
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    State(app): State<AppState>,
) -> Response {
    if peer.ip().is_loopback() && !headers.contains_key("x-forwarded-for") {
        Json(app.metrics.snapshot()).into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}
