//! Simulation cues → animation clips, hit/heal tints, projectiles, and HP
//! bars that face the camera.

use bevy::prelude::*;
use tbg_core::Archetype;

use super::{UnitVisual, VisualAssets};
use crate::art::anim::{Animator, Clip};
use crate::art::FigureLibrary;
use crate::camera::ArenaCamera;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VisualCue {
    AttackStart,
    Hit,
    Heal,
    Evolved,
    Death,
}

/// How long a dead unit stays on the field after its death clip.
const CORPSE_SECS: f32 = 0.9;

pub fn play_cue(v: &mut UnitVisual, anim: &mut Animator, a: &VisualAssets, cue: VisualCue) {
    if v.dying {
        return;
    }
    match cue {
        VisualCue::AttackStart => anim.play(match v.archetype {
            Archetype::Ranged => Clip::AttackRanged,
            Archetype::Support | Archetype::Control => Clip::Cast,
            Archetype::Tank | Archetype::Dps => Clip::AttackMelee,
        }),
        VisualCue::Hit => {
            v.tint = Some((a.flash.clone(), 0.08, false));
            anim.play_if_free(Clip::Hit);
        }
        VisualCue::Heal => v.tint = Some((a.heal.clone(), 0.15, false)),
        VisualCue::Evolved => anim.play(Clip::Evolve),
        VisualCue::Death => {
            v.dying = true;
            anim.play(Clip::Death);
        }
    }
}

pub(super) fn animate(
    mut commands: Commands,
    time: Res<Time>,
    lib: Res<FigureLibrary>,
    mut units: Query<(Entity, &mut UnitVisual, &mut Animator, &Transform)>,
) {
    let dt = time.delta_secs();
    for (root, mut v, mut anim, t) in &mut units {
        if v.dying {
            v.dead_for += dt;
            if v.dead_for > Clip::Death.duration() + CORPSE_SECS {
                commands.entity(root).try_despawn();
            }
            continue;
        }
        let moving = (t.translation.x - v.last_x).abs() > 0.0004;
        v.last_x = t.translation.x;
        anim.set_base(if moving { Clip::Walk } else { Clip::Idle });
        // Swap materials only on tint start and end, not every frame.
        let joints = v.joints;
        let tint = v.tint.take();
        if let Some((mat, left, applied)) = tint {
            if !applied {
                for j in joints {
                    commands.entity(j).try_insert(MeshMaterial3d(mat.clone()));
                }
            }
            if left - dt <= 0.0 {
                for j in joints {
                    commands
                        .entity(j)
                        .try_insert(MeshMaterial3d(lib.material.clone()));
                }
            } else {
                v.tint = Some((mat, left - dt, true));
            }
        }
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
