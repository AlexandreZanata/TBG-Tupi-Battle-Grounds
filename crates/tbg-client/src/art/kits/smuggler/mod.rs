//! Smuggler kit: no basic attack; a shifty idle with darting glances, a
//! sneaking walk, and a toss whenever one of his crates lands (`Cast`,
//! cued from the crate's spawn in `sync/actions.rs`). He pops in as if out
//! of a crate.

mod model;

use bevy::prelude::*;

use super::common::{moves, moves::*, Form};
use super::{tools, Attack, Gait, Kit, Moment};
use crate::art::{
    anim::{Clip, Pose},
    rig::{Joint, Proportions},
};
use crate::vfx::impacts::Burst;
use model::plan;

pub const KIT: Kit = Kit {
    anatomy: None,
    figure: model::spec,
    pose,
    gait: Gait::LIGHT,
    attack: Attack::None,
    evolve: Burst::Valor,
    durations: moves::DURATIONS,
};

pub fn pose(m: &Moment, pose: &mut Pose) {
    let f = Form::of(m.tier, m.branch);
    let p = plan(m.tier);
    match m.clip {
        Clip::Idle => {
            moves::idle(m, pose, &p, 0.1);
            // Darting glances and a coin flipped in the right hand.
            let glance = (m.clock * 0.9).sin().signum() * tools::wave(m.clock, 0.5, 3.0).abs();
            tools::add(pose, Joint::Head, [0.0, 35.0 * glance, 0.0]);
            let flip = ((m.clock * 2.0).sin() * 0.5 + 0.5).powi(3);
            tools::set(pose, Joint::UpperArmR, [-25.0, 0.0, -10.0]);
            tools::set(pose, Joint::ForearmR, [-80.0 - 20.0 * flip, 0.0, 0.0]);
            hands_on_hip(pose, &p);
        }
        Clip::Walk => {
            // A sneaking lean; the runner branch leans harder.
            let lean = if f.b { 14.0 } else { 9.0 };
            moves::walk(m, pose, lean, 4.0);
        }
        Clip::Cast => moves::toss(m, pose, &p),
        Clip::Hit => moves::hit(m, pose, 1.0),
        Clip::Death => moves::death(m, pose, &p, Fall::Forward),
        Clip::Spawn => moves::spawn(m, pose, &p, Entry::Pop),
        Clip::Evolve => moves::evolve(m, pose, &p, Power::Spin),
        Clip::Victory => moves::victory(m, pose, &p, Cheer::Dance),
        _ => {}
    }
}

/// Left fist on the hip.
fn hands_on_hip(pose: &mut Pose, p: &Proportions) {
    tools::reach(
        pose,
        true,
        Vec3::new(0.2, -0.02, 0.04),
        None,
        (p, Vec3::new(1.0, 0.0, -0.5)),
    );
}
