//! Shot-driven motion: aimed guns turn onto their target (`aim.rs`), the
//! Artillery barrel kicks and its gears notch on every shot, defenders turn
//! and recoil, hatches swing open, gallery towers fire on their own; mint
//! parts turn on their own.

use super::aim::gun_rotation;
use super::parts::Motion;
use super::Tower;
use crate::visuals::AimTarget;
use bevy::prelude::*;

/// Recoil of an Artillery form and its authoritative firing interval.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Gun {
    /// How far the barrel slides back, in world units.
    pub kick: f32,
    /// Seconds from the shot until the barrel is home again.
    pub recover: f32,
    /// Seconds between shots (gallery autofire only; live shots come from events).
    pub interval: f32,
}

/// A moving piece and the pose it returns to.
#[derive(Component)]
pub(super) struct Moving {
    pub motion: Motion,
    pub rest: Transform,
}

/// Gallery towers (not live) fire on their own at the authoritative
/// interval (one second for the gunless kinds), at a dummy target
/// sweeping in front of them.
pub(super) fn fire(time: Res<Time>, mut towers: Query<(&mut Tower, &Transform)>) {
    for (mut tower, root) in &mut towers {
        tower.since += time.delta_secs();
        if tower.live {
            continue;
        }
        let interval = tower.gun.map_or(1.0, |g| g.interval);
        let clock = time.elapsed_secs();
        let ahead = if tower.own { 1.0 } else { -1.0 };
        let dummy = Vec3::new(
            ahead * (4.0 + 1.5 * (clock * 0.7).sin()),
            0.6,
            1.8 * (clock * 0.45).sin(),
        );
        tower.target = Some(AimTarget::Point(root.translation + dummy));
        tower.idle = 0.0;
        if tower.since >= interval && tower.pending.is_none() {
            tower.pending = Some(0.0);
        }
    }
}

/// Pose every moving piece from its tower's gun clock.
pub(super) fn animate(
    time: Res<Time>,
    towers: Query<&Tower>,
    mut parts: Query<(&Moving, &ChildOf, &mut Transform)>,
) {
    let clock = time.elapsed_secs_wrapped();
    for (moving, parent, mut transform) in &mut parts {
        if let Ok(tower) = towers.get(parent.parent()) {
            *transform = pose(moving, tower, clock);
        }
    }
}

fn pose(moving: &Moving, tower: &Tower, clock: f32) -> Transform {
    let rest = moving.rest;
    let aimed = gun_rotation(tower.own, tower.yaw, tower.pitch);
    match moving.motion {
        Motion::Kick(distance) => {
            let k = kick(tower.since / recover(tower));
            rest.with_rotation(aimed)
                .with_translation(rest.translation + aimed * Vec3::NEG_X * distance * k)
        }
        Motion::Mounted(distance) => {
            let k = kick(tower.since / recover(tower));
            let turn = Quat::from_rotation_y(tower.yaw);
            rest.with_rotation(aimed)
                .with_translation(turn * rest.translation + aimed * Vec3::NEG_X * distance * k)
        }
        Motion::Hatch(angle) => {
            rest.with_rotation(rest.rotation * Quat::from_rotation_x(-angle * hatch(tower.since)))
        }
        Motion::Swivel => {
            let turn = Quat::from_rotation_y(tower.yaw);
            rest.with_rotation(turn * rest.rotation)
                .with_translation(turn * rest.translation)
        }
        Motion::Aim => rest.with_rotation(aimed),
        Motion::Notch(angle) => {
            let settle = tower
                .gun
                .map_or(1.0, |g| (tower.since / (g.recover * 0.5)).min(1.0));
            // Only the remainder matters for a wheel; it also keeps f32 exact.
            let turns = (tower.shots % 64) as f32 - 1.0 + settle;
            let turns = if tower.shots == 0 { 0.0 } else { turns };
            rest.with_rotation(rest.rotation * Quat::from_rotation_y(angle * turns))
        }
        Motion::Turn(speed) => {
            rest.with_rotation(rest.rotation * Quat::from_rotation_y(clock * speed))
        }
    }
}

/// Seconds a gun takes to settle after a shot: the Artillery's own, else a
/// light weapon's.
fn recover(tower: &Tower) -> f32 {
    tower.gun.map_or(0.3, |g| g.recover)
}

/// How far a hatch stands open (0..=1) `since` seconds after a shot:
/// snaps open, holds while the shot leaves, swings shut.
pub(super) fn hatch(since: f32) -> f32 {
    let open = (since / 0.06).clamp(0.0, 1.0);
    let shut = ((since - 0.3) / 0.35).clamp(0.0, 1.0);
    open * (1.0 - shut * shut * (3.0 - 2.0 * shut))
}

/// Recoil offset (0..=1) at `x` = time since the shot / recovery time: a
/// sharp kick in the first 15 %, then an eased return.
pub(super) fn kick(x: f32) -> f32 {
    const SNAP: f32 = 0.15;
    if !(0.0..1.0).contains(&x) {
        0.0
    } else if x < SNAP {
        x / SNAP
    } else {
        (1.0 - (x - SNAP) / (1.0 - SNAP)).powi(2)
    }
}
