//! Short scale/offset animations, hit tints, projectiles, and HP bars that
//! face the camera. No animation system: a cue is a timer per unit.

use bevy::prelude::*;

use super::{UnitVisual, VisualAssets};
use crate::camera::ArenaCamera;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VisualCue {
    Spawn,
    AttackStart,
    Hit,
    Heal,
    Evolved,
    Death,
}

impl VisualCue {
    fn duration(self) -> f32 {
        match self {
            VisualCue::Spawn => 0.3,
            VisualCue::AttackStart => 0.25,
            VisualCue::Evolved => 0.4,
            VisualCue::Death => 0.35,
            VisualCue::Hit | VisualCue::Heal => 0.1,
        }
    }
}

/// Start a cue. Hit and Heal only tint; the others animate the body.
pub fn play_cue(v: &mut UnitVisual, a: &VisualAssets, cue: VisualCue) {
    if v.dying {
        return;
    }
    match cue {
        VisualCue::Hit => v.tint = Some((a.flash.clone(), 0.08)),
        VisualCue::Heal => v.tint = Some((a.heal.clone(), 0.15)),
        VisualCue::Death => {
            v.dying = true;
            v.anim = Some((cue, 0.0));
        }
        _ => v.anim = Some((cue, 0.0)),
    }
}

pub(super) fn animate(
    mut commands: Commands,
    time: Res<Time>,
    a: Res<VisualAssets>,
    mut units: Query<(Entity, &mut UnitVisual)>,
) {
    let dt = time.delta_secs();
    for (root, mut v) in &mut units {
        let mut t = Transform::from_xyz(0.0, v.body_y, 0.0);
        if let Some((cue, elapsed)) = v.anim {
            let k = ((elapsed + dt) / cue.duration()).min(1.0);
            let facing = if v.own { 1.0 } else { -1.0 };
            match cue {
                VisualCue::Spawn => t.scale = Vec3::splat(k.max(0.05)),
                VisualCue::AttackStart => {
                    t.translation.x += facing * 0.25 * (k * std::f32::consts::PI).sin()
                }
                VisualCue::Evolved => {
                    t.scale = Vec3::splat(1.0 + 0.3 * (k * std::f32::consts::PI).sin())
                }
                VisualCue::Death => t.scale = Vec3::splat((1.0 - k).max(0.01)),
                VisualCue::Hit | VisualCue::Heal => {}
            }
            v.anim = (k < 1.0).then_some((cue, elapsed + dt));
            if cue == VisualCue::Death && k >= 1.0 {
                commands.entity(root).despawn();
                continue;
            }
        }
        let material = match &mut v.tint {
            Some((m, left)) if *left > 0.0 => {
                *left -= dt;
                m.clone()
            }
            _ => a.faction[v.faction].clone(),
        };
        commands
            .entity(v.body)
            .insert((t, MeshMaterial3d(material)));
    }
}

/// HP bars always face the camera.
pub(super) fn billboards(
    camera: Single<&Transform, With<ArenaCamera>>,
    units: Query<&UnitVisual>,
    mut transforms: Query<&mut Transform, Without<ArenaCamera>>,
) {
    for v in &units {
        if let Ok(mut t) = transforms.get_mut(v.bar) {
            t.rotation = camera.rotation;
        }
    }
}

#[derive(Component)]
pub(super) struct Projectile {
    from: Vec3,
    to: Vec3,
    t: f32,
}

pub fn spawn_projectile(commands: &mut Commands, a: &VisualAssets, from: Vec3, to: Vec3) {
    commands.spawn((
        Mesh3d(a.projectile.clone()),
        MeshMaterial3d(a.projectile_mat.clone()),
        Transform::from_translation(from),
        Projectile { from, to, t: 0.0 },
    ));
}

/// Projectiles arc from attacker to target in 0.25 s.
pub(super) fn fly(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut Projectile, &mut Transform)>,
) {
    for (e, mut p, mut t) in &mut q {
        p.t += time.delta_secs() / 0.25;
        if p.t >= 1.0 {
            commands.entity(e).despawn();
            continue;
        }
        let arc = 0.6 * (p.t * std::f32::consts::PI).sin();
        t.translation = p.from.lerp(p.to, p.t) + Vec3::Y * arc;
    }
}
