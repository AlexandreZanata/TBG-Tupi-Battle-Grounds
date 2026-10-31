//! Beetling kit: a hatchling beetle. It sniffs about with small jerks,
//! scuttles on six legs, rears and lunges to bite on the Strike, flips on
//! its back on death and hops out of its egg on arrival.

mod model;

use bevy::prelude::*;

use super::common::{moves, moves::*};
use super::{tools, Attack, Gait, Kit, Moment};
use crate::art::{
    anim::{Clip, Pose},
    rig::{Joint, Node, Proportions},
};
use crate::vfx::impacts::{Burst, Impact};
use model::plan;

/// Quick, low steps.
const SCUTTLE: Gait = Gait {
    stride: 0.9,
    lift: 0.35,
    bob: 0.01,
    sink: 0.03,
    stomp: false,
};

pub const KIT: Kit = Kit {
    anatomy: None,
    figure: model::spec,
    pose,
    gait: SCUTTLE,
    attack: Attack::Melee {
        from: Node::Body(Joint::Chest),
        tip: |_, _| model::tip(),
        impact: |_, _| Impact::Bite,
    },
    evolve: Burst::Bloom,
    durations: moves::DURATIONS,
};

pub fn pose(m: &Moment, pose: &mut Pose) {
    let p = plan(m.tier);
    match m.clip {
        Clip::Idle => {
            moves::idle(m, pose, &p, 0.1);
            let sniff = (m.clock * 1.7).sin().signum()
                * tools::hold((m.clock * 1.7).rem_euclid(1.0), [0.0, 0.05, 0.3, 0.35]);
            tools::add(pose, Joint::Chest, [6.0 * sniff.abs(), 12.0 * sniff, 0.0]);
        }
        Clip::Walk => moves::walk(m, pose, 0.0, 2.0),
        Clip::AttackMelee => bite(m, pose),
        Clip::Hit => {
            let k = tools::hold((m.t / 0.25).clamp(0.0, 1.0), [0.0, 0.2, 0.4, 1.0]);
            tools::shift(pose, Joint::Hips, [0.0, 0.0, -0.05 * k]);
            tools::add(pose, Joint::Chest, [-15.0 * k, 0.0, 0.0]);
        }
        Clip::Death => moves::death(m, pose, &p, Fall::Back),
        Clip::Spawn => moves::spawn(m, pose, &p, Entry::Pop),
        Clip::Evolve => moves::evolve(m, pose, &p, Power::Spin),
        Clip::Victory => moves::victory(m, pose, &p, Cheer::Dance),
        _ => {}
    }
    if m.clip != Clip::Walk {
        stance(pose, &p);
    }
}

/// Rear back (0–0.24 s), lunge so the jaws close on the Strike (0.30 s).
fn bite(m: &Moment, pose: &mut Pose) {
    let rear = tools::window(m.t, 0.0, 0.2) * (1.0 - tools::snap(m.t, 0.24, 0.3));
    let lunge = tools::snap(m.t, 0.24, 0.3) * (1.0 - tools::window(m.t, 0.4, 0.62));
    tools::face(pose, m.aim, Vec3::ZERO);
    tools::shift(
        pose,
        Joint::Hips,
        [0.0, 0.03 * rear, -0.03 * rear + 0.08 * lunge],
    );
    tools::add(pose, Joint::Chest, [-14.0 * rear + 16.0 * lunge, 0.0, 0.0]);
}

/// The legs ride the thighs: keep them upright so every foot lands.
fn stance(pose: &mut Pose, p: &Proportions) {
    tools::plant(pose, p, tools::stance(p, 1.0, 0.0));
}
