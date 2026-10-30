//! Shared motion blocks for kit champions, one per archetype gesture and
//! per life moment, each with styles so champions differ. Kits use the
//! common clip lengths below; the event times never move (ART-DIRECTION §4):
//! Strike at 0.30 s (k 0.484), ranged Release at 0.18 s (k 0.29), cast
//! Release at 0.22 s (k 0.355).

use bevy::prelude::*;

use crate::art::{
    anim::{Clip, Easing::*, Pose},
    kits::{
        tools::{self, hold, play, snap, window, Track},
        Moment,
    },
    rig::{Joint::*, Proportions},
};

/// Clip lengths every new kit uses (longer recoveries, same events).
pub const DURATIONS: &[(Clip, f32)] = &[
    (Clip::AttackMelee, ATTACK),
    (Clip::AttackRanged, ATTACK),
    (Clip::Cast, ATTACK),
    (Clip::Spawn, 0.6),
    (Clip::Evolve, 0.9),
    (Clip::Death, 0.9),
];
pub const ATTACK: f32 = 0.62;

const Z: [f32; 3] = [0.0; 3];

/// Two-handed overhead chop (axes, hammers, mallets).
#[rustfmt::skip]
pub const CHOP: &[Track] = &[
    (UpperArmR, &[(0.0, [-30.0, 0.0, -10.0], Z, CubicInOut), (0.32, [-190.0, 0.0, -8.0], Z, CubicIn), (0.484, [-45.0, 0.0, -6.0], Z, Linear), (0.645, [-40.0, 0.0, -6.0], Z, CubicInOut), (1.0, [-30.0, 0.0, -10.0], Z, CubicInOut)]),
    (ForearmR, &[(0.0, [-40.0, 0.0, 0.0], Z, CubicInOut), (0.32, [-55.0, 0.0, 0.0], Z, CubicIn), (0.484, [-8.0, 0.0, 0.0], Z, Linear), (1.0, [-40.0, 0.0, 0.0], Z, CubicInOut)]),
    (UpperArmL, &[(0.0, [-30.0, 0.0, 10.0], Z, CubicInOut), (0.32, [-185.0, 0.0, 6.0], Z, CubicIn), (0.484, [-50.0, 0.0, 6.0], Z, Linear), (0.645, [-45.0, 0.0, 6.0], Z, CubicInOut), (1.0, [-30.0, 0.0, 10.0], Z, CubicInOut)]),
    (ForearmL, &[(0.0, [-40.0, 0.0, 0.0], Z, CubicInOut), (0.32, [-50.0, 0.0, 0.0], Z, CubicIn), (0.484, [-10.0, 0.0, 0.0], Z, Linear), (1.0, [-40.0, 0.0, 0.0], Z, CubicInOut)]),
    (Spine, &[(0.0, [4.0, 0.0, 0.0], Z, CubicInOut), (0.32, [-14.0, 0.0, 0.0], Z, CubicIn), (0.484, [26.0, 0.0, 0.0], Z, Linear), (0.645, [24.0, 0.0, 0.0], Z, CubicInOut), (1.0, [4.0, 0.0, 0.0], Z, CubicInOut)]),
    (Hips, &[(0.0, Z, [0.0, -0.02, 0.0], CubicInOut), (0.32, Z, [0.0, 0.02, -0.03], CubicIn), (0.484, Z, [0.0, -0.07, 0.06], Linear), (0.645, Z, [0.0, -0.06, 0.05], CubicInOut), (1.0, Z, [0.0, -0.02, 0.0], CubicInOut)]),
];

