//! Rootcaller motion: sways like a tree at rest, shuffles on root feet,
//! and casts by raising both claws (0–0.14 s), driving them into the
//! ground at the Release (0.22 s), channelling with a tremble, and pulling
//! them out (to 0.6 s). Spawn sprouts from the ground.

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::model::plan;
use crate::art::{
    anim::{Clip, Easing::*, Pose},
    kits::{
        tools::{self, hold, play, window, Track},
        Moment,
    },
    rig::{Joint::*, Proportions},
};

const Z: [f32; 3] = [0.0; 3];

#[rustfmt::skip]
const CAST: &[Track] = &[
    (UpperArmL, &[(0.0, [-10.0, 0.0, 8.0], Z, CubicInOut), (0.233, [-165.0, 0.0, 20.0], Z, CubicIn), (0.367, [-35.0, 0.0, 10.0], Z, Linear), (0.75, [-38.0, 0.0, 10.0], Z, CubicInOut), (1.0, [-10.0, 0.0, 8.0], Z, CubicInOut)]),
    (UpperArmR, &[(0.0, [-10.0, 0.0, -8.0], Z, CubicInOut), (0.233, [-165.0, 0.0, -20.0], Z, CubicIn), (0.367, [-35.0, 0.0, -10.0], Z, Linear), (0.75, [-38.0, 0.0, -10.0], Z, CubicInOut), (1.0, [-10.0, 0.0, -8.0], Z, CubicInOut)]),
    (ForearmL, &[(0.0, [-15.0, 0.0, 0.0], Z, CubicInOut), (0.233, [-20.0, 0.0, 0.0], Z, CubicIn), (0.367, [-50.0, 0.0, 0.0], Z, Linear), (0.75, [-50.0, 0.0, 0.0], Z, CubicInOut), (1.0, [-15.0, 0.0, 0.0], Z, CubicInOut)]),
    (ForearmR, &[(0.0, [-15.0, 0.0, 0.0], Z, CubicInOut), (0.233, [-20.0, 0.0, 0.0], Z, CubicIn), (0.367, [-50.0, 0.0, 0.0], Z, Linear), (0.75, [-50.0, 0.0, 0.0], Z, CubicInOut), (1.0, [-15.0, 0.0, 0.0], Z, CubicInOut)]),
    (HandL, &[(0.0, Z, Z, CubicInOut), (0.233, [-20.0, 0.0, 0.0], Z, CubicIn), (0.367, [30.0, 0.0, 0.0], Z, Linear), (0.75, [30.0, 0.0, 0.0], Z, CubicInOut), (1.0, Z, Z, CubicInOut)]),
    (HandR, &[(0.0, Z, Z, CubicInOut), (0.233, [-20.0, 0.0, 0.0], Z, CubicIn), (0.367, [30.0, 0.0, 0.0], Z, Linear), (0.75, [30.0, 0.0, 0.0], Z, CubicInOut), (1.0, Z, Z, CubicInOut)]),
    (Spine, &[(0.0, [4.0, 0.0, 0.0], Z, CubicInOut), (0.233, [-12.0, 0.0, 0.0], Z, CubicIn), (0.367, [38.0, 0.0, 0.0], Z, Linear), (0.75, [36.0, 0.0, 0.0], Z, CubicInOut), (1.0, [4.0, 0.0, 0.0], Z, CubicInOut)]),
    (Chest, &[(0.0, Z, Z, CubicInOut), (0.233, [-6.0, 0.0, 0.0], Z, CubicIn), (0.367, [12.0, 0.0, 0.0], Z, Linear), (0.75, [12.0, 0.0, 0.0], Z, CubicInOut), (1.0, Z, Z, CubicInOut)]),
    (Head, &[(0.0, Z, Z, CubicInOut), (0.233, [-15.0, 0.0, 0.0], Z, CubicIn), (0.367, [-30.0, 0.0, 0.0], Z, Linear), (0.75, [-30.0, 0.0, 0.0], Z, CubicInOut), (1.0, Z, Z, CubicInOut)]),
    (Hips, &[(0.0, Z, Z, CubicInOut), (0.233, Z, [0.0, 0.04, -0.02], CubicIn), (0.367, Z, [0.0, -0.09, 0.04], Linear), (0.75, Z, [0.0, -0.09, 0.04], CubicInOut), (1.0, Z, Z, CubicInOut)]),
];

pub fn pose(m: &Moment, pose: &mut Pose) {
    let p = plan(m.tier);
    match m.clip {
        Clip::Idle => idle(m, pose, &p),
        Clip::Walk => walk(m, pose),
        Clip::Cast => cast(m, pose, &p),
        Clip::Spawn => sprout(m, pose),
        Clip::Evolve => bloom(m, pose, &p),
        _ => {}
    }
}

