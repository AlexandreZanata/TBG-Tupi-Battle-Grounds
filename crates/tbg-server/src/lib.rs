//! `tbg-server`: authoritative game server. Runs `tbg-core` matches at a
//! fixed tick and talks to browsers over WebSocket (docs/ARCHITECTURE.md §5).

use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex, OnceLock};

use axum::extract::{ConnectInfo, DefaultBodyLimit, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::serve::ListenerExt;
use axum::{Json, Router};
use tokio::net::TcpListener;
use tokio::sync::{mpsc, watch};

mod api;
pub mod auth;
pub mod botclient;
pub mod build_info;
mod chan;
pub mod config;
mod limits;
mod live;
mod lobby;
mod match_task;
pub mod metrics;
mod names;
mod patch_notes;
pub mod ranked;
pub mod release;
pub mod stats;
pub mod store;
mod version;
mod ws;

pub use config::Config;
use metrics::Metrics;

/// Shared by every request handler.
#[derive(Clone)]
pub(crate) struct AppState {
    pub auth: Arc<auth::Service>,
    pub lobby: mpsc::Sender<chan::LobbyCmd>,
    pub metrics: Arc<Metrics>,
    pub config: Arc<Config>,
    pub ips: Arc<Mutex<HashMap<IpAddr, usize>>>,
    pub next_conn_id: Arc<AtomicU64>,
    /// P13: snapshot reads and per-IP API limits.
    pub store: Option<Arc<store::Store>>,
    pub api_limits: Arc<Mutex<limits::Limits>>,
    /// P47: the lobby's live snapshot, read-only (`/api/live`, `/api/online`).
    pub live: watch::Receiver<Arc<live::LiveSnapshot>>,
    /// P66: slot identity, admission gate and release directory.
    pub release: Arc<release::Runtime>,
}

/// Handles the binary keeps to drive graceful shutdown.
#[derive(Clone, Default)]
pub struct Shared {
    pub metrics: Arc<Metrics>,
    /// P66: set by `run_with`; the shutdown drain reads its gate and load.
    pub release: Arc<OnceLock<Arc<release::Runtime>>>,
    /// P66: the opened store, so shutdown waits for queued records too.
    pub store: Arc<OnceLock<Arc<store::Store>>>,
}

/// Serve until the listener fails. Tests bind `127.0.0.1:0` and call this.
pub async fn run(config: Config, listener: TcpListener) -> std::io::Result<()> {
    run_with(config, listener, Shared::default()).await
}

pub async fn run_with(
    config: Config,
    listener: TcpListener,
    shared: Shared,
) -> std::io::Result<()> {
    let metrics = shared.metrics.clone();
    let store =
        config
            .db
            .as_ref()
            .and_then(|path| match store::Store::open_with(path, config.ranked) {
                Ok(r) => Some(Arc::new(r)),
                Err(e) => {
                    tracing::error!(
                        "match records disabled, cannot open {}: {e}",
                        path.display()
                    );
                    None
                }
            });
    // P71: a configured database that failed to open or migrate is unhealthy, so a
    // deploy check never passes a server that runs without its records.
    let store_failed = config.db.is_some() && store.is_none();
    let config = Arc::new(config);
    let release = shared
        .release
        .get_or_init(|| release::Runtime::new(&config))
        .clone();
    if let Some(store) = &store {
        let _ = shared.store.set(store.clone());
        tokio::spawn(release::run(
            release.clone(),
            store.clone(),
            metrics.clone(),
        ));
    } else if release.identity.is_some() {
        tracing::error!("a release slot needs TBG_DB; it will never admit new matches");
    }
    let (lobby_tx, lobby_rx) = mpsc::channel(chan::LOBBY_QUEUE);
    if config.auth_origins.iter().any(|v| !auth::valid_origin(v)) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid configured authentication origin",
        ));
    }
    let auth = Arc::new(auth::Service::new(
        store.clone(),
        lobby_tx.clone(),
        config.auth_clock.clone(),
        config.clone(),
    ));
    auth::Service::start(&auth);
    // P47: the lobby publishes the live snapshot; the API only reads it.
    let (live_tx, live_rx) = watch::channel(Arc::default());
    tokio::spawn(lobby::run(
        lobby_rx,
        lobby_tx.clone(),
        config.clone(),
        metrics.clone(),
        store.clone(),
        release.clone(),
        live_tx,
    ));
    let state = AppState {
        auth,
        lobby: lobby_tx,
        metrics,
        config: config.clone(),
        ips: Arc::default(),
        next_conn_id: Arc::new(AtomicU64::new(1)),
        store,
        api_limits: Arc::default(),
        live: live_rx,
        release,
    };
    let mut app = Router::new()
        .route(
            "/api/auth/{channel}/{operation}",
            post(auth::http::handler).layer(DefaultBodyLimit::max(4096)),
        )
        .route(
            "/health",
            get(move || async move {
                if store_failed {
                    (
                        axum::http::StatusCode::SERVICE_UNAVAILABLE,
                        "store unavailable",
                    )
                } else {
                    (StatusCode::OK, "ok")
                }
            }),
        )
        .route("/ws", get(ws::handler))
        .route("/ws/{slot}", get(ws::slot_handler))
        .route("/ready", get(release::api::ready))
        .route("/api/admission", get(release::api::admission))
        .route("/api/release", get(release::api::status))
        .route("/metrics", get(metrics_handler))
        .route("/api/leaderboard", get(api::leaderboard))
        .route("/api/players/{name}", get(api::player))
        .route("/api/live", get(api::live))
        .route("/api/version", get(version::get))
        .route("/api/patch-notes", get(patch_notes::list))
        .route("/api/patch-notes/{id}", get(patch_notes::detail))
        .route("/api/online", get(api::online))
        .with_state(state);
    if let Some(dir) = &config.static_dir {
        app = app.fallback_service(tower_http::services::ServeDir::new(dir));
    }
    let service = app.into_make_service_with_connect_info::<SocketAddr>();
    // P9: instant-feeling taps — disable Nagle on every socket; log, never panic.
    let listener = listener.tap_io(|stream: &mut tokio::net::TcpStream| {
        if let Err(e) = stream.set_nodelay(true) {
            tracing::warn!("failed to set TCP_NODELAY: {e:#}");
        }
    });
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
