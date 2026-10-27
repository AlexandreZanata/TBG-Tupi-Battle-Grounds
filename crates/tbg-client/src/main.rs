//! `tbg-client` placeholder entry point.
//!
//! Phase 00: verifies the Bevy dependency resolves and links.
//! Phase 01 builds the first Bevy/WebAssembly arena prototype here.

fn main() {
    // Scaffolding only: headless app core, no window, no scene yet.
    // Render/audio/windowing plugins arrive in Phase 01.
    let _app = bevy::app::App::new().add_plugins(bevy::MinimalPlugins);
    println!("tbg-client 0.1.0 — Phase 00 scaffolding (not playable)");
}
