//! Shot-driven motion: the Artillery barrel kicks and its gears notch on
//! every authoritative shot of that seat's Stronghold, timed to the render
//! delay so the kick lands with the projectile; mint parts turn on their own.

use super::parts::Motion;
use super::Tower;
use crate::sync::{LatestView, ViewBuffer};
use bevy::prelude::*;
use tbg_core::{Event, Target};

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

/// Upper bound on queued shots, so a hidden tab cannot pile them up.
const MAX_QUEUED: usize = 8;

/// Queue every `AttackStart` from a Stronghold onto that seat's live
/// tower. Events reach the scene one render delay after the view arrives,
/// so the kick waits that long too.
pub(super) fn queue_shots(
    view: Res<LatestView>,
    buffer: Res<ViewBuffer>,
    time: Res<Time>,
    mut towers: Query<&mut Tower>,
) {
    if !view.is_changed() {
        return;
    }
    let Some(v) = view.0.as_ref() else {
        return;
    };
    let at = time.elapsed_secs_f64() + f64::from(buffer.timeline.stats.delay_ms) / 1000.0;
    for ev in &v.events {
        let Event::AttackStart {
            from: Target::Stronghold(seat),
            ..
        } = ev
        else {
            continue;
        };
        let own = *seat == v.you;
        for mut tower in &mut towers {
            if tower.live && tower.own == own && tower.due.len() < MAX_QUEUED {
                tower.due.push(at);
            }
        }
    }
}

/// Advance every gun's clock and fire the shots that are due.
pub(super) fn fire(time: Res<Time>, mut towers: Query<&mut Tower>) {
    let now = time.elapsed_secs_f64();
    for mut tower in &mut towers {
        let Some(gun) = tower.gun else {
            tower.due.clear();
            continue;
        };
        tower.since += time.delta_secs();
        let queued = tower.due.len();
        tower.due.retain(|t| *t > now);
        let autofire = !tower.live && tower.since >= gun.interval;
        if tower.due.len() < queued || autofire {
            tower.since = 0.0;
            tower.shots = tower.shots.wrapping_add(1);
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
    match moving.motion {
        Motion::Kick(distance) => {
            let k = tower.gun.map_or(0.0, |g| kick(tower.since / g.recover));
            rest.with_translation(rest.translation + rest.rotation * Vec3::NEG_X * distance * k)
        }
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
