//! Diorama assembly: shared vertex-colored layers and authoritative Stronghold views.

use crate::art::mesh::{self, Shape};
use bevy::{light::NotShadowCaster, prelude::*};

mod camp;
mod cliffs;
mod clouds;
mod geometry;
mod ground;
mod life;
mod lighting;
mod maritime;
mod props;
mod rocks;
mod scene;
mod scenery;
mod strongholds;
pub(super) mod theme;
mod themes;
mod towers;
mod trees;
mod water;

pub(super) type Part = (Shape, Color, Transform);

pub use scene::ActiveArena;
pub(crate) use scene::ArenaScene;

pub fn plugin(app: &mut App) {
    scene::plugin(app);
    app.add_systems(
        Update,
        (towers::damage_states, life::animate, maritime::animate),
    );
    strongholds::plugin(app);
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

/// Builds one arena's static scene; every entity carries `ArenaScene`.
fn build(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    mats: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
    theme: theme::Theme,
) {
    info!("arena scene: {}", theme.name());
    let kit = themes::build(theme);
    let mat = mats.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 1.0,
        ..default()
    });
    let water_mat = mats.add(water::material::create(theme, images));
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
        ("foam", foam(theme)),
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
            ArenaScene,
            Name::new(format!("arena {name}")),
            Mesh3d(meshes.add(m)),
            MeshMaterial3d(if name == "sea" {
                water_mat.clone()
            } else {
                sea_mat.clone()
            }),
            Transform::default(),
            NotShadowCaster,
        ));
        if name == "foam" && theme != theme::Theme::Tundra {
            entity.insert(life::Foam);
        }
    }
    if let Some(terrain) = mesh::merge(terrain) {
        commands.spawn((
            ArenaScene,
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
            ArenaScene,
            Name::new("arena themed accents"),
            Mesh3d(meshes.add(glow)),
            MeshMaterial3d(glow_mat),
            Transform::default(),
            NotShadowCaster,
        ));
    }
    if theme == theme::Theme::Storm {
        let cloud = clouds::mesh();
        let triangles = mesh::triangles(&cloud);
        total += triangles;
        info!("arena squalls: {triangles} triangles");
        commands.spawn((
            ArenaScene,
            Name::new("arena distant squalls"),
            Mesh3d(meshes.add(cloud)),
            MeshMaterial3d(mats.add(StandardMaterial {
                base_color: Color::WHITE,
                unlit: true,
                alpha_mode: AlphaMode::Blend,
                cull_mode: None,
                ..default()
            })),
            Transform::default(),
            NotShadowCaster,
        ));
    }
    // The maritime scenery (isles, lighthouse, pier) belongs to the Forest's
    // seascape; the themed arenas have their own offshore silhouettes.
    if theme == theme::Theme::Forest {
        total += scenery::spawn(commands, meshes, &mat);
    }
    // The Strongholds are the kind-based D-46 towers (owner-approved P54
    // look) in every arena: `strongholds` builds them from the views.
    total += life::spawn(commands, meshes, mats, &mat, theme);
    info!("arena total: {total} triangles");
    lighting::spawn(commands, theme);
}

/// The coastal foam, plus the seascape's offshore foam in the Forest.
fn foam(theme: theme::Theme) -> Mesh {
    if theme != theme::Theme::Forest {
        return water::coast(theme);
    }
    mesh::merge(vec![
        (water::coast(theme), Transform::default()),
        (scenery::foam(), Transform::default()),
    ])
    .unwrap_or_else(|| Cuboid::new(0.01, 0.01, 0.01).into())
}
