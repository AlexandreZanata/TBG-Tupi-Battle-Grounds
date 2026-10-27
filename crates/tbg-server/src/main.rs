//! `tbg-server` placeholder entry point.
//!
//! Phase 00: verifies the Axum/Tokio/SQLite dependency set resolves.
//! Matchmaking, authoritative simulation, and persistence arrive in
//! Phase 04. This binary must never depend on Bevy rendering.

#[tokio::main]
async fn main() {
    // Scaffolding only: no listener, no routes, no database yet.
    let _router: axum::Router = axum::Router::new();
    println!("tbg-server 0.1.0 — Phase 00 scaffolding (no networking yet)");
}