/// One-handed side slash, right arm (swords, cleavers, claws).
#[rustfmt::skip]
pub const SLASH: &[Track] = &[
    (Chest, &[(0.0, [4.0, 0.0, 0.0], Z, CubicInOut), (0.32, [0.0, -45.0, 0.0], Z, CubicIn), (0.484, [8.0, 50.0, 0.0], Z, CubicOut), (0.645, [8.0, 45.0, 0.0], Z, CubicInOut), (1.0, [4.0, 0.0, 0.0], Z, CubicInOut)]),
    (UpperArmR, &[(0.0, [-30.0, 0.0, -12.0], Z, CubicInOut), (0.32, [-95.0, 0.0, -80.0], Z, CubicIn), (0.484, [-85.0, 0.0, -10.0], Z, CubicOut), (0.645, [-75.0, 0.0, -5.0], Z, CubicInOut), (1.0, [-30.0, 0.0, -12.0], Z, CubicInOut)]),
    (ForearmR, &[(0.0, [-45.0, 0.0, 0.0], Z, CubicInOut), (0.32, [-60.0, 0.0, 0.0], Z, CubicIn), (0.484, [-5.0, 0.0, 0.0], Z, CubicOut), (1.0, [-45.0, 0.0, 0.0], Z, CubicInOut)]),
    (UpperArmL, &[(0.0, [-10.0, 0.0, 12.0], Z, CubicInOut), (0.32, [-40.0, 0.0, 30.0], Z, CubicIn), (0.484, [-10.0, 0.0, 35.0], Z, CubicInOut), (1.0, [-10.0, 0.0, 12.0], Z, CubicInOut)]),
    (Hips, &[(0.0, Z, [0.0, -0.02, 0.0], CubicInOut), (0.32, Z, [0.0, -0.03, -0.02], CubicIn), (0.484, Z, [0.0, -0.06, 0.05], CubicInOut), (1.0, Z, [0.0, -0.02, 0.0], CubicInOut)]),
];

/// A lunging thrust with the right arm (spears, daggers, pincers).
#[rustfmt::skip]
pub const THRUST: &[Track] = &[
    (UpperArmR, &[(0.0, [-35.0, 0.0, -12.0], Z, CubicInOut), (0.32, [-60.0, 0.0, -25.0], Z, CubicIn), (0.484, [-92.0, 0.0, -4.0], Z, Linear), (0.645, [-88.0, 0.0, -4.0], Z, CubicInOut), (1.0, [-35.0, 0.0, -12.0], Z, CubicInOut)]),
    (ForearmR, &[(0.0, [-40.0, 0.0, 0.0], Z, CubicInOut), (0.32, [-95.0, 0.0, 0.0], Z, CubicIn), (0.484, [-4.0, 0.0, 0.0], Z, Linear), (1.0, [-40.0, 0.0, 0.0], Z, CubicInOut)]),
    (Chest, &[(0.0, [4.0, 0.0, 0.0], Z, CubicInOut), (0.32, [-4.0, -25.0, 0.0], Z, CubicIn), (0.484, [12.0, 20.0, 0.0], Z, Linear), (1.0, [4.0, 0.0, 0.0], Z, CubicInOut)]),
    (Hips, &[(0.0, Z, [0.0, -0.02, 0.0], CubicInOut), (0.32, Z, [0.0, -0.03, -0.06], CubicIn), (0.484, Z, [0.0, -0.07, 0.1], Linear), (0.645, Z, [0.0, -0.06, 0.09], CubicInOut), (1.0, Z, [0.0, -0.02, 0.0], CubicInOut)]),
];

/// An overhand throw with the right arm; the release lands at k 0.29.
#[rustfmt::skip]
pub const THROW: &[Track] = &[
    (UpperArmR, &[(0.0, [-20.0, 0.0, -10.0], Z, CubicInOut), (0.2, [-170.0, 0.0, -25.0], Z, CubicIn), (0.29, [-80.0, 0.0, -10.0], Z, CubicOut), (0.45, [-30.0, 0.0, -8.0], Z, CubicInOut), (1.0, [-20.0, 0.0, -10.0], Z, CubicInOut)]),
    (ForearmR, &[(0.0, [-30.0, 0.0, 0.0], Z, CubicInOut), (0.2, [-90.0, 0.0, 0.0], Z, CubicIn), (0.29, [-10.0, 0.0, 0.0], Z, CubicOut), (1.0, [-30.0, 0.0, 0.0], Z, CubicInOut)]),
    (UpperArmL, &[(0.0, [-10.0, 0.0, 10.0], Z, CubicInOut), (0.2, [-70.0, 0.0, 20.0], Z, CubicIn), (0.29, [-20.0, 0.0, 25.0], Z, CubicOut), (1.0, [-10.0, 0.0, 10.0], Z, CubicInOut)]),
    (Chest, &[(0.0, [2.0, 0.0, 0.0], Z, CubicInOut), (0.2, [-12.0, -30.0, 0.0], Z, CubicIn), (0.29, [14.0, 25.0, 0.0], Z, CubicOut), (1.0, [2.0, 0.0, 0.0], Z, CubicInOut)]),
    (Hips, &[(0.0, Z, Z, CubicInOut), (0.2, Z, [0.0, 0.0, -0.05], CubicIn), (0.29, Z, [0.0, -0.04, 0.06], CubicOut), (1.0, Z, Z, CubicInOut)]),
];

