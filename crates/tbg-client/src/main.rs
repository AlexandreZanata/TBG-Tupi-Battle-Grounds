//! `tbg-client` placeholder entry point.
//!
//! Phase 1 (docs/ROADMAP.md) turns this into the Bevy WebGL2 arena spike.

fn main() {
    // Headless app core only: no window or renderer yet.
    let _app = bevy::app::App::new().add_plugins(bevy::MinimalPlugins);
    println!(
        "tbg-client {} (protocol {}): scaffolding, not playable",
        env!("CARGO_PKG_VERSION"),
        tbg_core::protocol::PROTOCOL_VERSION
    );
}
