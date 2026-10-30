//! Bloodweaver kit: no basic attack. She idles weaving slow circles with
//! her free hand, glides on the walk, and raises both hands to draw blood
//! from one ally into another (`Cast`, cued from the transfer). She
//! rises out of the ground and sinks back into it.

mod model;

use bevy::prelude::*;

use super::common::{moves, moves::*};
use super::{tools, Attack, Gait, Kit, Moment};
use crate::art::{
    anim::{Clip, Pose},
    rig::Proportions,
};
use crate::vfx::impacts::Burst;
use model::plan;

pub const KIT: Kit = Kit {
    figure: model::spec,
    pose,
    gait: Gait::LIGHT,
    attack: Attack::None,
    evolve: Burst::Soul,
    durations: moves::DURATIONS,
};

pub fn pose(m: &Moment, pose: &mut Pose) {
    let p = plan(m.tier);
    match m.clip {
        Clip::Idle => {
            moves::idle(m, pose, &p, 0.1);
            staff(pose, &p);
            let a = m.clock * 1.6;
            let at = Vec3::new(0.2 + 0.06 * a.cos(), 0.22 + 0.06 * a.sin(), 0.24);
            tools::reach(pose, true, at, None, (&p, Vec3::new(1.0, -0.6, -0.3)));
        }
        Clip::Walk => {
            moves::walk(m, pose, 3.0, 2.0);
            staff(pose, &p);
        }
        Clip::Cast => moves::strike(m, pose, &p, CAST, 1.1),
        Clip::Hit => moves::hit(m, pose, 1.0),
        Clip::Death => moves::death(m, pose, &p, Fall::Sink),
        Clip::Spawn => moves::spawn(m, pose, &p, Entry::Rise),
        Clip::Evolve => moves::evolve(m, pose, &p, Power::Spin),
        Clip::Victory => moves::victory(m, pose, &p, Cheer::Both),
        _ => staff(pose, &p),
    }
}

/// The spindle staff held upright at the right side.
fn staff(pose: &mut Pose, p: &Proportions) {
    tools::reach(
        pose,
        false,
        Vec3::new(-0.24, 0.08, 0.12),
        Some(Quat::IDENTITY),
        (p, Vec3::new(-1.0, -0.5, -0.4)),
    );
}
