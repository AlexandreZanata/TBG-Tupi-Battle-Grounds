//! Water waves, waterfall flow and the 1v1 flame silhouette, with no gameplay state.

use super::{
    fog::{FogMask, FogMaterial, FogTexture},
    terrain, to_world,
};
use crate::art::mesh::{self, Shape};
use bevy::{light::NotShadowCaster, mesh::VertexAttributeValues, prelude::*};

#[derive(Component)]
pub struct WaterSurface;
#[derive(Component)]
struct Flame(f32);
#[derive(Component)]
pub struct WaterfallSurface;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, braziers.after(super::setup_material))
        .add_systems(Update, (waves, waterfall_flow, flicker));
}

/// Gateposts and roadside braziers sit beyond the walking ribbon.
pub fn fire_sites() -> Vec<Vec3> {
    let mut sites = Vec::new();
    for team in tbg_moba_core::Team::ALL {
        let c = tbg_moba_core::map::stronghold(team);
        let c = Vec2::new(c.x as f32, c.y as f32) / 1000.;
        for k in 0..8 {
            let a = k as f32 * std::f32::consts::TAU / 8.;
            let p = c + Vec2::new(a.cos(), a.sin()) * 19.5;
            if terrain::lane_distance(p) > 5. {
                sites.push(to_world(p.x, p.y, 3.1));
            }
        }
    }
    for lane in tbg_moba_core::Lane::ALL {
        for w in tbg_moba_core::map::lane_units(lane).windows(2) {
            let (a, b) = (terrain::v(w[0]), terrain::v(w[1]));
            let d = (b - a).normalize_or_zero();
            let p = (a + b) * 0.5 + Vec2::new(-d.y, d.x) * 7.;
            if terrain::river_distance(p) > 12. && terrain::height(p) < 1. {
                sites.push(to_world(p.x, p.y, terrain::height(p) + 2.1));
            }
        }
    }
    sites
}
fn braziers(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<FogMaterial>>,
    fog: Res<FogTexture>,
) {
    // The same nested cone silhouette and warm emissive palette as arena/life.rs.
    let mesh = mesh::merge(vec![
        (
            mesh::paint(
                mesh::build(Shape::Cone(0.21, 0.62)),
                Color::srgb(1., 0.32, 0.035),
            ),
            Transform::from_xyz(0., 0.22, 0.),
        ),
        (
            mesh::paint(
                mesh::build(Shape::Cone(0.12, 0.42)),
                Color::srgb(1., 0.82, 0.24),
            ),
            Transform::from_xyz(0., 0.18, 0.),
        ),
    ])
    .unwrap_or_else(|| Mesh::from(Cone::new(0.2, 0.6)));
    let mesh = meshes.add(mesh);
    let material = materials.add(FogMaterial {
        base: StandardMaterial {
            emissive: LinearRgba::rgb(3., 1.1, 0.12),
            unlit: true,
            ..default()
        },
        extension: FogMask {
            texture: fog.0.clone(),
        },
    });
    for (i, at) in fire_sites().into_iter().enumerate() {
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(at).with_scale(Vec3::splat(1.4)),
            Flame(i as f32 * 1.7),
            NotShadowCaster,
        ));
    }
}
fn waves(
    time: Res<Time>,
    surfaces: Query<&Mesh3d, With<WaterSurface>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for mesh in &surfaces {
        let Some(mut mesh) = meshes.get_mut(&mesh.0) else {
            continue;
        };
        let Some(VertexAttributeValues::Float32x3(positions)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
        else {
            continue;
        };
        for p in positions {
            // X/Z remain authored. Only the free river surface moves, not bridge decks.
            p[1] = terrain::WATER_LEVEL
                + (p[0] * 0.65 + p[2] * 0.32 + time.elapsed_secs() * 2.2).sin() * 0.055;
        }
    }
}
fn flicker(time: Res<Time>, mut q: Query<(&Flame, &mut Transform)>) {
    for (Flame(phase), mut t) in &mut q {
        let s = (time.elapsed_secs() * 7. + phase).sin();
        t.scale = Vec3::new(1.4 - s * 0.08, 1.4 + s * 0.22, 1.4 - s * 0.08);
        t.rotation = Quat::from_rotation_z(s * 0.08);
    }
}

fn waterfall_flow(
    time: Res<Time>,
    surfaces: Query<&Mesh3d, With<WaterfallSurface>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for handle in &surfaces {
        let Some(mut mesh) = meshes.get_mut(&handle.0) else {
            continue;
        };
        let colors = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            Some(VertexAttributeValues::Float32x3(p)) => p
                .iter()
                .map(|p| {
                    let k = ((p[1] * 2.3 + time.elapsed_secs() * 6.).sin() * 0.5 + 0.5) * 0.3;
                    Color::srgb(0.25 + k, 0.65 + k * 0.6, 0.76 + k * 0.5)
                        .to_linear()
                        .to_f32_array()
                })
                .collect::<Vec<_>>(),
            _ => continue,
        };
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    }
}
