//! `tbg-client`: the Bevy browser client (WASM/WebGL2).
//!
//! Renders `MatchView`s and sends `PlayerIntention`s; never decides
//! outcomes. Practice matches run `tbg-core` locally against the bot.

use bevy::prelude::*;

mod arena;
mod autoplay;
mod camera;
mod config;
mod link;
mod local;
mod net;
mod overlay;
mod stress;
mod sync;
mod ui;
mod visuals;

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    Menu,
    InMatch,
    Result,
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Tupi Battle Grounds".into(),
                canvas: Some("#bevy".into()),
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }),))
        .insert_resource(ClearColor(Color::srgb_u8(0xA9, 0xD8, 0xF0)))
        .insert_resource(Time::<Fixed>::from_hz(20.0))
        .init_state::<GameState>()
        .add_plugins((
            camera::plugin,
            arena::plugin,
            overlay::plugin,
            net::plugin,
            link::plugin,
            visuals::plugin,
            sync::plugin,
            stress::plugin,
            ui::plugin,
            autoplay::plugin,
        ))
        .run();
}