/// Both hands raised, then pushed at the target; release at k 0.355.
#[rustfmt::skip]
pub const CAST: &[Track] = &[
    (UpperArmL, &[(0.0, [-15.0, 0.0, 10.0], Z, CubicInOut), (0.25, [-160.0, 0.0, 25.0], Z, CubicIn), (0.355, [-95.0, 0.0, 8.0], Z, CubicOut), (0.6, [-90.0, 0.0, 8.0], Z, CubicInOut), (1.0, [-15.0, 0.0, 10.0], Z, CubicInOut)]),
    (UpperArmR, &[(0.0, [-15.0, 0.0, -10.0], Z, CubicInOut), (0.25, [-160.0, 0.0, -25.0], Z, CubicIn), (0.355, [-95.0, 0.0, -8.0], Z, CubicOut), (0.6, [-90.0, 0.0, -8.0], Z, CubicInOut), (1.0, [-15.0, 0.0, -10.0], Z, CubicInOut)]),
    (ForearmL, &[(0.0, [-20.0, 0.0, 0.0], Z, CubicInOut), (0.355, [-5.0, 0.0, 0.0], Z, CubicInOut), (1.0, [-20.0, 0.0, 0.0], Z, CubicInOut)]),
    (ForearmR, &[(0.0, [-20.0, 0.0, 0.0], Z, CubicInOut), (0.355, [-5.0, 0.0, 0.0], Z, CubicInOut), (1.0, [-20.0, 0.0, 0.0], Z, CubicInOut)]),
    (Chest, &[(0.0, Z, Z, CubicInOut), (0.25, [-12.0, 0.0, 0.0], Z, CubicIn), (0.355, [14.0, 0.0, 0.0], Z, CubicOut), (1.0, Z, Z, CubicInOut)]),
    (Hips, &[(0.0, Z, Z, CubicInOut), (0.25, Z, [0.0, 0.04, 0.0], CubicIn), (0.355, Z, [0.0, -0.05, 0.04], CubicOut), (1.0, Z, Z, CubicInOut)]),
];

/// One hand (and its staff) lifted, then pointed at the target; release at
/// k 0.355 (supports, single-hand casters).
#[rustfmt::skip]
pub const POINT: &[Track] = &[
    (UpperArmR, &[(0.0, [-20.0, 0.0, -10.0], Z, CubicInOut), (0.25, [-150.0, 0.0, -20.0], Z, CubicIn), (0.355, [-85.0, 0.0, -6.0], Z, CubicOut), (0.6, [-80.0, 0.0, -6.0], Z, CubicInOut), (1.0, [-20.0, 0.0, -10.0], Z, CubicInOut)]),
    (ForearmR, &[(0.0, [-30.0, 0.0, 0.0], Z, CubicInOut), (0.25, [-20.0, 0.0, 0.0], Z, CubicIn), (0.355, [-5.0, 0.0, 0.0], Z, CubicOut), (1.0, [-30.0, 0.0, 0.0], Z, CubicInOut)]),
    (UpperArmL, &[(0.0, [-10.0, 0.0, 10.0], Z, CubicInOut), (0.25, [-40.0, 0.0, 40.0], Z, CubicIn), (0.355, [-20.0, 0.0, 30.0], Z, CubicOut), (1.0, [-10.0, 0.0, 10.0], Z, CubicInOut)]),
    (Chest, &[(0.0, Z, Z, CubicInOut), (0.25, [-10.0, -10.0, 0.0], Z, CubicIn), (0.355, [10.0, 10.0, 0.0], Z, CubicOut), (1.0, Z, Z, CubicInOut)]),
    (Hips, &[(0.0, Z, Z, CubicInOut), (0.25, Z, [0.0, 0.03, 0.0], CubicIn), (0.355, Z, [0.0, -0.03, 0.03], CubicOut), (1.0, Z, Z, CubicInOut)]),
];

/// Play an attack table over the common length and keep the feet down.
pub fn strike(m: &Moment, pose: &mut Pose, p: &Proportions, table: &[Track], wide: f32) {
    play(pose, table, (m.t / ATTACK).clamp(0.0, 1.0));
    tools::face(pose, m.aim, Vec3::ZERO);
    tools::plant(pose, p, tools::stance(p, wide, 0.06));
}

