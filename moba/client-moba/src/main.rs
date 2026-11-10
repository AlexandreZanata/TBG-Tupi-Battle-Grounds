//! Desktop/browser entry of the TBG 5v5 MOBA client.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod app;
// The 1v1 champion figures, kits and animation, compiled from the 1v1
// sources so both modes always show exactly the same champions (D-112).
// ponytail: path-shared until a shared art crate exists (needs a 1v1 release).
// Lints for shared 1v1 sources are owned by the 1v1 workspace's Clippy run.
#[path = "../../../crates/tbg-client/src/art/mod.rs"]
#[allow(dead_code, unused_imports, unused_variables, clippy::all)]
mod art;
mod camera;
// Review screenshots write files, so they exist only in desktop builds.
#[cfg(not(target_arch = "wasm32"))]
mod capture;
mod config;
mod hud;
mod i18n;
mod input;
mod link;
mod menu;
mod palette;
mod scene;
mod units;
// The 1v1 footprint solver the shared art's clearance tests measure against.
#[cfg(test)]
#[path = "../../../crates/tbg-client/src/crowd.rs"]
#[allow(dead_code, clippy::all)]
mod crowd;
#[allow(dead_code)]
mod vfx;
/// Constants the shared 1v1 art reads from the 1v1 `visuals` module.
mod visuals {
    /// Mirrors `crates/tbg-client/src/visuals/cues.rs`.
    pub const CORPSE_SECS: f32 = 0.9;
}

fn main() {
    app::run();
}
