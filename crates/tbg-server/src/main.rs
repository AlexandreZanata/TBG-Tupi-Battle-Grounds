//! `tbg-server` placeholder entry point.
//!
//! Phase 1 (docs/ROADMAP.md) adds the WebSocket ping route; Phase 5 adds
//! matchmaking and the authoritative match loop. Never depends on Bevy.

#[tokio::main]
async fn main() {
    // Scaffolding only: no listener or routes yet.
    let _router: axum::Router = axum::Router::new();
    println!(
        "tbg-server {} (protocol {}): scaffolding, no networking yet",
        env!("CARGO_PKG_VERSION"),
        tbg_core::protocol::PROTOCOL_VERSION
    );
}
