//! Palette spheres under the same warm key and hemispherical fill as the arena.

use crate::art::{mesh, palette::*};
use bevy::prelude::*;

pub fn spawn(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    let mat = mats.add(StandardMaterial {
        perceptual_roughness: 0.9,
        ..default()
    });
    let rows = [
        FACTIONS[0],
        FACTIONS[1],
        FACTIONS[2],
        FACTIONS[3],
        ENVIRONMENT,
        UI,
    ];
    for (row, colors) in rows.into_iter().enumerate() {
        for (col, color) in colors.into_iter().enumerate() {
            commands.spawn((
                Mesh3d(meshes.add(mesh::paint(mesh::build(mesh::Shape::Ball(0.38)), color))),
                MeshMaterial3d(mat.clone()),
                Transform::from_xyz(3.8 + col as f32 * 1.2, 0.6, -3.0 + row as f32 * 1.2),
            ));
        }
    }
    let slots = SLOTS.map(|s| format!("{s:?}")).join(" / ");
    commands.spawn((
        Text::new(format!(
            "P15 PALETTE | {slots}\nCrown / Barrow / Ironworks / Wildwood / Environment / UI"
        )),
        TextFont {
            font_size: FontSize::Px(16.0),
            ..default()
        },
        TextColor(INK),
        Node {
            position_type: PositionType::Absolute,
            left: px(12),
            top: px(12),
            max_width: percent(94),
            ..default()
        },
    ));
}
