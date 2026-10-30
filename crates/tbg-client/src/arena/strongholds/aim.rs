//! Towers aim and fire their own projectiles. A shot order (the render-
//! delayed `AttackStart` of a Stronghold) turns the tower's weapon onto the
//! target — the Artillery carriage and barrel, the Sentinel ballista, or a
//! defender (`gunners.rs`) — yaw toward it, elevation along the launch
//! tangent of its own shot, then fires from the real muzzle. A tower whose
//! defender fell looses a crossbow bolt from its top. The weapon keeps
//! tracking its target between shots and settles back when it has none.

use bevy::prelude::*;
use tbg_core::StrongholdKind;

use super::parts::Form;
use super::{Look, Tower};
use crate::art::kits::aim::{pitch, tangent};
use crate::vfx::{
    impacts::{self, Area, Muzzle},
    shots::{self, ShotKind},
    VfxAssets,
};
use crate::visuals::{AimTarget, TowerOrders, UnitVisual};

/// Radians per second a turret slews.
const SLEW: f32 = 7.0;
/// A shot waits at most this long for the turret to come onto its target.
const SETTLE: f32 = 0.15;
/// Seconds without a new order before the turret settles back.
const FORGET: f32 = 2.5;
/// Aim at the chest of a unit, not its feet.
const CHEST: f32 = 0.6;

/// The gun of a form: the aimed piece's origin on the own (+X) tower, its
/// rest elevation, and its muzzle in the piece's space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Turret {
    pub pivot: Vec3,
    pub rest_pitch: f32,
    pub muzzle: Vec3,
    pub shot: ShotKind,
    pub flash: Muzzle,
    /// Carried by a defender who walks round the tower's axis to face it.
    pub orbit: bool,
    /// Largest yaw either side of facing the enemy (a gun port's arc).
    pub arc: f32,
}

/// The aimed gun of a look, if it has one.
pub(crate) fn turret(look: Look) -> Option<Turret> {
    let form = Form::of(look.tier, look.branch);
    match look.kind {
        StrongholdKind::Artillery => Some(super::artillery::turret(form)),
        StrongholdKind::Bastion if form == Form::B2 && look.damage < 1 => {
            Some(super::bastion::turret())
        }
        kind => super::gunners::turret(kind, look.damage),
    }
}

/// Where a gunless tower looses its bolt from (own tower space).
fn top(kind: StrongholdKind) -> Vec3 {
    let y = match kind {
        StrongholdKind::Classic => 2.05,
        StrongholdKind::Command => 2.2,
        StrongholdKind::Treasury => 1.5,
        _ => 1.45,
    };
    Vec3::new(0.25, y, 0.0)
}

/// Mirror an own-tower point onto this tower's side.
fn side(own: bool, p: Vec3) -> Vec3 {
    if own {
        p
    } else {
        Vec3::new(-p.x, p.y, p.z)
    }
}

/// World rotation of the aimed gun for `yaw` and absolute elevation `up`.
pub(super) fn gun_rotation(own: bool, yaw: f32, up: f32) -> Quat {
    let face = if own {
        Quat::IDENTITY
    } else {
        Quat::from_rotation_y(std::f32::consts::PI)
    };
    Quat::from_rotation_y(yaw) * face * Quat::from_rotation_z(up)
}

/// Where the weapon's pivot stands for `yaw`: a carried weapon walks
/// round the tower's axis with its defender.
pub(super) fn pivot_at(own: bool, t: &Turret, root: Vec3, yaw: f32) -> Vec3 {
    let at = side(own, t.pivot);
    root + if t.orbit {
        Quat::from_rotation_y(yaw) * at
    } else {
        at
    }
}

/// Yaw and elevation that put the weapon of a tower at `root` onto `target`.
pub(super) fn solve(own: bool, t: &Turret, root: Vec3, target: Vec3) -> (f32, f32) {
    // A defender stands on the ray from the axis, so aim from the axis.
    let from = if t.orbit {
        root + Vec3::Y * t.pivot.y
    } else {
        pivot_at(own, t, root, 0.0)
    };
    let d = target - from;
    let s = if own { 1.0 } else { -1.0 };
    let yaw = (-d.z * s).atan2(d.x * s).clamp(-t.arc, t.arc);
    let muzzle = pivot_at(own, t, root, yaw) + gun_rotation(own, yaw, t.rest_pitch) * t.muzzle;
    let up = pitch(tangent(muzzle, target, shots::arc(t.shot), 0.0));
    (yaw, up.clamp(-0.45, 1.2))
}

