//! Authoritative 5v5 room server: one match, ten seats, bots on every empty seat.

pub mod conn;
pub mod room;

use axum::{routing::get, Router};
use tokio::net::TcpListener;

/// HTTP routes: `/moba/ws` (game socket) and `/moba/health`.
pub fn router(room: room::RoomHandle) -> Router {
    Router::new()
        .route("/moba/ws", get(conn::upgrade))
        .route("/moba/health", get(|| async { "ok" }))
        .with_state(room)
}

/// Serves until the listener fails. Snapshots are small and frequent, so
/// Nagle's algorithm is disabled on every socket.
pub async fn serve(listener: TcpListener, seed: u64) -> std::io::Result<()> {
    use axum::serve::ListenerExt;
    let room = room::spawn(seed);
    let listener = listener.tap_io(|tcp| {
        if let Err(error) = tcp.set_nodelay(true) {
            tracing::warn!(%error, "TCP_NODELAY unavailable");
        }
    });
    axum::serve(listener, router(room)).await
}