/// Breathing and a slow look around; `weight` is how heavy the body is.
pub fn idle(m: &Moment, pose: &mut Pose, p: &Proportions, weight: f32) {
    let breath = (m.clock * (1.9 - 0.5 * weight)).sin();
    tools::add(pose, Chest, [2.0 * breath * (1.0 + weight), 0.0, 0.0]);
    tools::add(
        pose,
        Spine,
        [0.0, 0.0, 1.5 * tools::wave(m.clock, 0.6, 0.4)],
    );
    tools::add(
        pose,
        Head,
        [
            3.0 * tools::wave(m.clock, 0.8, 2.0),
            14.0 * tools::wave(m.clock, 0.33, 1.0),
            0.0,
        ],
    );
    tools::shift(pose, Hips, [0.0, -0.01 * weight + 0.004 * breath, 0.0]);
    tools::face(pose, m.aim, Vec3::ZERO);
    let engaged = m.aim.map_or(0.0, |a| a.weight);
    tools::shift(pose, Hips, [0.0, -0.025 * engaged, 0.0]);
    tools::plant(
        pose,
        p,
        tools::stance(p, 1.1 + 0.25 * engaged, 0.03 * engaged),
    );
}

/// A stride's upper body: lean forward, roll onto the planted leg.
pub fn walk(m: &Moment, pose: &mut Pose, lean: f32, roll: f32) {
    let phase = m.walk.unwrap_or(0.0) * std::f32::consts::TAU;
    tools::add(pose, Spine, [lean, 0.0, roll * phase.sin()]);
    tools::add(pose, Chest, [0.0, -roll * 1.5 * phase.sin(), 0.0]);
    tools::add(
        pose,
        Head,
        [-lean * 0.6, roll * phase.sin(), -roll * 0.5 * phase.sin()],
    );
}

/// Knocked back by a blow: `power` 1 for humans, 0.6 for heavies.
pub fn hit(m: &Moment, pose: &mut Pose, power: f32) {
    let k = hold((m.t / 0.25).clamp(0.0, 1.0), [0.0, 0.25, 0.4, 1.0]);
    tools::add(pose, Chest, [-14.0 * power * k, 0.0, 6.0 * power * k]);
    tools::add(pose, Head, [-12.0 * power * k, 8.0 * k, 0.0]);
    tools::add(pose, UpperArmL, [-25.0 * power * k, 0.0, 20.0 * k]);
    tools::add(pose, UpperArmR, [-25.0 * power * k, 0.0, -20.0 * k]);
}

#[derive(Clone, Copy)]
pub enum Fall {
    /// Knees buckle, then a fall onto the back.
    Back,
    /// Staggers and falls on the face.
    Forward,
    /// Drops to the knees and slumps there.
    Kneel,
    /// Sinks into the ground (skeletons, plants, constructs breaking down).
    Sink,
}

