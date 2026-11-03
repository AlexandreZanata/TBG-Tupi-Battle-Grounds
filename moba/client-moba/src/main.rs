//! Desktop/browser entry of the TBG 5v5 MOBA client.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod app;
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

fn main() {
    app::run();
}
