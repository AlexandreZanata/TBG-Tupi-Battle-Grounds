//! Short-lived combat feedback: projectiles, hit sparks, death bursts, move markers.

use bevy::prelude::*;
use tbg_moba_core::types::Side;
use tbg_moba_core::UnitKind;

use super::ground_y;
use crate::palette;
use crate::scene::to_world;

/// An attack or heal left `from` towards `to` (map units).
#[derive(Message, Clone, Copy)]
pub struct Shot {
    pub target: u32,
    pub from: Vec2,
    pub to: Vec2,
    pub kind: UnitKind,
    pub side: Side,
}

/// A unit died at `at`.
#[derive(Message, Clone, Copy)]
pub struct Burst {
    pub at: Vec2,
    pub kind: UnitKind,
    pub side: Side,
}

/// The player ordered a move to `at`.
#[derive(Message, Clone, Copy)]
pub struct Marker {
    pub at: Vec2,
}

#[derive(Component)]
struct Flight {
    target: u32,
    from: Vec3,
    to: Vec3,
    t: f32,
    duration: f32,
    arc: f32,
}

#[derive(Component)]
struct Fade {
    t: f32,
    duration: f32,
    grow: f32,
    size: f32,
    rise: f32,
}

#[derive(Resource)]
struct FxAssets {
    ball: Handle<Mesh>,
    ring: Handle<Mesh>,
    /// Blue, red, neutral, dust, gold.
    glow: [Handle<StandardMaterial>; 5],
}