/// Death over 0.9 s; the figure stays down until it is removed.
pub fn death(m: &Moment, pose: &mut Pose, p: &Proportions, fall: Fall) {
    let k = (m.t / 0.9).clamp(0.0, 1.0);
    let buckle = window(k, 0.0, 0.3);
    let drop = window(k, 0.25, 0.7);
    pose[Root.index()] = (Vec3::ZERO, Vec3::ZERO);
    for j in [Spine, Chest, Head] {
        pose[j.index()].0 = Vec3::ZERO;
    }
    match fall {
        Fall::Back => {
            pose[Root.index()] = (
                Vec3::new(-82.0 * drop, 0.0, 6.0 * drop),
                Vec3::new(0.0, 0.08 * drop, -0.2 * drop),
            );
            tools::shift(pose, Hips, [0.0, -0.08 * buckle, 0.0]);
            tools::set(pose, UpperArmL, [-110.0 * drop, 0.0, 60.0 * drop]);
            tools::set(pose, UpperArmR, [-100.0 * drop, 0.0, -70.0 * drop]);
            tools::set(pose, Head, [20.0 * drop, 20.0 * drop, 0.0]);
        }
        Fall::Forward => {
            pose[Root.index()] = (
                Vec3::new(80.0 * drop, 0.0, -5.0 * drop),
                Vec3::new(0.0, 0.06 * drop, 0.2 * drop),
            );
            tools::set(pose, Spine, [20.0 * buckle, 0.0, 0.0]);
            tools::set(pose, UpperArmL, [-150.0 * drop, 0.0, 30.0]);
            tools::set(pose, UpperArmR, [-140.0 * drop, 0.0, -30.0]);
            tools::set(pose, Head, [-30.0 * drop, 0.0, 0.0]);
        }
        Fall::Kneel => {
            tools::shift(pose, Hips, [0.0, -p.leg * 0.55 * window(k, 0.0, 0.4), 0.0]);
            tools::set(pose, Spine, [45.0 * drop, 0.0, 10.0 * drop]);
            tools::set(pose, Head, [35.0 * drop, 0.0, 0.0]);
            tools::set(pose, UpperArmL, [10.0, 0.0, 12.0 * drop]);
            tools::set(pose, UpperArmR, [10.0, 0.0, -12.0 * drop]);
            pose[Root.index()].0.x = 20.0 * window(k, 0.6, 1.0);
        }
        Fall::Sink => {
            tools::set(pose, Spine, [30.0 * buckle, 0.0, 15.0 * buckle]);
            tools::set(pose, Head, [30.0 * drop, 0.0, 0.0]);
            tools::set(pose, UpperArmL, [-30.0, 0.0, 40.0 * drop]);
            tools::set(pose, UpperArmR, [-30.0, 0.0, -40.0 * drop]);
            pose[Root.index()].1 = Vec3::Y * -p.leg * 2.2 * window(k, 0.35, 1.0);
        }
    }
    if !matches!(fall, Fall::Kneel) {
        tools::plant(pose, p, tools::stance(p, 1.2, 0.0));
    } else {
        tools::plant(pose, p, tools::stance(p, 1.3, 0.12));
    }
}

#[derive(Clone, Copy)]
pub enum Entry {
    /// Dropped from above, squats on landing.
    Drop,
    /// Climbs out of the ground.
    Rise,
    /// Strides in from behind, chest out.
    March,
    /// Hops up and lands with a flourish.
    Pop,
}

/// Arrival over 0.6 s.
pub fn spawn(m: &Moment, pose: &mut Pose, p: &Proportions, entry: Entry) {
    let k = (m.t / 0.6).clamp(0.0, 1.0);
    let (lift, back) = match entry {
        Entry::Drop => {
            let fall = (1.0 - k / 0.55).max(0.0);
            (0.9 * fall * fall, 0.0)
        }
        Entry::Rise => {
            let under = (1.0 - k / 0.7).max(0.0);
            (-p.leg * 2.3 * under * under, 0.0)
        }
        Entry::March => (0.0, -0.5 * (1.0 - window(k, 0.0, 0.8))),
        Entry::Pop => (
            0.35 * (k * std::f32::consts::PI).sin().max(0.0) * (1.0 - window(k, 0.7, 1.0)),
            0.0,
        ),
    };
    pose[Root.index()] = (Vec3::ZERO, Vec3::new(0.0, lift, back));
    let squat = hold(k, [0.5, 0.62, 0.7, 1.0]);
    tools::shift(pose, Hips, [0.0, -0.07 * squat, 0.0]);
    tools::add(pose, Spine, [14.0 * squat, 0.0, 0.0]);
    let reach = 1.0 - window(k, 0.45, 1.0);
    let up = match entry {
        Entry::Rise => 150.0,
        Entry::Pop => 120.0,
        _ => 40.0,
    };
    tools::set(pose, UpperArmL, [-up * reach, 0.0, 20.0 * reach]);
    tools::set(pose, UpperArmR, [-up * reach, 0.0, -20.0 * reach]);
    tools::plant(pose, p, tools::stance(p, 1.2, 0.03));
}

#[derive(Clone, Copy)]
pub enum Power {
    /// Gathers, then rises with the weapon held high.
    Raise,
    /// A full turn with arms wide.
    Spin,
    /// Crouches, then bursts up with arms flung wide.
    Burst,
}

