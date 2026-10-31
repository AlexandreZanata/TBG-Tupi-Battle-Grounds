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
        }
        Clip::Walk => moves::walk(m, p, 9.0, 4.0),
        Clip::AttackMelee | Clip::AttackRanged => {
            moves::strike(m, p, &body, THRUST, 1.2);
            tools::add(
                p,
                UpperArmL,
                [-15.0 * tools::hold(m.t, [0.0, 0.15, 0.25, 0.5]), 0.0, 0.0],
            );
        }
        Clip::Hit => moves::hit(m, p, 0.5),
        Clip::Death => moves::death(m, p, &body, Fall::Kneel),
        Clip::Spawn => moves::spawn(m, p, &body, Entry::Drop),
        Clip::Evolve => {
            moves::evolve(m, p, &body, Power::Raise);
            slam(p, m.t, 0.5);
        }
        Clip::Victory => moves::victory(m, p, &body, Cheer::Both),
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
