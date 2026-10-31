//! Bone Brute motion: a hunched, breathing idle; a lumbering walk; attacks
//! that alternate per swing — overhead two-hand slam and side sweep
//! (base, Ossified), shield bash and chop (Bone Wall), a double chop
//! (Butcher), and a wide reaping sweep (Reaper). Strike lands at 0.30 s of
//! a 0.62 s clip (k = 0.484). Spawn claws up out of the ground.

use bevy::prelude::*;

use super::model::{plan, Form};
use crate::art::{
    anim::{Clip, Easing::*, Pose},
    kits::{
        tools::{self, hold, play, snap, window, Track},
        Moment,
    },
    rig::{Joint::*, Proportions},
};

const Z: [f32; 3] = [0.0; 3];
/// The kit's attack length (mod.rs) and the Strike as a fraction of it.
pub const ATTACK: f32 = 0.62;

#[rustfmt::skip]
const SLAM: &[Track] = &[
    (UpperArmR, &[(0.0, [-40.0, 0.0, -14.0], Z, CubicInOut), (0.32, [-195.0, 0.0, -10.0], Z, CubicIn), (0.484, [-50.0, 0.0, -8.0], Z, Linear), (0.645, [-45.0, 0.0, -8.0], Z, CubicInOut), (1.0, [-40.0, 0.0, -14.0], Z, CubicInOut)]),
    (ForearmR, &[(0.0, [-55.0, 0.0, 0.0], Z, CubicInOut), (0.32, [-60.0, 0.0, 0.0], Z, CubicIn), (0.484, [-10.0, 0.0, 0.0], Z, Linear), (0.645, [-12.0, 0.0, 0.0], Z, CubicInOut), (1.0, [-55.0, 0.0, 0.0], Z, CubicInOut)]),
    (HandR, &[(0.0, [20.0, 0.0, 0.0], Z, CubicInOut), (0.32, [-20.0, 0.0, 0.0], Z, CubicIn), (0.484, [40.0, 0.0, 0.0], Z, Linear), (1.0, [20.0, 0.0, 0.0], Z, CubicInOut)]),
    (UpperArmL, &[(0.0, [-8.0, 0.0, 12.0], Z, CubicInOut), (0.32, [-185.0, 0.0, 8.0], Z, CubicIn), (0.484, [-55.0, 0.0, 6.0], Z, Linear), (0.645, [-50.0, 0.0, 6.0], Z, CubicInOut), (1.0, [-8.0, 0.0, 12.0], Z, CubicInOut)]),
    (ForearmL, &[(0.0, [-22.0, 0.0, 0.0], Z, CubicInOut), (0.32, [-50.0, 0.0, 0.0], Z, CubicIn), (0.484, [-15.0, 0.0, 0.0], Z, CubicInOut), (1.0, [-22.0, 0.0, 0.0], Z, CubicInOut)]),
    (Spine, &[(0.0, [14.0, 0.0, 0.0], Z, CubicInOut), (0.32, [-8.0, 0.0, 0.0], Z, CubicIn), (0.484, [36.0, 0.0, 0.0], Z, Linear), (0.645, [34.0, 0.0, 0.0], Z, CubicInOut), (1.0, [14.0, 0.0, 0.0], Z, CubicInOut)]),
    (Chest, &[(0.0, [10.0, 0.0, 0.0], Z, CubicInOut), (0.32, [-8.0, 0.0, 0.0], Z, CubicIn), (0.484, [14.0, 0.0, 0.0], Z, CubicInOut), (1.0, [10.0, 0.0, 0.0], Z, CubicInOut)]),
    (Head, &[(0.0, [-22.0, 0.0, 0.0], Z, CubicInOut), (0.32, [-5.0, 0.0, 0.0], Z, CubicIn), (0.484, [-40.0, 0.0, 0.0], Z, CubicInOut), (1.0, [-22.0, 0.0, 0.0], Z, CubicInOut)]),
    (Hips, &[(0.0, Z, [0.0, -0.04, 0.0], CubicInOut), (0.32, Z, [0.0, 0.01, -0.03], CubicIn), (0.484, Z, [0.0, -0.1, 0.06], Linear), (0.645, Z, [0.0, -0.09, 0.05], CubicInOut), (1.0, Z, [0.0, -0.04, 0.0], CubicInOut)]),
];

