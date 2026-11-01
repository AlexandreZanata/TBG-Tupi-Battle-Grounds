//! Nozzle-bound spectral flame motion and the unchanged authoritative ground segment.

use super::{spawn, Motion, VfxAssets};
use bevy::{light::NotShadowCaster, prelude::*};

/// Cosmetic lifetime matches the original flame; particles never decide hits.
const LIFE: f32 = 0.20;

#[derive(Component)]
pub(crate) struct FlameJet {
    nozzle: Option<(Entity, Vec3)>,
    fallback: Vec3,
    end: Vec3,
    piece: u8,
    age: f32,
}

pub fn profane_flame(
    commands: &mut Commands,
    assets: &VfxAssets,
    start: f32,
    end: f32,
    nozzle: Option<(Entity, Vec3)>,
) {
    // The strip remains exactly [start, end], including friendly fire and both seats.
    spawn(
        commands,
        &assets.chip,
        &assets.green,
        Vec3::new((start + end) * 0.5, 0.025, 0.0),
        Motion::Segment {
            size: Vec3::new((end - start).abs(), 0.018, 0.12),
        },
        0.0,
        0.28,
    );
    let count = if crate::config::low_motion() { 4 } else { 8 };
    for piece in 0..count {
        let (mesh, material) = match piece {
            0 => (&assets.spike, &assets.green),
            1 => (&assets.spike, &assets.white),
            2 | 5 | 6 => (&assets.puff, &assets.gold),
            7 => (&assets.puff, &assets.soot),
            _ => (&assets.puff, &assets.green),
        };
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_scale(Vec3::ZERO),
            FlameJet {
                nozzle,
                fallback: Vec3::new(start, 0.55, 0.0),
                end: Vec3::new(end, 0.55, 0.0),
                piece,
                age: 0.0,
            },
            NotShadowCaster,
        ));
    }
}

pub(crate) fn animate(
    mut commands: Commands,
    time: Res<Time>,
    sockets: Query<&GlobalTransform, Without<FlameJet>>,
    mut jets: Query<(Entity, &mut FlameJet, &mut Transform, &mut GlobalTransform)>,
) {
    for (entity, mut jet, mut local, mut global) in &mut jets {
        jet.age += time.delta_secs();
        if jet.age >= LIFE {
            commands.entity(entity).despawn();
            continue;
        }
        let origin = jet
            .nozzle
            .and_then(|(entity, point)| {
                sockets
                    .get(entity)
                    .ok()
                    .map(|socket| socket.transform_point(point))
            })
            .filter(|point| point.is_finite())
            .unwrap_or(jet.fallback);
        // Keep the last valid world origin if the authoritative emitter disappears.
        jet.fallback = origin;
        *local = flame_pose(origin, jet.end, jet.age / LIFE, jet.piece);
        // These independent roots follow already-propagated sockets without a one-frame lag.
        *global = GlobalTransform::from(*local);
    }
}

fn flame_pose(from: Vec3, to: Vec3, k: f32, piece: u8) -> Transform {
    let delta = to - from;
    let length = delta.length();
    let rotation = Quat::from_rotation_arc(Vec3::Y, delta.normalize_or(Vec3::X));
    let base = Transform::from_translation(from).with_rotation(rotation);
    let fade = (1.0 - k).sqrt();
    let flutter = 0.85 + 0.15 * (k * 31.0).sin();
    match piece {
        0 => base.with_scale(Vec3::new(0.30 * flutter, length, 0.26 * fade)),
        1 => base.with_scale(Vec3::new(0.12 * flutter, length * 0.82, 0.10 * fade)),
        7 => base
            .with_translation(from.lerp(to, 0.18 + k * 0.16) + Vec3::Y * 0.12 * k)
            .with_scale(Vec3::splat(0.035 * fade)),
        _ => {
            let phase = f32::from(piece - 2) * 0.21;
            let along = (phase + k * 0.9).fract();
            let width = if piece == 6 { 0.022 } else { 0.14 } * fade;
            // Tongues rise in world space; the jet rotation would flatten them
            // along the BattleLine and hide their flame silhouette behind bodies.
            base.with_translation(
                from.lerp(to, along) + Vec3::Y * (0.18 + 0.25 * (phase + k * 21.0).sin().abs()),
            )
            .with_rotation(Quat::from_rotation_z(-delta.x.signum() * 0.35))
            .with_scale(Vec3::new(width, width * 3.2, width))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pyre_jet_reaches_the_event_endpoint_for_both_seats() {
        // P70: the nozzle is cosmetic; the authored event segment must not gain range.
        for direction in [-1.0, 1.0] {
            let from = Vec3::new(direction * 0.6, 0.9, 0.1);
            let end = Vec3::new(direction * 2.8, 0.55, 0.0);
            for k in [0.0, 0.25, 0.5, 0.9] {
                let pose = flame_pose(from, end, k, 0);
                assert!(pose.transform_point(Vec3::Y).distance(end) < 1e-5);
                assert_eq!(pose.translation, from);
            }
        }
    }
}