/// Take this frame's Stronghold shot orders.
pub(super) fn orders(mut orders: ResMut<TowerOrders>, mut towers: Query<&mut Tower>) {
    for (own, target) in orders.0.drain(..) {
        for mut tower in &mut towers {
            if tower.live && tower.own == own {
                tower.target = Some(target);
                tower.idle = 0.0;
                tower.pending = Some(0.0);
            }
        }
    }
}

/// Slew every turret toward its target and fire pending shots once on it.
pub(super) fn track(
    mut commands: Commands,
    time: Res<Time>,
    vfx: Res<VfxAssets>,
    mut towers: Query<(&mut Tower, &Transform)>,
    units: Query<&Transform, With<UnitVisual>>,
) {
    let dt = time.delta_secs();
    for (mut tower, root) in &mut towers {
        tower.idle += dt;
        let target = tower.target.and_then(|t| match t {
            AimTarget::Unit(e) => units.get(e).ok().map(|u| u.translation + Vec3::Y * CHEST),
            AimTarget::Point(p) => Some(p),
        });
        if target.is_none() || tower.idle > FORGET {
            tower.target = None;
        }
        let own = tower.own;
        let goal = match (tower.turret, target) {
            (Some(t), Some(at)) => Some(solve(own, &t, root.translation, at)),
            _ => None,
        };
        let (yaw, up) = goal.unwrap_or((0.0, tower.turret.map_or(0.0, |t| t.rest_pitch)));
        let step = SLEW * dt;
        let yaw_error = yaw - tower.yaw;
        tower.yaw += yaw_error.clamp(-step, step);
        tower.pitch += (up - tower.pitch).clamp(-step, step);
        let Some(waited) = tower.pending else {
            continue;
        };
        if yaw_error.abs() > 0.08 && waited < SETTLE {
            tower.pending = Some(waited + dt);
            continue;
        }
        tower.pending = None;
        tower.since = 0.0;
        tower.shots = tower.shots.wrapping_add(1);
        if let Some(at) = target {
            fire(&mut commands, &vfx, &tower, root.translation, at);
        }
    }
}

fn fire(commands: &mut Commands, vfx: &VfxAssets, tower: &Tower, root: Vec3, at: Vec3) {
    let area = Area::default();
    let Some(t) = tower.turret else {
        let from = root + side(tower.own, top(tower.kind()));
        impacts::muzzle(
            commands,
            vfx,
            Muzzle::Bolt,
            from,
            (at - from).normalize_or_zero(),
            None,
        );
        shots::fire(commands, vfx, ShotKind::Crossbow, from, at, area);
        return;
    };
    let turn = gun_rotation(tower.own, tower.yaw, tower.pitch);
    let from = pivot_at(tower.own, &t, root, tower.yaw) + turn * t.muzzle;
    impacts::muzzle(commands, vfx, t.flash, from, turn * Vec3::X, None);
    shots::fire(commands, vfx, t.shot, from, at, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aimed_barrels_point_along_their_shots_on_both_sides() {
        let t = Turret {
            pivot: Vec3::new(0.05, 1.2, 0.0),
            rest_pitch: 0.12,
            muzzle: Vec3::new(0.66, 0.0, 0.0),
            shot: ShotKind::Cannon,
            flash: Muzzle::Cannon,
            orbit: false,
            arc: std::f32::consts::PI,
        };
        for (own, root) in [
            (true, Vec3::new(-0.9, 0.0, 0.0)),
            (false, Vec3::new(16.9, 0.0, 0.0)),
        ] {
            for target in [
                Vec3::new(4.0, 0.6, 0.8),
                Vec3::new(12.0, 0.6, -0.8),
                Vec3::new(8.0, 0.6, 0.0),
            ] {
                let (yaw, up) = solve(own, &t, root, target);
                let turn = gun_rotation(own, yaw, up);
                let muzzle = pivot_at(own, &t, root, yaw) + turn * t.muzzle;
                let barrel = turn * Vec3::X;
                let flight = tangent(muzzle, target, shots::arc(t.shot), 0.0);
                assert!(
                    barrel
                        .xz()
                        .normalize()
                        .angle_to(flight.xz().normalize())
                        .abs()
                        < 0.03,
                    "own {own} target {target:?}: barrel {barrel:?} vs flight {flight:?}"
                );
                assert!(
                    (pitch(barrel) - pitch(flight)).abs() < 0.05,
                    "elevation off for {target:?}"
                );
            }
        }
    }
}
