//! Shared game modules and native Android entry; online outcomes remain server-owned.

use bevy::prelude::*;

mod arena;
mod art;
mod audio;
mod auth;
mod autoplay;
mod camera;
#[cfg(all(
    not(target_os = "android"),
    any(debug_assertions, feature = "ui-acceptance")
))]
mod capture;
mod celebration;
mod config;
mod crowd;
mod emoji;
mod gallery;
mod i18n;
mod link;
mod local;
mod net;
mod overlay;
mod platform;
mod stress;
mod sync;
mod training;
mod ui;
mod vfx;
mod visuals;

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    Menu,
    Queue,
    InMatch,
    Training,
    Result,
    /// `?gallery`: art review scene.
    Gallery,
}

pub mod app;

#[cfg(target_os = "android")]
#[bevy::prelude::bevy_main]
fn main() {
    app::run();
}
