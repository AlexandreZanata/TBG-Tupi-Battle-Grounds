//! `tbg-client`: the Bevy browser client (WASM/WebGL2).
//!
//! Renders `MatchView`s and sends `PlayerIntention`s; never decides
//! outcomes. Practice matches run `tbg-core` locally against the bot.

use bevy::asset::AssetMetaCheck;
use bevy::prelude::*;

mod arena;
mod art;
mod audio;
mod autoplay;
mod camera;
mod config;
mod gallery;
mod i18n;
mod link;
mod local;
mod net;
mod overlay;
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
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Tupi Battle Grounds".into(),
                        canvas: Some("#bevy".into()),
                        fit_canvas_to_parent: true,
                        ..default()
                    }),
                    ..default()
                })
                // Static hosts answer unknown `.meta` paths with index.html
                // (200), which the asset loader would try to parse.
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                }),
        )
        .insert_resource(ClearColor(art::palette::SKY_TOP))
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
            autoplay::plugin,
            gallery::plugin,
            i18n::plugin,
            vfx::plugin,
            audio::plugin,
        ))
        .run();
}