#[rustfmt::skip]
const SWEEP: &[Track] = &[
    (Chest, &[(0.0, [10.0, 0.0, 0.0], Z, CubicInOut), (0.32, [4.0, -50.0, 0.0], Z, CubicIn), (0.484, [12.0, 55.0, 0.0], Z, CubicOut), (0.645, [12.0, 50.0, 0.0], Z, CubicInOut), (1.0, [10.0, 0.0, 0.0], Z, CubicInOut)]),
    (UpperArmR, &[(0.0, [-40.0, 0.0, -14.0], Z, CubicInOut), (0.32, [-80.0, 0.0, -75.0], Z, CubicIn), (0.484, [-85.0, 0.0, -15.0], Z, CubicOut), (0.645, [-80.0, 0.0, -10.0], Z, CubicInOut), (1.0, [-40.0, 0.0, -14.0], Z, CubicInOut)]),
    (ForearmR, &[(0.0, [-55.0, 0.0, 0.0], Z, CubicInOut), (0.32, [-30.0, 0.0, 0.0], Z, CubicIn), (0.484, [-5.0, 0.0, 0.0], Z, CubicOut), (0.645, [-15.0, 0.0, 0.0], Z, CubicInOut), (1.0, [-55.0, 0.0, 0.0], Z, CubicInOut)]),
    (HandR, &[(0.0, [20.0, 0.0, 0.0], Z, CubicInOut), (0.32, [0.0, 0.0, -20.0], Z, CubicIn), (0.484, [0.0, 0.0, 20.0], Z, CubicInOut), (1.0, [20.0, 0.0, 0.0], Z, CubicInOut)]),
    (UpperArmL, &[(0.0, [-8.0, 0.0, 12.0], Z, CubicInOut), (0.32, [-60.0, 0.0, 30.0], Z, CubicIn), (0.484, [-20.0, 0.0, 40.0], Z, CubicInOut), (1.0, [-8.0, 0.0, 12.0], Z, CubicInOut)]),
    (Spine, &[(0.0, [14.0, 0.0, 0.0], Z, CubicInOut), (0.32, [8.0, 0.0, 6.0], Z, CubicIn), (0.484, [20.0, 0.0, -6.0], Z, CubicInOut), (1.0, [14.0, 0.0, 0.0], Z, CubicInOut)]),
    (Hips, &[(0.0, Z, [0.0, -0.04, 0.0], CubicInOut), (0.32, Z, [0.0, -0.05, -0.02], CubicIn), (0.484, Z, [0.0, -0.08, 0.05], CubicInOut), (1.0, Z, [0.0, -0.04, 0.0], CubicInOut)]),
];

#[rustfmt::skip]
const BASH: &[Track] = &[
    (UpperArmL, &[(0.0, [-8.0, 0.0, 12.0], Z, CubicInOut), (0.32, [-40.0, 0.0, 30.0], Z, CubicIn), (0.484, [-88.0, 0.0, 5.0], Z, Linear), (0.645, [-85.0, 0.0, 5.0], Z, CubicInOut), (1.0, [-8.0, 0.0, 12.0], Z, CubicInOut)]),
    (ForearmL, &[(0.0, [-22.0, 0.0, 0.0], Z, CubicInOut), (0.32, [-80.0, 0.0, 0.0], Z, CubicIn), (0.484, [-10.0, 0.0, 0.0], Z, Linear), (0.645, [-10.0, 0.0, 0.0], Z, CubicInOut), (1.0, [-22.0, 0.0, 0.0], Z, CubicInOut)]),
    (HandL, &[(0.0, Z, Z, CubicInOut), (0.32, [0.0, -60.0, 0.0], Z, CubicIn), (0.484, [0.0, -90.0, 0.0], Z, CubicInOut), (0.645, [0.0, -90.0, 0.0], Z, CubicInOut), (1.0, Z, Z, CubicInOut)]),
    (Chest, &[(0.0, [10.0, 0.0, 0.0], Z, CubicInOut), (0.32, [5.0, 30.0, 0.0], Z, CubicIn), (0.484, [14.0, -25.0, 0.0], Z, CubicInOut), (1.0, [10.0, 0.0, 0.0], Z, CubicInOut)]),
    (Spine, &[(0.0, [14.0, 0.0, 0.0], Z, CubicInOut), (0.32, [8.0, 0.0, 0.0], Z, CubicIn), (0.484, [24.0, 0.0, 0.0], Z, CubicInOut), (1.0, [14.0, 0.0, 0.0], Z, CubicInOut)]),
    (Hips, &[(0.0, Z, [0.0, -0.04, 0.0], CubicInOut), (0.32, Z, [0.0, -0.04, -0.04], CubicIn), (0.484, Z, [0.0, -0.08, 0.08], Linear), (0.645, Z, [0.0, -0.08, 0.07], CubicInOut), (1.0, Z, [0.0, -0.04, 0.0], CubicInOut)]),
];

