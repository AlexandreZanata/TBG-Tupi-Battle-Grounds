//! Volthar's hammer performance: a held guard, an overhead throw of lightning and a
//! two-handed charge on the server-counted fourth bolt. The right wrist always
//! solves onto the hammer grip, so the weapon never floats or crosses the body.

use super::{hammer, model};
use crate::art::{
    anim::{Clip, Pose},
    ik,
    kits::{tools, Moment},
    rig::{Joint::*, Node::*, Proportions, Socket, Spring},
};
use bevy::prelude::*;
use std::f32::consts::TAU;

/// Hammer grip and shaft angles in chest space: (origin, pitch, yaw, roll).
#[derive(Clone, Copy)]
struct Hold {
    at: Vec3,
    pitch: f32,
    yaw: f32,
    roll: f32,
    /// 0 = right hand only, 1 = the left hand also on the support grip.
    both: f32,
}

// The head turns edge-on to the face (roll) so no striking face or spike crosses it.
const GUARD: Hold = Hold {
    at: Vec3::new(-0.50, -0.12, 0.10),
    pitch: -72.0,
    yaw: -18.0,
    roll: 90.0,
    both: 0.0,
};
const LOADED: Hold = Hold {
    at: Vec3::new(-0.30, 0.52, -0.06),
    pitch: -158.0,
    yaw: -6.0,
    roll: 0.0,
    both: 0.0,
};
const THROWN: Hold = Hold {
    at: Vec3::new(-0.18, 0.36, 0.42),
    pitch: -6.0,
    yaw: 8.0,
    roll: 0.0,
    both: 0.0,
};

fn mix(a: Hold, b: Hold, k: f32) -> Hold {
    Hold {
        at: a.at.lerp(b.at, k),
        pitch: a.pitch + (b.pitch - a.pitch) * k,
        yaw: a.yaw + (b.yaw - a.yaw) * k,
        roll: a.roll + (b.roll - a.roll) * k,
        both: a.both + (b.both - a.both) * k,
    }
}

pub(super) fn pose(m: &Moment, p: &mut Pose) {
    p.fill((Vec3::ZERO, Vec3::ZERO));
    let body = model::plan(m.tier);
    let (hold, spin) = match m.clip {
        Clip::Idle => (idle(m, p), 0.8),
        Clip::Walk => (walk(m, p), 0.8),
        Clip::AttackMelee | Clip::AttackRanged | Clip::Cast => attack(m, p),
        Clip::Hit => (hit(m, p), 1.2),
        Clip::Death => (death(m, p, &body), 0.0),
        Clip::Spawn => (spawn(m, p), 2.0),
        Clip::Evolve => (evolve(m, p), 3.0),
        Clip::Victory => (victory(m, p), 1.6),
    };
    if !matches!(m.clip, Clip::Walk | Clip::Death) {
        tools::plant(p, &body, tools::stance(&body, 1.2, 0.07));
    }
    let hold = aim(m, p, &body, hold);
    grip(p, &body, hold);
    sparks(m, p, spin);
}

fn idle(m: &Moment, p: &mut Pose) -> Hold {
    let breath = (m.clock * 1.3).sin();
    tools::set(p, Spine, [1.0 + 1.5 * breath, 0.0, 0.0]);
    tools::set(p, Chest, [0.0, 3.0 * tools::wave(m.clock, 0.35, 1.0), 0.0]);
    tools::set(p, Head, [-3.0 - breath, 4.0 * (m.clock * 0.45).sin(), 0.0]);
    p[Secondary(Spring::Back).index()].0.x = 3.0 + 2.0 * breath;
    Hold {
        at: GUARD.at + Vec3::Y * 0.012 * breath,
        ..GUARD
    }
}

fn walk(m: &Moment, p: &mut Pose) -> Hold {
    let step = (m.walk.unwrap_or(m.t / 0.7) * TAU).sin();
    tools::set(p, Chest, [5.0, 3.0 * step, 1.5 * step]);
    tools::set(p, Head, [-4.0, -2.0 * step, -step]);
    p[Secondary(Spring::Back).index()].0.x = 10.0 + 3.0 * step.abs();
    // The free arm swings opposite the hammer side.
    tools::set(p, UpperArmL, [-14.0 * step, 0.0, 10.0]);
    tools::set(p, ForearmL, [-18.0, 0.0, 0.0]);
    Hold {
        at: GUARD.at + Vec3::new(0.0, 0.02 * step.abs(), 0.03 * step),
        pitch: GUARD.pitch + 4.0 * step,
        ..GUARD
    }
}