fn idle(m: &Moment, pose: &mut Pose, p: &Proportions) {
    let engaged = m.aim.map_or(0.0, |a| a.weight);
    let sway = tools::wave(m.clock, 0.55, 0.7);
    tools::add(pose, Spine, [3.0 + 6.0 * engaged, 0.0, 3.5 * sway]);
    tools::add(
        pose,
        Head,
        [
            -4.0 * engaged,
            8.0 * tools::wave(m.clock, 0.3, 2.0),
            -4.0 * sway,
        ],
    );
    // Claws flex; engaged, they lift, ready to strike the ground.
    let flex = (m.clock * 2.2).sin();
    for (upper, fore, hand, s) in [
        (UpperArmL, ForearmL, HandL, 1.0f32),
        (UpperArmR, ForearmR, HandR, -1.0),
    ] {
        tools::set(pose, upper, [-12.0 - 30.0 * engaged, 0.0, 8.0 * s]);
        tools::set(pose, fore, [-15.0 - 40.0 * engaged, 0.0, 0.0]);
        tools::set(pose, hand, [8.0 * flex - 10.0 * engaged, 0.0, 0.0]);
    }
    tools::face(pose, m.aim, Vec3::ZERO);
    tools::shift(pose, Hips, [0.0, -0.025 * engaged, 0.0]);
    tools::plant(pose, p, tools::stance(p, 1.15 + 0.2 * engaged, 0.02));
}

fn walk(m: &Moment, pose: &mut Pose) {
    let phase = m.walk.unwrap_or(0.0) * TAU;
    tools::add(pose, Spine, [6.0, 0.0, 5.0 * phase.sin()]);
    tools::add(pose, Head, [-4.0, 0.0, -4.0 * phase.sin()]);
    tools::add(pose, ForearmL, [-25.0, 0.0, 0.0]);
    tools::add(pose, ForearmR, [-25.0, 0.0, 0.0]);
}

fn cast(m: &Moment, pose: &mut Pose, p: &Proportions) {
    let k = (m.t / Clip::Cast.duration()).clamp(0.0, 1.0);
    play(pose, CAST, k);
    let tremble = (m.t * 70.0).sin() * hold(k, [0.37, 0.42, 0.7, 0.75]);
    tools::add(pose, Chest, [0.0, 0.0, 2.5 * tremble]);
    tools::add(pose, Head, [2.0 * tremble, 0.0, 0.0]);
    tools::face(pose, m.aim, Vec3::ZERO);
    tools::plant(pose, p, tools::stance(p, 1.3, 0.06));
}

/// Evolve (0.9 s): sinks its claws, then springs up with its branches
/// thrown open to the sky.
fn bloom(m: &Moment, pose: &mut Pose, p: &Proportions) {
    let k = (m.t / 0.9).clamp(0.0, 1.0);
    let crouch = hold(k, [0.0, 0.25, 0.3, 0.45]);
    let open = hold(k, [0.35, 0.55, 0.75, 1.0]);
    pose[Root.index()] = (Vec3::ZERO, Vec3::Y * 0.06 * open);
    tools::add(pose, Spine, [35.0 * crouch - 12.0 * open, 0.0, 0.0]);
    tools::add(pose, Head, [-25.0 * open, 0.0, 0.0]);
    for (upper, fore, s) in [(UpperArmL, ForearmL, 1.0f32), (UpperArmR, ForearmR, -1.0)] {
        tools::set(
            pose,
            upper,
            [-30.0 * crouch - 150.0 * open, 0.0, s * (10.0 + 35.0 * open)],
        );
        tools::set(pose, fore, [-30.0 * crouch - 10.0 * open, 0.0, 0.0]);
    }
    tools::shift(pose, Hips, [0.0, -0.1 * crouch, 0.0]);
    tools::plant(pose, p, tools::stance(p, 1.3, 0.0));
}

/// Grows out of the ground, branches first, then stands.
fn sprout(m: &Moment, pose: &mut Pose) {
    let k = (m.t / 0.55).clamp(0.0, 1.0);
    let under = (1.0 - k / 0.75).max(0.0);
    pose[Root.index()].1 = Vec3::new(0.0, -0.9 * under * under, 0.0);
    let reach = 1.0 - window(k, 0.55, 1.0);
    tools::set(pose, UpperArmL, [-170.0 * reach, 0.0, 15.0 * reach]);
    tools::set(pose, UpperArmR, [-170.0 * reach, 0.0, -15.0 * reach]);
    tools::add(pose, Head, [-20.0 * reach, 0.0, 0.0]);
}