#[rustfmt::skip]
const DOUBLE: &[Track] = &[
    (UpperArmR, &[(0.0, [-40.0, 0.0, -14.0], Z, CubicInOut), (0.19, [-190.0, 0.0, -10.0], Z, CubicIn), (0.355, [-50.0, 0.0, -10.0], Z, CubicInOut), (0.55, [-45.0, 0.0, -10.0], Z, CubicInOut), (1.0, [-40.0, 0.0, -14.0], Z, CubicInOut)]),
    (ForearmR, &[(0.0, [-55.0, 0.0, 0.0], Z, CubicInOut), (0.19, [-60.0, 0.0, 0.0], Z, CubicIn), (0.355, [-10.0, 0.0, 0.0], Z, CubicInOut), (1.0, [-55.0, 0.0, 0.0], Z, CubicInOut)]),
    (UpperArmL, &[(0.0, [-30.0, 0.0, 14.0], Z, CubicInOut), (0.32, [-190.0, 0.0, 10.0], Z, CubicIn), (0.484, [-50.0, 0.0, 10.0], Z, Linear), (0.645, [-45.0, 0.0, 10.0], Z, CubicInOut), (1.0, [-30.0, 0.0, 14.0], Z, CubicInOut)]),
    (ForearmL, &[(0.0, [-50.0, 0.0, 0.0], Z, CubicInOut), (0.32, [-60.0, 0.0, 0.0], Z, CubicIn), (0.484, [-10.0, 0.0, 0.0], Z, CubicInOut), (1.0, [-50.0, 0.0, 0.0], Z, CubicInOut)]),
    (HandL, &[(0.0, [20.0, 0.0, 0.0], Z, CubicInOut), (0.32, [-20.0, 0.0, 0.0], Z, CubicIn), (0.484, [40.0, 0.0, 0.0], Z, CubicInOut), (1.0, [20.0, 0.0, 0.0], Z, CubicInOut)]),
    (Chest, &[(0.0, [10.0, 0.0, 0.0], Z, CubicInOut), (0.19, [0.0, -15.0, 0.0], Z, CubicIn), (0.355, [16.0, 20.0, 0.0], Z, CubicIn), (0.484, [18.0, -20.0, 0.0], Z, CubicInOut), (1.0, [10.0, 0.0, 0.0], Z, CubicInOut)]),
    (Spine, &[(0.0, [14.0, 0.0, 0.0], Z, CubicInOut), (0.355, [28.0, 0.0, 0.0], Z, CubicIn), (0.484, [34.0, 0.0, 0.0], Z, CubicInOut), (1.0, [14.0, 0.0, 0.0], Z, CubicInOut)]),
    (Hips, &[(0.0, Z, [0.0, -0.04, 0.0], CubicInOut), (0.484, Z, [0.0, -0.09, 0.05], CubicInOut), (1.0, Z, [0.0, -0.04, 0.0], CubicInOut)]),
];

pub fn pose(m: &Moment, pose: &mut Pose) {
    let f = Form::of(m.tier, m.branch);
    let p = plan(m.tier);
    let reaper = f.b && f.t2;
    match m.clip {
        Clip::Idle => idle(m, pose, &p, f),
        Clip::Walk => walk(m, pose, f),
        Clip::AttackMelee if reaper => reap(m, pose, &p),
        Clip::AttackMelee => attack(m, pose, &p, f),
        Clip::Spawn => rise(m, pose),
        Clip::Evolve => roar(m, pose, &p),
        _ => {}
    }
    if reaper && !matches!(m.clip, Clip::AttackMelee | Clip::Death | Clip::Evolve) {
        scythe(pose, &p, -20.0, 18.0, 0.0);
    }
}