/// The Evolution flourish over 0.9 s (the form changes at its start).
pub fn evolve(m: &Moment, pose: &mut Pose, p: &Proportions, power: Power) {
    let k = (m.t / 0.9).clamp(0.0, 1.0);
    let gather = hold(k, [0.0, 0.2, 0.25, 0.4]);
    let peak = hold(k, [0.3, 0.45, 0.72, 0.95]);
    pose[Root.index()] = (Vec3::ZERO, Vec3::Y * 0.06 * peak);
    tools::shift(pose, Hips, [0.0, -0.08 * gather, 0.0]);
    tools::add(pose, Spine, [24.0 * gather - 10.0 * peak, 0.0, 0.0]);
    tools::add(pose, Head, [10.0 * gather - 20.0 * peak, 0.0, 0.0]);
    match power {
        Power::Raise => {
            tools::set(pose, UpperArmR, [-20.0 - 150.0 * peak, 0.0, -10.0]);
            tools::set(pose, ForearmR, [-20.0 * (1.0 - peak), 0.0, 0.0]);
            tools::set(
                pose,
                UpperArmL,
                [-20.0 * gather - 30.0 * peak, 0.0, 30.0 * peak],
            );
        }
        Power::Spin => {
            pose[Root.index()].0.y = 360.0 * window(k, 0.15, 0.85);
            tools::set(pose, UpperArmL, [-10.0, 0.0, 10.0 + 80.0 * peak]);
            tools::set(pose, UpperArmR, [-10.0, 0.0, -10.0 - 80.0 * peak]);
        }
        Power::Burst => {
            for (upper, fore, s) in [(UpperArmL, ForearmL, 1.0f32), (UpperArmR, ForearmR, -1.0)] {
                tools::set(
                    pose,
                    upper,
                    [-20.0 * gather - 60.0 * peak, 0.0, s * (10.0 + 100.0 * peak)],
                );
                tools::set(pose, fore, [-50.0 * gather - 20.0 * peak, 0.0, 0.0]);
            }
        }
    }
    tools::plant(pose, p, tools::stance(p, 1.3, 0.03));
}

#[derive(Clone, Copy)]
pub enum Cheer {
    /// Pumps the weapon arm.
    Raise,
    /// Both arms up, bouncing.
    Both,
    /// Rocks side to side.
    Dance,
}

/// The victory loop (0.8 s).
pub fn victory(m: &Moment, pose: &mut Pose, p: &Proportions, cheer: Cheer) {
    let k = (m.t / 0.8).rem_euclid(1.0);
    let beat = (k * std::f32::consts::TAU).sin();
    pose[Root.index()] = (Vec3::ZERO, Vec3::Y * 0.04 * beat.max(0.0));
    for j in [UpperArmL, UpperArmR, ForearmL, ForearmR] {
        pose[j.index()].0 = Vec3::ZERO;
    }
    match cheer {
        Cheer::Raise => {
            tools::set(pose, UpperArmR, [-160.0 - 15.0 * beat, 0.0, -15.0]);
            tools::set(pose, UpperArmL, [-20.0, 0.0, 25.0]);
            tools::set(pose, ForearmL, [-60.0, 0.0, 0.0]);
        }
        Cheer::Both => {
            tools::set(pose, UpperArmL, [-160.0 - 10.0 * beat, 0.0, 25.0]);
            tools::set(pose, UpperArmR, [-160.0 + 10.0 * beat, 0.0, -25.0]);
        }
        Cheer::Dance => {
            tools::add(pose, Spine, [0.0, 0.0, 10.0 * beat]);
            tools::set(pose, UpperArmL, [-40.0, 0.0, 50.0 + 20.0 * beat]);
            tools::set(pose, UpperArmR, [-40.0, 0.0, -50.0 + 20.0 * beat]);
        }
    }
    tools::add(pose, Head, [-15.0, 10.0 * beat, 0.0]);
    tools::plant(pose, p, tools::stance(p, 1.2, 0.0));
}

/// Snap a weapon arm into a ready guard while engaged (`w` 0..1).
pub fn guard(pose: &mut Pose, w: f32) {
    tools::add(pose, UpperArmR, [-30.0 * w, 0.0, -6.0 * w]);
    tools::add(pose, ForearmR, [-40.0 * w, 0.0, 0.0]);
    tools::add(pose, UpperArmL, [-25.0 * w, 0.0, 8.0 * w]);
    tools::add(pose, ForearmL, [-30.0 * w, 0.0, 0.0]);
}

/// How far into a one-shot clip the moment is, after `start` seconds.
pub fn past(m: &Moment, start: f32, span: f32) -> f32 {
    snap(m.t, start, start + span)
}

