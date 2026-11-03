//! Server-driven rolling casks: interpolate the two visible positions and roll along the ground.
use super::VfxAssets;
use crate::art::{
    mesh::{self, Shape::*},
    palette::*,
};
use crate::{app::Screen, link::Feed, scene::to_world, units::ground_y};
use bevy::prelude::*;
use tbg_moba_core::geom::Pos;

#[derive(Component)]
struct Cask(u32);
pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, draw.run_if(in_state(Screen::Match)))
        .add_systems(OnExit(Screen::Match), clear);
}
fn clear(mut c: Commands, all: Query<Entity, With<Cask>>) {
    for e in &all {
        c.entity(e).despawn();
    }
}
fn draw(
    mut c: Commands,
    feed: Res<Feed>,
    a: Res<VfxAssets>,
    time: Res<Time>,
    mut active: Query<(Entity, &Cask, &mut Transform)>,
) {
    let Some(world) = &feed.curr else { return };
    let pos = |p: Pos| Vec2::new(p.x as f32, p.y as f32) / 1000.0;
    let alpha = feed.alpha(time.elapsed_secs_f64());
    for (e, k, _) in &active {
        if !world.barrels.iter().any(|b| b.source == k.0) {
            c.entity(e).despawn();
        }
    }
    for b in &world.barrels {
        let prev = feed
            .prev
            .as_ref()
            .and_then(|w| w.barrels.iter().find(|v| v.source == b.source))
            .map_or(b.origin, |v| v.at);
        let p = pos(prev).lerp(pos(b.at), alpha);
        let d = (pos(b.at) - pos(prev)).normalize_or_zero();
        let yaw = d.y.atan2(d.x);
        let scale = 1.16 * 1.2;
        let radius = 0.205 * scale;
        let t = Transform::from_translation(to_world(p.x, p.y, ground_y(p) + radius))
            .with_scale(Vec3::splat(scale))
            .with_rotation(
                Quat::from_rotation_y(yaw)
                    * Quat::from_rotation_z(-pos(b.origin).distance(p) / radius),
            );
        if let Some((_, _, mut old)) = active.iter_mut().find(|(_, k, _)| k.0 == b.source) {
            *old = t;
        } else {
            c.spawn((
                Cask(b.source),
                Mesh3d(a.casks[0].clone()),
                MeshMaterial3d(a.painted.clone()),
                t,
            ));
        }
    }
}

const PROFILE: &[[f32; 2]] = &[
    [0.145, -0.23],
    [0.185, -0.18],
    [0.205, -0.08],
    [0.205, 0.08],
    [0.185, 0.18],
    [0.145, 0.23],
];
pub(super) fn meshes(meshes: &mut Assets<Mesh>) -> [Handle<Mesh>; 5] {
    crate::art::FORMS.map(|(t, b)| {
        let wood = if t > 0 && b == 0 { OAK } else { DARK_LEATHER };
        let turn = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
        let mut parts = vec![(
            mesh::paint(mesh::build(Lathe(PROFILE)), wood),
            Transform::from_rotation(turn),
        )];
        for z in [-0.17, 0.17] {
            parts.push((
                mesh::paint(mesh::build(Ring(0.017, 0.188, 360.0)), IRON),
                Transform::from_xyz(0.0, 0.0, z).with_rotation(turn),
            ));
        }
        for z in [-0.232, 0.232] {
            parts.push((
                mesh::paint(mesh::build(Cyl(0.145, 0.018)), OAK),
                Transform::from_xyz(0.0, 0.0, z).with_rotation(turn),
            ));
        }
        if t == 2 {
            parts.push((
                mesh::paint(mesh::build(Ball(0.035)), if b == 0 { AMBER } else { LEAF }),
                Transform::from_xyz(0.0, 0.20, 0.0),
            ));
        }
        meshes.add(mesh::merge(parts).unwrap_or_else(|| mesh::build(Ball(0.20))))
    })
}