pub fn plugin(app: &mut App) {
    app.add_message::<Shot>()
        .add_message::<Burst>()
        .add_message::<Marker>()
        .add_systems(Startup, setup)
        .add_systems(Update, (shots, bursts, markers, fly, fade));
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut glow = |c: Color, e: LinearRgba| {
        materials.add(StandardMaterial {
            base_color: c,
            emissive: e,
            unlit: true,
            ..default()
        })
    };
    commands.insert_resource(FxAssets {
        ball: meshes.add(
            Sphere::new(1.0)
                .mesh()
                .ico(1)
                .unwrap_or_else(|_| Sphere::new(1.0).into()),
        ),
        ring: meshes.add(
            Annulus::new(0.8, 1.0)
                .mesh()
                .resolution(24)
                .build()
                .rotated_by(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ),
        glow: [
            glow(palette::BLUE_GLOW, LinearRgba::rgb(0.4, 1.6, 4.0)),
            glow(palette::RED_GLOW, LinearRgba::rgb(4.0, 0.9, 0.4)),
            glow(palette::NEUTRAL, LinearRgba::rgb(3.0, 1.4, 0.3)),
            glow(Color::srgb(0.75, 0.72, 0.66), LinearRgba::BLACK),
            glow(palette::GOLD, LinearRgba::rgb(3.0, 2.2, 0.5)),
        ],
    });
}

fn side_index(side: Side) -> usize {
    match side {
        Side::Team(t) => t.index(),
        Side::Neutral => 2,
    }
}

fn lifted(p: Vec2, y: f32) -> Vec3 {
    to_world(p.x, p.y, ground_y(p) + y)
}

fn shots(mut commands: Commands, mut reader: MessageReader<Shot>, assets: Res<FxAssets>) {
    for shot in reader.read() {
        let ranged = shot.from.distance(shot.to) > 3.5;
        let material = side_index(shot.side);
        if !ranged {
            // Melee: a spark on the target.
            commands.spawn((
                Mesh3d(assets.ball.clone()),
                MeshMaterial3d(assets.glow[material].clone()),
                Transform::from_translation(lifted(shot.to, 1.0)).with_scale(Vec3::splat(0.25)),
                Fade {
                    t: 0.0,
                    duration: 0.28,
                    grow: 1.6,
                    size: 0.25,
                    rise: 0.0,
                },
            ));
            continue;
        }
        let (start, size, speed) = if shot.kind.structure() {
            (8.0, 0.45, 40.0)
        } else {
            (1.3, 0.2, 32.0)
        };
        let (from, to) = (lifted(shot.from, start), lifted(shot.to, 1.0));
        commands.spawn((
            Mesh3d(assets.ball.clone()),
            MeshMaterial3d(assets.glow[material].clone()),
            Transform::from_translation(from).with_scale(Vec3::splat(size)),
            Flight {
                target: shot.target,
                from,
                to,
                t: 0.0,
                duration: (from.distance(to) / speed).clamp(0.08, 0.4),
                arc: if shot.kind.structure() { 0.0 } else { 1.2 },
            },
        ));
    }
}

fn bursts(mut commands: Commands, mut reader: MessageReader<Burst>, assets: Res<FxAssets>) {
    for burst in reader.read() {
        let big = burst.kind.structure();
        let (count, size) = if big { (10, 1.6) } else { (4, 0.45) };
        for k in 0..count {
            let a = k as f32 / count as f32 * std::f32::consts::TAU;
            let spread = if big { 3.0 } else { 0.6 };
            let at = lifted(burst.at, if big { 3.0 } else { 0.6 })
                + Vec3::new(a.cos(), 0.0, a.sin()) * spread;
            let material = if big || matches!(burst.kind, UnitKind::Commander(_)) {
                side_index(burst.side)
            } else {
                4
            };
            commands.spawn((
                Mesh3d(assets.ball.clone()),
                MeshMaterial3d(assets.glow[material].clone()),
                Transform::from_translation(at).with_scale(Vec3::splat(size)),
                Fade {
                    t: 0.0,
                    duration: if big { 1.1 } else { 0.5 },
                    grow: 2.2,
                    size,
                    rise: if big { 2.5 } else { 0.8 },
                },
            ));
        }
    }
}

fn markers(mut commands: Commands, mut reader: MessageReader<Marker>, assets: Res<FxAssets>) {
    for marker in reader.read() {
        commands.spawn((
            Mesh3d(assets.ring.clone()),
            MeshMaterial3d(assets.glow[4].clone()),
            Transform::from_translation(lifted(marker.at, 0.15)).with_scale(Vec3::splat(1.6)),
            Fade {
                t: 0.0,
                duration: 0.5,
                grow: 0.2,
                size: 1.6,
                rise: 0.0,
            },
        ));
    }
}

fn fly(
    mut commands: Commands,
    time: Res<Time>,
    mut flights: Query<(Entity, &mut Flight, &mut Transform)>,
    feed: Res<crate::link::Feed>,
) {
    for (entity, mut f, mut transform) in &mut flights {
        f.t += time.delta_secs() / f.duration;
        if f.t >= 1.0
            || !feed
                .curr
                .as_ref()
                .is_some_and(|w| w.units.iter().any(|u| u.id == f.target))
        {
            commands.entity(entity).despawn();
            continue;
        }
        transform.translation = f.from.lerp(f.to, f.t) + Vec3::Y * f.arc * 4.0 * f.t * (1.0 - f.t);
    }
}

fn fade(
    mut commands: Commands,
    time: Res<Time>,
    mut fades: Query<(Entity, &mut Fade, &mut Transform)>,
) {
    for (entity, mut f, mut transform) in &mut fades {
        f.t += time.delta_secs() / f.duration;
        if f.t >= 1.0
            || !feed
                .curr
                .as_ref()
                .is_some_and(|w| w.units.iter().any(|u| u.id == f.target))
        {
            commands.entity(entity).despawn();
            continue;
        }
        // Grow quickly, then shrink away (no per-entity transparency needed).
        let k = (f.t * std::f32::consts::PI).sin();
        transform.scale = Vec3::splat(f.size * (1.0 + f.grow * f.t) * k.max(0.05));
        transform.translation.y += f.rise * time.delta_secs();
    }
}
