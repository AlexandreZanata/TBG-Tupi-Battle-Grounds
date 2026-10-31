//! `tbg-client`: the Bevy browser client (WASM/WebGL2).
//!
//! Renders `MatchView`s and sends `PlayerIntention`s; never decides
//! outcomes. Practice matches run `tbg-core` locally against the bot.

#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;

mod arena;
mod art;
mod audio;
mod autoplay;
mod camera;
#[cfg(debug_assertions)]
mod capture;
mod celebration;
mod config;
mod crowd;
mod gallery;
mod i18n;
mod link;
mod local;
mod net;
mod overlay;
mod platform;
mod stress;
mod sync;
mod ui;
mod vfx;
mod visuals;

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    Menu,
    Queue,
    InMatch,
    Result,
    /// `?gallery`: art review scene.
    Gallery,
}

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    if std::env::args().any(|arg| arg == "--version") {
        println!(
            "Tupi Battle Grounds {} · protocol {} · source {}",
            env!("CARGO_PKG_VERSION"),
            tbg_core::protocol::PROTOCOL_VERSION,
            option_env!("TBG_SOURCE_COMMIT").unwrap_or("development")
        );
        return;
    }
    #[cfg(not(target_arch = "wasm32"))]
    let window = platform::window::settings();
    #[cfg(target_arch = "wasm32")]
    let window = Window {
        title: "Tupi Battle Grounds".into(),
        canvas: Some("#bevy".into()),
        fit_canvas_to_parent: true,
        ..default()
    };
    let plugins = DefaultPlugins;
    #[cfg(not(target_arch = "wasm32"))]
    let plugins = plugins.set(platform::window::log_plugin());
    App::new()
        .add_plugins(
            plugins
                .set(WindowPlugin {
                    primary_window: Some(window),
                    ..default()
                })
                // Static hosts answer unknown `.meta` paths with index.html
                // (200), which the asset loader would try to parse.
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    #[cfg(not(target_arch = "wasm32"))]
                    file_path: platform::window::asset_path(),
                    ..default()
                }),
        )
        .insert_resource(ClearColor(if cfg!(target_os = "linux") {
            Color::srgb_u8(7, 15, 22)
        } else {
            art::palette::SKY_TOP
        }))
        .insert_resource(Time::<Fixed>::from_hz(20.0))
        .init_state::<GameState>()
        .add_plugins((
            art::plugin,
            camera::plugin,
            arena::plugin,
            overlay::plugin,
            link::plugin,
            visuals::plugin,
            sync::plugin,
            stress::plugin,
            ui::plugin,
            celebration::plugin,
            autoplay::plugin,
            gallery::plugin,
            i18n::plugin,
            vfx::plugin,
            audio::plugin,
        ))
        .add_plugins(platform_plugin)
        .run();
}

fn platform_plugin(app: &mut App) {
    #[cfg(debug_assertions)]
    capture::plugin(app);
    #[cfg(not(target_arch = "wasm32"))]
    platform::window::plugin(app);
    #[cfg(target_arch = "wasm32")]
    let _ = app;
}
