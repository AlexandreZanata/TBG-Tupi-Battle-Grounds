//! Kit attacks leave the real weapon: on Release a kit fires its own shot
//! from its muzzle toward the engaged target (with its muzzle flash and
//! backblast); on Strike a melee kit lands its impact where the blade meets
//! the target; heavy kits raise dust on every footfall.

use bevy::prelude::*;

use super::UnitVisual;
use crate::art::{
    anim::{Animator, ClipEvent},
    kits::Attack,
    rig::Node,
};
use crate::vfx::{
    impacts::{self, Area, Impact},
    shots, VfxAssets,
};

/// How far a melee target may stand from the attacker and still take the
/// impact (the blade may be behind it mid-combo, e.g. a shield bash).
const REACH: f32 = 2.6;

pub(super) fn release(
    mut commands: Commands,
    vfx: Res<VfxAssets>,
    mut units: Query<(&Animator, &mut UnitVisual, &Transform)>,
    globals: Query<&GlobalTransform>,
) {
    for (a, mut v, root) in &mut units {
        let Some(kit) = a.kit.filter(|_| !v.dying) else {
            continue;
        };
        let (tier, branch) = a.form;
        let area = Area {
            splash: v.splash,
            hold: v.hold,
        };
        if let Attack::TwinShot {
            hands,
            muzzle,
            shot,
            flash,
        } = kit.attack
        {
            let primary = a.attack_variant();
            if let Some(to) = v.secondary_shot.take() {
                twin_fire(
                    &mut commands,
                    &vfx,
                    &globals,
                    a,
                    (
                        hands[1 - primary],
                        muzzle,
                        shots::ShotKind::CrossBullet,
                        flash,
                    ),
                    to,
                );
            }
            if a.events.contains(&ClipEvent::Release) {
                let ahead = Quat::from_rotation_y(v.yaw) * Vec3::Z * v.range;
                let to = a.aim_at.unwrap_or(root.translation + ahead + Vec3::Y * 0.4);
                twin_fire(
                    &mut commands,
                    &vfx,
                    &globals,
                    a,
                    (hands[primary], muzzle, shot, flash),
                    to,
                );
            }
            continue;
        }
        match kit.attack {
            Attack::Shot {
                from,
                muzzle,
                shot,
                flash,
                ..
            } if a.events.contains(&ClipEvent::Release) => {
                let Ok(g) = globals.get(a.joints[from.index()]) else {
                    continue;
                };
                let (_, rotation, _) = g.to_scale_rotation_translation();
                let start = g.transform_point(muzzle(tier, branch));
                let ahead = Quat::from_rotation_y(v.yaw) * Vec3::Z * v.range;
                let to = a.aim_at.unwrap_or(root.translation + ahead + Vec3::Y * 0.4);
                shots::fire(&mut commands, &vfx, shot(tier, branch), start, to, area);
                if let Some((kind, back)) = flash(tier, branch) {
                    let dir = (rotation * Vec3::Z).normalize_or(Vec3::X);
                    let back = back.map(|b| g.transform_point(b));
                    impacts::muzzle(&mut commands, &vfx, kind, start, dir, back);
                }
            }
            Attack::Melee { from, tip, impact } if a.events.contains(&ClipEvent::Strike) => {
                let Ok(g) = globals.get(a.joints[from.index()]) else {
                    continue;
                };
                let blade = g.transform_point(tip(tier, branch));
                let at = a
                    .aim_at
                    .filter(|t| t.xz().distance(root.translation.xz()) < REACH)
                    .unwrap_or(blade);
                let kind = impact(tier, branch);
                impacts::land(&mut commands, &vfx, kind, at.with_y(0.3), area);
                if kind == Impact::Reap {
                    let to = at - root.translation;
                    let facing = to.x.atan2(to.z);
                    let center = root.translation.with_y(0.35);
                    impacts::sweep(&mut commands, &vfx, center, facing, v.splash.max(1.0), true);
                }
            }
            _ => {}
        }
    }
}

/// Heavy feet raise dust where they land.
pub(super) fn stomp(
    mut commands: Commands,
    vfx: Res<VfxAssets>,
    units: Query<&Animator, With<UnitVisual>>,
    globals: Query<&GlobalTransform>,
) {
    for a in &units {
        if !a.gait.stomp || !a.events.contains(&ClipEvent::Footstep) {
            continue;
        }
        let foot = a.stepping_foot();
        let Ok(g) = globals.get(a.joints[Node::Body(foot).index()]) else {
            continue;
        };
        let size = a.plan.scale / 1.6;
        impacts::stomp(&mut commands, &vfx, g.translation(), size);
    }
}

/// Both muzzles use the same world-joint path; the client never chooses a victim.
fn twin_fire(
    commands: &mut Commands,
    vfx: &VfxAssets,
    globals: &Query<&GlobalTransform>,
    a: &Animator,
    emission: (Node, Vec3, shots::ShotKind, impacts::Muzzle),
    to: Vec3,
) {
    let (from, muzzle, shot, flash) = emission;
    let Ok(g) = globals.get(a.joints[from.index()]) else {
        return;
    };
    let start = g.transform_point(muzzle);
    shots::fire(commands, vfx, shot, start, to, Area::default());
    impacts::muzzle(
        commands,
        vfx,
        flash,
        start,
        (to - start).normalize_or(Vec3::Z),
        None,
    );
}
