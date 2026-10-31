//! Real-joint ranged releases, counted cosmetic variants, melee contact and heavy footfalls.
use super::{AimTarget, UnitVisual};
use crate::art::{
    anim::{Animator, ClipEvent},
    kits::{Attack, Flash, Kit},
    rig::Node,
};
use crate::vfx::{
    impacts::{self, Area, Impact},
    shots, VfxAssets,
};
use bevy::prelude::*;
/// Cosmetic lunge bound; impacts still reach the authoritative selected target.
pub(super) const REACH: f32 = 3.5;
type Emission = (Node, Vec3, shots::ShotKind, Flash);
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
        if twin(&mut commands, &vfx, &globals, a, &mut v, root, kit) {
            continue;
        }
        let area = Area {
            splash: v.splash,
            hold: v.hold,
        };
        if a.events.contains(&ClipEvent::Release) {
            if let Some(emission) = shot(a, &v, &kit.attack) {
                projectile(
                    &mut commands,
                    &vfx,
                    &globals,
                    a,
                    emission,
                    destination(a, &v, root),
                    area,
                );
            }
        }
        if a.events.contains(&ClipEvent::Strike) {
            melee(&mut commands, &vfx, &globals, a, &v, root, &kit.attack);
        }
    }
}
fn shot(a: &Animator, v: &UnitVisual, attack: &Attack) -> Option<Emission> {
    let (t, b) = a.form;
    match *attack {
        Attack::Shot {
            from,
            muzzle,
            shot,
            flash,
            ..
        } => Some((from, muzzle(t, b), shot(t, b), flash(t, b))),
        Attack::CountedShot {
            from,
            muzzle,
            normal,
            powered,
            every,
            ..
        } => {
            let unit = matches!(v.aim, Some((AimTarget::Unit(_), _)));
            let ready = unit && a.counter.saturating_add(1) >= every(t, b).max(1);
            Some((
                from,
                muzzle(t, b),
                if ready { powered } else { normal },
                None,
            ))
        }
        _ => None,
    }
}
fn destination(a: &Animator, v: &UnitVisual, root: &Transform) -> Vec3 {
    let ahead = Quat::from_rotation_y(v.yaw) * Vec3::Z * v.range;
    a.aim_at.unwrap_or(root.translation + ahead + Vec3::Y * 0.4)
}
fn twin(
    commands: &mut Commands,
    vfx: &VfxAssets,
    globals: &Query<&GlobalTransform>,
    a: &Animator,
    v: &mut UnitVisual,
    root: &Transform,
    kit: &Kit,
) -> bool {
    let Attack::TwinShot {
        hands,
        muzzle,
        shot,
        flash,
    } = kit.attack
    else {
        return false;
    };
    let primary = a.attack_variant();
    let flash = Some((flash, None));
    if let Some(to) = v.secondary_shot.take() {
        projectile(
            commands,
            vfx,
            globals,
            a,
            (
                hands[1 - primary],
                muzzle,
                shots::ShotKind::CrossBullet,
                flash,
            ),
            to,
            Area::default(),
        );
    }
    if a.events.contains(&ClipEvent::Release) {
        projectile(
            commands,
            vfx,
            globals,
            a,
            (hands[primary], muzzle, shot, flash),
            destination(a, v, root),
            Area::default(),
        );
    }
    true
}
fn projectile(
    commands: &mut Commands,
    vfx: &VfxAssets,
    globals: &Query<&GlobalTransform>,
    a: &Animator,
    emission: Emission,
    to: Vec3,
    area: Area,
) {
    let (from, muzzle, shot, flash) = emission;
    let Ok(g) = globals.get(a.joints[from.index()]) else {
        return;
    };
    let start = g.transform_point(muzzle);
    shots::fire(commands, vfx, shot, start, to, area);
    if let Some((kind, back)) = flash {
        let (_, rotation, _) = g.to_scale_rotation_translation();
        let dir = (rotation * Vec3::Z).normalize_or(Vec3::X);
        impacts::muzzle(
            commands,
            vfx,
            kind,
            start,
            dir,
            back.map(|p| g.transform_point(p)),
        );
    }
}
fn melee(
    commands: &mut Commands,
    vfx: &VfxAssets,
    globals: &Query<&GlobalTransform>,
    a: &Animator,
    v: &UnitVisual,
    root: &Transform,
    attack: &Attack,
) {
    let Attack::Melee { from, tip, impact } = *attack else {
        return;
    };
    let (t, b) = a.form;
    let Ok(g) = globals.get(a.joints[from.index()]) else {
        return;
    };
    let blade = g.transform_point(tip(t, b));
    let at = a.aim_at.unwrap_or(blade);
    let kind = impact(t, b);
    let area = Area {
        splash: v.splash,
        hold: v.hold,
    };
    if v.kind() == tbg_core::UnitKind::Longaxe {
        shots::fire(
            commands,
            vfx,
            shots::ShotKind::SlashWave,
            root.translation.with_y(0.05),
            at.with_y(0.05),
            area,
        );
        if b == 1 && t > 0 {
            impacts::sweep(
                commands,
                vfx,
                root.translation.with_y(0.15),
                v.yaw,
                v.range,
                true,
            );
        }
    }
    impacts::land(commands, vfx, kind, at.with_y(0.3), area);
    if kind == Impact::Reap {
        let to = at - root.translation;
        impacts::sweep(
            commands,
            vfx,
            root.translation.with_y(0.35),
            to.x.atan2(to.z),
            v.splash.max(1.0),
            true,
        );
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