/// Release is fixed at 0.18 s (ART-DIRECTION §4); the charged fourth bolt and
/// the wrath branch only raise higher and add the second hand.
fn attack(m: &Moment, p: &mut Pose) -> (Hold, f32) {
    let charged = m.counter >= 3;
    let heavy = m.branch == 1 && m.tier > 0;
    let load = tools::hold(m.t, [0.0, 0.10, 0.12, 0.18]);
    let strike = tools::hold(m.t, [0.12, 0.18, 0.28, 0.72]);
    let power = if charged { 1.0 } else { 0.0 } + if heavy { 0.5 } else { 0.0 };
    tools::set(
        p,
        Chest,
        [
            -10.0 * load * (1.0 + 0.4 * power) + 14.0 * strike * (1.0 + 0.3 * power),
            12.0 * load - 6.0 * strike,
            0.0,
        ],
    );
    tools::set(p, Head, [-6.0 * load + 4.0 * strike, -6.0 * load, 0.0]);
    tools::shift(
        p,
        Hips,
        [0.0, -0.03 * strike * (1.0 + power), 0.025 * strike],
    );
    p[Secondary(Spring::Back).index()].0.x = 6.0 + 10.0 * strike;
    let mut loaded = LOADED;
    let mut thrown = THROWN;
    if charged || heavy {
        loaded.at.y += 0.12 * power.min(1.0);
        loaded.both = 1.0;
        thrown.both = 1.0;
        thrown.at.x += 0.04;
    } else {
        tools::set(p, UpperArmL, [-30.0 * strike, 0.0, 18.0 + 10.0 * load]);
        tools::set(p, ForearmL, [-25.0, 0.0, 0.0]);
    }
    let hold = mix(mix(GUARD, loaded, load), thrown, strike);
    (hold, 1.0 + 3.0 * load * (0.3 + power))
}

fn hit(m: &Moment, p: &mut Pose) -> Hold {
    let recoil = tools::hold(m.t, [0.0, 0.05, 0.09, 0.25]);
    tools::set(p, Chest, [-7.0 * recoil, 0.0, -3.0 * recoil]);
    tools::set(p, Head, [-6.0 * recoil, 0.0, 2.0 * recoil]);
    tools::shift(p, Hips, [0.0, -0.02 * recoil, -0.03 * recoil]);
    Hold {
        at: GUARD.at + Vec3::new(0.0, -0.03, -0.04) * recoil,
        pitch: GUARD.pitch + 10.0 * recoil,
        ..GUARD
    }
}

fn death(m: &Moment, p: &mut Pose, body: &Proportions) -> Hold {
    let kneel = tools::window(m.t, 0.08, 0.65);
    let bow = tools::window(m.t, 0.30, 0.95);
    tools::shift(p, Hips, [0.0, -body.leg * 0.45 * kneel, 0.0]);
    tools::set(p, ThighL, [-70.0 * kneel, 0.0, 0.0]);
    tools::set(p, ShinL, [80.0 * kneel, 0.0, 0.0]);
    tools::set(p, ThighR, [-20.0 * kneel, 0.0, 0.0]);
    tools::set(p, ShinR, [95.0 * kneel, 0.0, 0.0]);
    tools::set(p, Spine, [16.0 * bow, 0.0, 4.0 * bow]);
    tools::set(p, Head, [14.0 * bow, 0.0, 0.0]);
    tools::set(p, UpperArmL, [-20.0 * bow, 0.0, 12.0]);
    p[Secondary(Spring::Back).index()].0.x = -6.0 * bow;
    // The hammer head lowers onto the ground before him, still in his grip.
    mix(
        GUARD,
        Hold {
            at: Vec3::new(-0.36, 0.02, 0.36),
            pitch: 62.0,
            yaw: 0.0,
            roll: 0.0,
            both: 0.0,
        },
        kneel,
    )
}

fn spawn(m: &Moment, p: &mut Pose) -> Hold {
    let rise = tools::window(m.t, 0.0, 0.5);
    tools::shift(p, Root, [0.0, -0.22 * (1.0 - rise), 0.0]);
    tools::set(p, Head, [-10.0 * (1.0 - rise), 0.0, 0.0]);
    mix(evolve_pose(1.0), GUARD, rise)
}

