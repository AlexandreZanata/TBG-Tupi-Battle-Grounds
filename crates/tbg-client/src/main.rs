//! Desktop/browser entry for the shared game library.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
fn main() {
    tbg_client::app::run();
}