/// The hunch every pose starts from: spine bowed, head raised to look out.
fn hunch(pose: &mut Pose, w: f32) {
    tools::add(pose, Spine, [14.0 * w, 0.0, 0.0]);
    tools::add(pose, Chest, [8.0 * w, 0.0, 0.0]);
    tools::add(pose, Head, [-22.0 * w, 0.0, 0.0]);
}

fn idle(m: &Moment, pose: &mut Pose, p: &Proportions, f: Form) {
    let engaged = m.aim.map_or(0.0, |a| a.weight);
    let breath = (m.clock * 1.3).sin();
    hunch(pose, 1.0);
    tools::add(pose, Chest, [3.0 * breath, 0.0, 0.0]);
    tools::add(
        pose,
        Spine,
        [0.0, 0.0, 2.0 * tools::wave(m.clock, 0.7, 0.3)],
    );
    tools::add(
        pose,
        Head,
        [
            4.0 * tools::wave(m.clock, 0.9, 2.0),
            12.0 * tools::wave(m.clock, 0.35, 0.0),
            0.0,
        ],
    );
    // Arms hang heavy, the cleaver tipped down; engaged, it comes up ready.
    tools::set(
        pose,
        UpperArmR,
        [-18.0 - 22.0 * engaged, 0.0, -10.0 - 4.0 * engaged],
    );
    tools::set(pose, ForearmR, [-40.0 - 15.0 * engaged, 0.0, 0.0]);
    tools::set(pose, HandR, [22.0 - 12.0 * engaged, 0.0, 0.0]);
    let shield = if f.a { -40.0 } else { 0.0 };
    tools::set(
        pose,
        UpperArmL,
        [-8.0 + shield * 0.3 - 20.0 * engaged, 0.0, 12.0],
    );
    tools::set(pose, ForearmL, [-22.0 + shield, 0.0, 0.0]);
    tools::shift(
        pose,
        Hips,
        [0.0, -0.03 - 0.02 * engaged + 0.004 * breath, 0.0],
    );
    tools::face(pose, m.aim, Vec3::ZERO);
    tools::plant(
        pose,
        p,
        tools::stance(p, 1.2 + 0.2 * engaged, 0.03 + 0.04 * engaged),
    );
}

fn walk(m: &Moment, pose: &mut Pose, f: Form) {
    let phase = m.walk.unwrap_or(0.0) * std::f32::consts::TAU;
    hunch(pose, 1.0);
    // Lumbering: the whole trunk rolls onto the planted leg.
    tools::add(pose, Spine, [4.0, 0.0, 6.0 * phase.sin()]);
    tools::add(pose, Chest, [0.0, -10.0 * phase.sin(), -3.0 * phase.sin()]);
    tools::add(
        pose,
        Head,
        [
            -3.0 * (phase * 2.0).cos(),
            6.0 * phase.sin(),
            4.0 * phase.sin(),
        ],
    );
    tools::add(pose, ForearmR, [-35.0, 0.0, 0.0]);
    tools::set(pose, HandR, [25.0, 0.0, 0.0]);
    if f.a {
        tools::add(pose, ForearmL, [-35.0, 0.0, 0.0]);
    }
}

fn attack(m: &Moment, pose: &mut Pose, p: &Proportions, f: Form) {
    let k = (m.t / ATTACK).clamp(0.0, 1.0);
    let tracks = match (f.a && f.t2, f.b, m.variant) {
        (true, _, 0) => BASH,
        (true, _, _) => SLAM,
        (_, true, _) => DOUBLE,
        (_, _, 1) => SWEEP,
        _ => SLAM,
    };
    play(pose, tracks, k);
    tools::face(pose, m.aim, Vec3::ZERO);
    tools::plant(pose, p, tools::stance(p, 1.35, 0.07));
}