/// Both hands on a pole held in the right hand's socket (haft along +Y):
/// `swing` turns it about the body (degrees, + toward the left), `tilt`
/// leans its top forward (degrees), `drop` lowers the hands, `span` is
/// how far down the haft the left hand holds.
pub fn pole(pose: &mut Pose, p: &Proportions, (swing, tilt): (f32, f32), drop: f32, span: f32) {
    let turn = Quat::from_rotation_y(swing.to_radians());
    let grip = turn * Quat::from_rotation_x(tilt.to_radians());
    let wrist = turn * Vec3::new(-0.1, 0.0, 0.26) + Vec3::new(-0.08, 0.1 - drop, 0.0);
    tools::reach(
        pose,
        false,
        wrist,
        Some(grip),
        (p, Vec3::new(-1.0, -0.5, -0.4)),
    );
    let lower = wrist + grip * Vec3::new(0.0, -span, 0.0);
    tools::reach(
        pose,
        true,
        lower,
        Some(grip),
        (p, Vec3::new(1.0, -0.5, -0.4)),
    );
}

/// A long weapon at the shoulder in both hands (crossbows, guns): the
/// right hand grips at `at` (chest space) with the weapon turned by
/// `turn`; the left hand holds `fore` (weapon space).
pub fn shoulder_arms(pose: &mut Pose, p: &Proportions, at: Vec3, turn: Quat, fore: Vec3) {
    tools::reach(
        pose,
        false,
        at,
        Some(turn),
        (p, Vec3::new(-1.0, -0.7, -0.5)),
    );
    tools::reach(
        pose,
        true,
        at + turn * fore,
        Some(turn),
        (p, Vec3::new(1.0, -0.8, -0.3)),
    );
}

/// Elevation (radians) and body yaw that put a weapon at chest height
/// onto the aim, for the shot `arc`; applies the yaw to the Root.
pub fn aim_level(m: &Moment, pose: &mut Pose, height: f32, arc: f32) -> f32 {
    let Some(aim) = m.aim else {
        return 0.0;
    };
    tools::face(pose, m.aim, Vec3::ZERO);
    let from = Vec3::Y * height;
    let need = crate::art::kits::aim::tangent(from, aim.target, arc, 0.0);
    crate::art::kits::aim::pitch(need).clamp(-0.5, 0.9) * aim.weight
}

/// The overhand `THROW` retimed so the release lands on a cast's Release
/// (k 0.355) instead of a ranged one (k 0.29): tossed flasks, crates.
pub fn toss(m: &Moment, pose: &mut Pose, p: &Proportions) {
    let k = (m.t / ATTACK).clamp(0.0, 1.0);
    let k = if k < 0.355 {
        k * 0.29 / 0.355
    } else {
        0.29 + (k - 0.355) * 0.71 / 0.645
    };
    play(pose, THROW, k);
    tools::face(pose, m.aim, Vec3::ZERO);
    tools::plant(pose, p, tools::stance(p, 1.2, 0.06));
}

/// A staff (along the right socket's +Y) held upright at the right side:
/// `tilt` leans its head forward (degrees), `lift` raises the hand,
/// `fore` pushes it forward.
pub fn staff(pose: &mut Pose, p: &Proportions, tilt: f32, lift: f32, fore: f32) {
    let grip = Quat::from_rotation_x(tilt.to_radians());
    tools::reach(
        pose,
        false,
        Vec3::new(-0.22, 0.06 + lift, 0.12 + fore),
        Some(grip),
        (p, Vec3::new(-1.0, -0.5, -0.4)),
    );
}

/// A staff cast: raise the staff, thrust its head at the target on the
/// cast Release (0.22 s), the free hand clawing after it; then settle.
pub fn staff_cast(m: &Moment, pose: &mut Pose, p: &Proportions) {
    let raise = window(m.t, 0.0, 0.2);
    let thrust = snap(m.t, 0.2, 0.24) * (1.0 - window(m.t, 0.4, 0.62));
    tools::face(pose, m.aim, Vec3::ZERO);
    tools::add(
        pose,
        Chest,
        [-8.0 * raise * (1.0 - thrust) + 12.0 * thrust, 0.0, 0.0],
    );
    staff(
        pose,
        p,
        -15.0 * raise + 70.0 * thrust,
        0.15 * raise,
        0.18 * thrust,
    );
    tools::reach(
        pose,
        true,
        Vec3::new(0.18, 0.2 + 0.1 * raise, 0.25 + 0.1 * thrust),
        None,
        (p, Vec3::new(1.0, -0.6, -0.3)),
    );
    tools::plant(pose, p, tools::stance(p, 1.3, 0.08));
}