fn evolve_pose(display: f32) -> Hold {
    mix(
        GUARD,
        Hold {
            at: Vec3::new(-0.20, 0.70, 0.10),
            pitch: -92.0,
            yaw: 0.0,
            roll: 0.0,
            both: 1.0,
        },
        display,
    )
}

fn evolve(m: &Moment, p: &mut Pose) -> Hold {
    let gather = tools::hold(m.t, [0.0, 0.18, 0.28, 0.45]);
    let display = tools::hold(m.t, [0.30, 0.48, 0.75, 1.0]);
    tools::shift(p, Hips, [0.0, -0.05 * gather, 0.0]);
    tools::set(p, Chest, [6.0 * gather - 8.0 * display, 0.0, 0.0]);
    tools::set(p, Head, [-12.0 * display, 0.0, 0.0]);
    p[Secondary(Spring::Back).index()].0.x = 14.0 * display;
    evolve_pose(display)
}

/// A storm salute: the hammer pumps overhead twice per beat, the cape billows.
fn victory(m: &Moment, p: &mut Pose) -> Hold {
    let phase = (m.t / 0.9).rem_euclid(1.0);
    let pump = tools::hold(phase, [0.0, 0.25, 0.45, 0.9]);
    let beat = (phase * TAU).sin();
    tools::shift(p, Root, [0.0, 0.02 * pump, 0.0]);
    tools::set(p, Chest, [-6.0 * pump, 4.0 * beat, 0.0]);
    tools::set(p, Head, [-14.0 * pump, 0.0, 3.0 * beat]);
    p[Secondary(Spring::Back).index()].0.x = 12.0 + 6.0 * pump;
    let mut raised = evolve_pose(1.0);
    raised.both = 0.0;
    raised.at.y += 0.05 * pump;
    raised.at.x -= 0.06;
    tools::set(p, UpperArmL, [-150.0 * pump, 0.0, 25.0]);
    tools::set(p, ForearmL, [-20.0, 0.0, 0.0]);
    raised
}

/// Point the bolt line at the authoritative target during the throw only.
fn aim(m: &Moment, p: &mut Pose, body: &Proportions, mut hold: Hold) -> Hold {
    let throwing = matches!(m.clip, Clip::AttackMelee | Clip::AttackRanged | Clip::Cast);
    let Some(a) = m.aim else {
        return hold;
    };
    if throwing {
        let chest = tools::chest(p, body);
        let target = chest.inverse().transform_point3(a.target) - hold.at;
        let wanted = -target.y.atan2(target.xz().length()).to_degrees();
        let strike = tools::hold(m.t, [0.12, 0.18, 0.28, 0.72]);
        hold.pitch += (wanted.clamp(-25.0, 25.0) - THROWN.pitch) * a.weight * strike;
    }
    if !matches!(m.clip, Clip::Death) {
        tools::face(p, m.aim, Vec3::ZERO);
    }
    hold
}

fn frame(hold: Hold) -> Quat {
    Quat::from_rotation_y(hold.yaw.to_radians())
        * Quat::from_rotation_x(hold.pitch.to_radians())
        * Quat::from_rotation_z(hold.roll.to_radians())
}

fn grip(p: &mut Pose, body: &Proportions, hold: Hold) {
    let rotation = frame(hold);
    p[Prop(Socket::WeaponR).index()] = (ik::degrees(rotation), hold.at);
    tools::reach(
        p,
        false,
        hold.at,
        Some(rotation),
        (body, Vec3::new(-1.0, -0.5, 0.2)),
    );
    if hold.both > 0.5 {
        tools::reach(
            p,
            true,
            hold.at + rotation * hammer::SUPPORT,
            Some(rotation),
            (body, Vec3::new(1.0, -0.5, 0.2)),
        );
    }
}

fn sparks(m: &Moment, p: &mut Pose, spin: f32) {
    let jitter = tools::wave(m.clock, 7.0, 2.0);
    p[Prop(Socket::OffhandL).index()] = (
        Vec3::new(0.0, 0.0, (m.clock * 360.0 * spin).rem_euclid(360.0)),
        Vec3::new(0.0, 0.0, 0.015 * jitter),
    );
}

/// Restore the hammer hand after a crossfade, using the blended hammer frame.
pub(crate) fn finish_blend(pose: &mut Pose, body: &Proportions) {
    let (rotation, origin) = pose[Prop(Socket::WeaponR).index()];
    tools::reach(
        pose,
        false,
        origin,
        Some(ik::quat(rotation)),
        (body, Vec3::new(-1.0, -0.5, 0.2)),
    );
}
