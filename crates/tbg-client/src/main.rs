//! `tbg-client` browser spike: Bevy WebGL2 placeholder arena + RTT overlay.
//!
//! P1 proves the WASM build, touch input, and WebSocket ping. No gameplay.

use bevy::prelude::*;

mod arena;
mod camera;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Tupi Battle Grounds".into(),
                canvas: Some("#bevy".into()),
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb_u8(0xA9, 0xD8, 0xF0)))
        .add_plugins((camera::plugin, arena::plugin))
        .run();
}