/// Reaper: both hands on the scythe, a wide sweep from the right side
/// across the front; the blade passes the target at 0.30 s.
fn reap(m: &Moment, pose: &mut Pose, p: &Proportions) {
    let t = m.t;
    let wind = window(t, 0.0, 0.2);
    let sweep = snap(t, 0.2, 0.3);
    let back = window(t, 0.4, ATTACK);
    let swing = (-20.0 - 95.0 * wind + 165.0 * sweep) * (1.0 - back) + -20.0 * back;
    let tilt = 18.0 + 70.0 * wind.max(sweep) * (1.0 - back);
    hunch(pose, 1.0);
    tools::add(pose, Chest, [0.0, swing * 0.4, 0.0]);
    tools::add(pose, Spine, [16.0 * sweep * (1.0 - back), 0.0, 0.0]);
    tools::shift(
        pose,
        Hips,
        [
            0.0,
            -0.04 - 0.05 * sweep * (1.0 - back),
            0.05 * sweep * (1.0 - back),
        ],
    );
    scythe(pose, p, swing * 0.6, tilt, 0.12 * sweep * (1.0 - back));
    tools::face(pose, m.aim, Vec3::ZERO);
    tools::plant(pose, p, tools::stance(p, 1.4, 0.08));
}

/// Both hands on the haft: `swing` turns it about the body (degrees, +
/// toward the left), `tilt` lowers the blade forward, `drop` lowers the
/// hands.
fn scythe(pose: &mut Pose, p: &Proportions, swing: f32, tilt: f32, drop: f32) {
    let turn = Quat::from_rotation_y(swing.to_radians());
    let grip = turn * Quat::from_rotation_x(tilt.to_radians());
    let wrist = turn * Vec3::new(-0.12, 0.0, 0.3) + Vec3::new(-0.1, 0.08 - drop, 0.0);
    tools::reach(
        pose,
        false,
        wrist,
        Some(grip),
        (p, Vec3::new(-1.0, -0.5, -0.4)),
    );
    let lower = wrist + grip * Vec3::new(0.0, -0.26, 0.0);
    tools::reach(
        pose,
        true,
        lower,
        Some(grip),
        (p, Vec3::new(1.0, -0.5, -0.4)),
    );
}

/// Evolve (0.9 s): gathers itself, then rears up and roars, arms flung
/// wide, the skull thrown back.
fn roar(m: &Moment, pose: &mut Pose, p: &Proportions) {
    let k = (m.t / 0.9).clamp(0.0, 1.0);
    let crouch = hold(k, [0.0, 0.2, 0.25, 0.4]);
    let roar = hold(k, [0.3, 0.45, 0.7, 0.95]);
    pose[Root.index()] = (Vec3::ZERO, Vec3::ZERO);
    hunch(pose, 1.0 - roar);
    tools::add(pose, Spine, [30.0 * crouch - 18.0 * roar, 0.0, 0.0]);
    tools::add(pose, Head, [10.0 * crouch - 30.0 * roar, 0.0, 0.0]);
    for (upper, fore, s) in [(UpperArmL, ForearmL, 1.0f32), (UpperArmR, ForearmR, -1.0)] {
        tools::set(
            pose,
            upper,
            [-20.0 * crouch - 40.0 * roar, 0.0, s * (10.0 + 100.0 * roar)],
        );
        tools::set(pose, fore, [-40.0 * crouch - 30.0 * roar, 0.0, 0.0]);
    }
    tools::shift(pose, Hips, [0.0, -0.08 * crouch, 0.0]);
    tools::plant(pose, p, tools::stance(p, 1.4, 0.03));
}

/// Claws out of the ground: rises from below the BattleLine and stands.
fn rise(m: &Moment, pose: &mut Pose) {
    let k = (m.t / 0.6).clamp(0.0, 1.0);
    let under = (1.0 - k / 0.7).max(0.0);
    pose[Root.index()].1 = Vec3::new(0.0, -under * under, 0.0);
    let claw = 1.0 - window(k, 0.5, 1.0);
    hunch(pose, 1.0);
    tools::set(pose, UpperArmL, [-150.0 * claw, 0.0, 20.0 * claw]);
    tools::set(pose, UpperArmR, [-140.0 * claw, 0.0, -20.0 * claw]);
    tools::add(pose, Spine, [20.0 * claw, 0.0, 0.0]);
}
