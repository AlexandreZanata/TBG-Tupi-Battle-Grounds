//! Rolling-cask presentation on the shared server snapshot timeline; no local collision decisions.
use super::VfxAssets;
use crate::art::{
    mesh::{self, Shape::*},
    palette::*,
};
use bevy::prelude::*;
use tbg_core::{barrel::Barrel, Event, MatchView};

#[derive(Resource, Default)]
pub(crate) struct Entities(Vec<((u32, u32), Entity)>);
impl Entities {
    pub(crate) fn len(&self) -> usize {
        self.0.len()
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
pub(crate) fn clear(c: &mut Commands, entities: &mut Entities) {
    for (_, e) in entities.0.drain(..) {
        c.entity(e).despawn();
    }
}

/// Rolling entities belong to their battle; leaving it must not retain world props.
pub(crate) fn clear_system(mut c: Commands, mut entities: ResMut<Entities>) {
    clear(&mut c, &mut entities);
}

pub(crate) fn place(
    c: &mut Commands,
    a: &VfxAssets,
    entities: &mut Entities,
    from: &MatchView,
    to: &MatchView,
    alpha: f32,
) {
    let key = |b: &Barrel| (b.source, b.launched);
    let mut keys = Vec::new();
    for b in &from.barrels {
        let next = to.barrels.iter().find(|v| key(v) == key(b));
        let end = next
            .map(|v| v.x)
            .or_else(|| {
                to.events.iter().find_map(|e| match e {
                    Event::BarrelBurst { source, center, .. } if *source == b.source => {
                        Some(*center)
                    }
                    _ => None,
                })
            })
            .unwrap_or(
                b.x + b.direction
                    * b.remaining
                        .min(b.speed * (to.tick.saturating_sub(from.tick).min(100) as i32)),
            );
        if next.is_some() || alpha < 1.0 {
            draw(
                c,
                a,
                entities,
                b,
                b.x as f32 + (end as f32 - b.x as f32) * alpha,
                from.you,
                lateral(b, to, end, b.x as f32 + (end as f32 - b.x as f32) * alpha),
                &mut keys,
            );
        }
    }
    for b in &to.barrels {
        if from.barrels.iter().any(|v| key(v) == key(b)) {
            continue;
        }
        let tick = from.tick as f32 + to.tick.saturating_sub(from.tick) as f32 * alpha;
        if tick + 0.001 < b.launched as f32 {
            continue;
        }
        let span = to.tick.saturating_sub(b.launched).max(1) as f32;
        let k = ((tick - b.launched as f32) / span).clamp(0.0, 1.0);
        let origin = from.units.iter().find(|u| u.id == b.source).map_or(
            b.x.saturating_sub(b.direction.signum().saturating_mul(b.speed.max(0))),
            |u| u.x,
        );
        draw(
            c,
            a,
            entities,
            b,
            origin as f32 + (b.x as f32 - origin as f32) * k,
            from.you,
            lateral(b, to, b.x, origin as f32 + (b.x as f32 - origin as f32) * k),
            &mut keys,
        );
    }
    entities.0.retain(|(key, e)| {
        if keys.contains(key) {
            true
        } else {
            c.entity(*e).despawn();
            false
        }
    });
}
fn lateral(b: &Barrel, view: &MatchView, end: i32, x: f32) -> f32 {
    let destination = view
        .events
        .iter()
        .find_map(|e| match e {
            Event::BarrelBurst {
                source,
                center,
                center_z,
                ..
            } if *source == b.source => Some((*center, *center_z)),
            _ => None,
        })
        .or_else(|| {
            view.units
                .iter()
                .find(|u| u.id == b.target)
                .map(|u| (u.x, i32::from(u.lane_z)))
        });
    let (tx, tz) = destination.unwrap_or((end, b.origin_z));
    let span = tx - b.origin_x;
    let k = if span == 0 {
        1.0
    } else {
        ((x - b.origin_x as f32) / span as f32).clamp(0.0, 1.0)
    };
    (b.origin_z as f32 + (tz - b.origin_z) as f32 * k) / 1000.0
}
#[allow(clippy::too_many_arguments)] // Shared snapshot projection keeps roll and lateral interpolation together.
fn draw(
    c: &mut Commands,
    a: &VfxAssets,
    entities: &mut Entities,
    b: &Barrel,
    x: f32,
    you: tbg_core::Seat,
    z: f32,
    keys: &mut Vec<(u32, u32)>,
) {
    let key = (b.source, b.launched);
    keys.push(key);
    let slot = if b.tier == 0 {
        0
    } else if b.branch == 0 {
        usize::from(b.tier.min(2))
    } else {
        usize::from(b.tier.min(2)) + 2
    };
    let scale = 1.16 * 1.2 * (1.0 + 0.08 * f32::from(b.tier.min(2)));
    let x = crate::sync::world_tile(x / 1000.0, you);
    let z = z * if you == tbg_core::Seat::A { 1.0 } else { -1.0 };
    let transform = Transform::from_xyz(x, 0.205 * scale, z)
        .with_scale(Vec3::splat(scale))
        .with_rotation(Quat::from_rotation_z(-x / (0.205 * scale)));
    if let Some((_, e)) = entities.0.iter().find(|(k, _)| *k == key) {
        c.entity(*e).insert(transform);
    } else {
        let e = c
            .spawn((
                Mesh3d(a.casks[slot].clone()),
                MeshMaterial3d(a.painted.clone()),
                transform,
            ))
            .id();
        entities.0.push((key, e));
    }
}
