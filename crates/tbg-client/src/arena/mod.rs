//! Diorama assembly: shared vertex-colored layers and authoritative Stronghold views.

use crate::art::mesh::{self, Shape};
use bevy::{light::NotShadowCaster, prelude::*};

mod camp;
mod cliffs;
mod geometry;
mod ground;
mod life;
mod lighting;
mod props;
mod rocks;
pub(super) mod theme;
mod themes;
mod towers;
mod trees;
mod water;

pub(super) type Part = (Shape, Color, Transform);

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_arena)
        .add_systems(Update, (towers::damage_states, life::animate));
}

pub(super) fn painted(parts: Vec<Part>) -> Mesh {
    mesh::merge(
        parts
            .into_iter()
            .map(|(s, c, t)| (mesh::paint(mesh::build(s), c), t))
            .collect(),
    )
    .unwrap_or_else(|| Cuboid::new(0.01, 0.01, 0.01).into())
}

fn spawn_arena(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    let theme = theme::Theme::selected();
    info!("arena theme: {} (visual only)", theme.name());
    let kit = themes::build(theme);
    let mat = mats.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 1.0,
        ..default()
    });
    let sea_mat = mats.add(StandardMaterial {
        base_color: Color::WHITE,
        unlit: true,
        ..default()
    });
    let mut total = 0;
    let mut terrain = Vec::with_capacity(5);
    for (name, m) in [
        ("meadow", ground::meadow(theme)),
        ("BattleLine", ground::lane(theme)),
        ("cliffs", cliffs::mesh(theme)),
        ("sea", water::sea(theme)),
        ("foam", water::coast(theme)),
        ("offshore rocks", water::rocks(theme)),
        (
            "props",
            painted(if theme == theme::Theme::Forest {
                props::scatter()
            } else {
                kit.props
            }),
        ),
    ] {
        let triangles = mesh::triangles(&m);
        total += triangles;
        info!("arena {name}: {triangles} triangles");
        if name != "sea" && name != "foam" {
            terrain.push((m, Transform::default()));
            continue;
        }
        let mut entity = commands.spawn((
            Name::new(format!("arena {name}")),
            Mesh3d(meshes.add(m)),
            MeshMaterial3d(sea_mat.clone()),
            Transform::default(),
            NotShadowCaster,
        ));
        if name == "foam" && theme != theme::Theme::Tundra {
            entity.insert(life::Foam);
        }
    }
    if let Some(terrain) = mesh::merge(terrain) {
        commands.spawn((
            Name::new("arena merged terrain"),
            Mesh3d(meshes.add(terrain)),
            MeshMaterial3d(mat.clone()),
            Transform::default(),
        ));
    }
    if !kit.glow.is_empty() {
        let glow = painted(kit.glow);
        let triangles = mesh::triangles(&glow);
        total += triangles;
        info!("arena accents: {triangles} triangles");
        let glow_mat = mats.add(StandardMaterial {
            base_color: Color::WHITE,
            unlit: true,
            ..default()
        });
        commands.spawn((
            Name::new("arena themed accents"),
            Mesh3d(meshes.add(glow)),
            MeshMaterial3d(glow_mat),
            Transform::default(),
            NotShadowCaster,
        ));
    }
    for (own, x) in [(true, -0.9), (false, 16.9)] {
        let pieces = if theme == theme::Theme::Forest {
            towers::tower(own)
        } else {
            themes::structures::tower(theme, own)
        };
        for (piece, hide_below) in pieces {
            total += mesh::triangles(&piece);
            commands.spawn((
                Mesh3d(meshes.add(piece)),
                MeshMaterial3d(mat.clone()),
                Transform::from_xyz(x, 0.0, 0.0),
                towers::TowerPiece { own, hide_below },
            ));
        }
    }
    total += life::spawn(&mut commands, &mut meshes, &mut mats, &mat, theme);
    info!("arena total: {total} triangles");
    lighting::spawn(&mut commands, theme);
}
