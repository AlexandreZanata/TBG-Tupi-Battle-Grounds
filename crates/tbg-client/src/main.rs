//! `tbg-client` browser spike: Bevy WebGL2 placeholder arena + RTT overlay.
//!
//! P1 proves the WASM build, touch input, and WebSocket ping. No gameplay.

use bevy::prelude::*;

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
        .add_systems(Startup, setup_camera)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(8.0, 12.0, -6.0).looking_at(Vec3::new(8.0, 0.0, 0.0), Vec3::Y),
    ));
}
