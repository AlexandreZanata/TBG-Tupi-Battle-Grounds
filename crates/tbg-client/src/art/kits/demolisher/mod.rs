//! Demolisher's staggered piston settling, hydraulic punch and independent double-fist slam.

mod model;
use super::{
    common::{moves, moves::*},
    tools, Attack, Gait, Kit, Moment,
};
use crate::art::{
    anim::{Clip, Pose},
    rig::{Joint::*, Node::Body},
};
use crate::vfx::impacts::{Burst, Impact};
use bevy::prelude::*;
pub const KIT: Kit = Kit {
    anatomy: None,
    figure: model::spec,
    pose,
    gait: Gait::HEAVY,
    attack: Attack::Melee {
        from: Body(HandR),
        tip: |_, _| Vec3::new(0.0, -0.05, 0.25),
        impact: |_, _| Impact::Sparks,
    },
    evolve: Burst::Forge,
    durations: moves::DURATIONS,
};
fn pose(m: &Moment, p: &mut Pose) {
    let body = model::plan(m.tier);
    match m.clip {
        Clip::Idle => {
            moves::idle(m, p, &body, 0.6);
            tools::add(p, ForearmL, [(m.clock * 1.2).sin() * 4.0, 0.0, 0.0]);
            tools::add(p, ForearmR, [(m.clock * 1.2 + 1.7).sin() * 4.0, 0.0, 0.0]);
            tools::set(p, UpperArmL, [-6.0, 0.0, 12.0]);
            tools::set(p, UpperArmR, [-6.0, 0.0, -12.0]);
            tools::add(p, Head, [-5.0, 6.0 * (m.clock * 0.5).sin(), 0.0]);
        }
        Clip::Walk => {
            moves::walk(m, p, 9.0, 4.0);
            let step = (m.walk.unwrap_or(0.0) * std::f32::consts::TAU).sin();
            tools::set(p, UpperArmL, [-10.0 + 10.0 * step, 0.0, 15.0]);
            tools::set(p, UpperArmR, [-10.0 - 10.0 * step, 0.0, -15.0]);
            tools::set(p, ForearmL, [-10.0, 0.0, 0.0]);
            tools::set(p, ForearmR, [-10.0, 0.0, 0.0]);
        }
        Clip::AttackMelee | Clip::AttackRanged => {
            let load = tools::hold(m.t, [0.0, 0.16, 0.23, 0.30]);
            let drive = tools::hold(m.t, [0.23, 0.30, 0.38, 0.62]);
            tools::set(p, UpperArmR, [-35.0 - 48.0 * drive, 0.0, -12.0]);
            tools::set(p, ForearmR, [-65.0 * load - 8.0 * drive, 0.0, 0.0]);
            tools::set(p, UpperArmL, [-18.0, 0.0, 18.0]);
            tools::set(p, ForearmL, [-30.0, 0.0, 0.0]);
            tools::set(p, Chest, [7.0 * drive, -9.0 * load + 12.0 * drive, 0.0]);
            tools::face(p, m.aim, Vec3::ZERO);
            tools::plant(p, &body, tools::stance(&body, 1.35, 0.07));
        }
        Clip::Hit => moves::hit(m, p, 0.5),
        Clip::Death => moves::death(m, p, &body, Fall::Kneel),
        Clip::Spawn => moves::spawn(m, p, &body, Entry::Drop),
        Clip::Evolve => {
            moves::evolve(m, p, &body, Power::Raise);
            slam(p, m.t, 0.5);
        }
        Clip::Victory => {
            let pump = (m.t / 0.8 * std::f32::consts::TAU).sin();
            tools::set(p, UpperArmL, [-25.0 - 35.0 * pump.max(0.0), 0.0, 22.0]);
            tools::set(p, UpperArmR, [-25.0 - 35.0 * (-pump).max(0.0), 0.0, -22.0]);
            tools::set(p, ForearmL, [-20.0 - 20.0 * pump.max(0.0), 0.0, 0.0]);
            tools::set(p, ForearmR, [-20.0 - 20.0 * (-pump).max(0.0), 0.0, 0.0]);
            tools::set(p, Head, [-6.0, 8.0 * pump, 0.0]);
            tools::set(p, Chest, [4.0, 0.0, 3.0 * pump]);
            tools::plant(p, &body, tools::stance(&body, 1.4, 0.0));
        }
        Clip::Cast => slam(p, m.t, 1.0),
    }
    if let Some(t) = m.special {
        slam(p, t, 1.0);
    }
}
fn slam(p: &mut Pose, t: f32, w: f32) {
    let wind = tools::hold(t, [0.0, 0.20, 0.23, 0.30]) * w;
    let down = tools::hold(t, [0.23, 0.30, 0.40, 0.75]) * w;
    for j in [UpperArmL, UpperArmR] {
        tools::set(p, j, [-145.0 * wind + 15.0 * down, 0.0, 0.0]);
    }
    for j in [ForearmL, ForearmR] {
        tools::set(p, j, [-35.0 * wind - 12.0 * down, 0.0, 0.0]);
    }
    tools::add(p, Chest, [-8.0 * wind + 15.0 * down, 0.0, 0.0]);
    tools::shift(p, Hips, [0.0, -0.04 * down, 0.0]);
}
