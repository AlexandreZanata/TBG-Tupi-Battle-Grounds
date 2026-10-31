//! Simulation cues → animation clips, hit/heal tints, projectiles, and HP
//! bars that face the camera.

use bevy::prelude::*;
use tbg_core::Archetype;

use super::{AimTarget, UnitVisual, VisualAssets};
use crate::art::anim::{Animator, Clip};
use crate::art::FigureLibrary;
use crate::camera::ArenaCamera;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VisualCue {
    AttackStart,
    CargoDrop,
    Hit,
    Heal,
    Evolved,
    Death,
}

/// How long a dead unit stays on the field after its death clip.
pub const CORPSE_SECS: f32 = 0.9;
/// A unit stops walking only after this long without moving, so a frame
/// without a new view never flickers it into idle.
const STOP_SECS: f32 = 0.15;
/// Aim at the chest of a unit, not its feet.
const CHEST: f32 = 0.6;

pub fn play_cue(v: &mut UnitVisual, anim: &mut Animator, a: &VisualAssets, cue: VisualCue) {
    if v.dying {
        return;
    }
    match cue {
        VisualCue::CargoDrop => anim.gesture = 0.0,
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
    targets: Query<&Transform, With<UnitVisual>>,
) {
    let dt = time.delta_secs();
    for (root, mut v, mut anim, t) in &mut units {
        anim.aim_at = aim(&mut v, &targets, dt);
        if v.dying {
            v.dead_for += dt;
            commands.entity(v.bar).try_insert(Visibility::Hidden);
            commands.entity(v.ring).try_insert(Visibility::Hidden);
            for t_cube in &v.tiers {
                commands.entity(*t_cube).try_insert(Visibility::Hidden);
            }

            // A-1 faction deaths (D-47) for units without a kit; kits play
            // their own Death clip (D-51, D-53).
            if !v.has_kit() {
                let form = lib.get(v.form.0, v.form.1, v.form.2);
                let faction = tbg_core::content::UNITS[v.form.0 as usize].faction;
                let death_transform =
                    crate::art::death::evaluate(faction, v.dead_for, form.plan.scale, v.yaw);
                commands.entity(v.figure).try_insert(death_transform);
            }

            if v.dead_for > Clip::Death.duration() + CORPSE_SECS {
                commands.entity(root).try_despawn();
            }
            continue;
        }
        let form = lib.get(v.form.0, v.form.1, v.form.2);
        anim.native = form.native;
        anim.plan = form.plan;
        anim.form = (v.form.1, v.form.2);
        let growth = f32::from((v.fx >> 11) & 7) / 7.0;
        if v.kind() == tbg_core::UnitKind::Nester && growth < anim.growth {
            anim.gesture = 0.0;
        }
        anim.growth = growth;
        let moving = (t.translation.x - v.last_x).abs() > 0.0004;
        v.last_x = t.translation.x;
        v.still = if moving { 0.0 } else { v.still + dt };
        anim.set_base(if v.still < STOP_SECS {
            Clip::Walk
        } else {
            Clip::Idle
        });
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

/// The engaged target's world point, while the engagement lasts and the
/// target still exists.
fn aim(v: &mut UnitVisual, targets: &Query<&Transform, With<UnitVisual>>, dt: f32) -> Option<Vec3> {
    let (target, left) = v.aim?;
    let at = match target {
        AimTarget::Unit(e) => targets.get(e).ok().map(|t| t.translation + Vec3::Y * CHEST),
        AimTarget::Point(p) => Some(p),
    };
    v.aim = at.filter(|_| left > dt).map(|_| (target, left - dt));
    at
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
