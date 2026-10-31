//! Rampart kit: an ogre who plants his wall before him, plods behind it,
//! and punches (or rams) round its edge while keeping it up; he drops to
//! his knees behind it when he falls and lands from his drop with a quake.

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

pub const KIT: Kit = Kit {
    anatomy: None,
    figure: model::spec,
    pose,
    gait: Gait::HEAVY,
    attack: Attack::Melee {
        from: Node::Body(Joint::HandR),
        tip: |_, _| model::tip(),
        impact: |_, _| Impact::Crush,
    },
    evolve: Burst::Forge,
    durations: moves::DURATIONS,
};

pub fn pose(m: &Moment, pose: &mut Pose) {
    let p = plan(m.tier);
    match m.clip {
        Clip::Idle => moves::idle(m, pose, &p, 0.9),
        Clip::Walk => moves::walk(m, pose, 4.0, 5.0),
        Clip::AttackMelee => moves::strike(m, pose, &p, THRUST, 1.4),
        Clip::Hit => moves::hit(m, pose, 0.4),
        Clip::Death => moves::death(m, pose, &p, Fall::Kneel),
        Clip::Spawn => moves::spawn(m, pose, &p, Entry::Drop),
        Clip::Evolve => moves::evolve(m, pose, &p, Power::Burst),
        Clip::Victory => moves::victory(m, pose, &p, Cheer::Raise),
        _ => {}
    }
    if m.clip != Clip::Death {
        hold_wall(pose, &p);
    }
}

/// The left hand stays on the wall's grip whatever the body does.
fn hold_wall(pose: &mut Pose, p: &Proportions) {
    tools::reach(
        pose,
        true,
        model::grip(p),
        None,
        (p, Vec3::new(1.0, -0.6, -0.4)),
    );
}
