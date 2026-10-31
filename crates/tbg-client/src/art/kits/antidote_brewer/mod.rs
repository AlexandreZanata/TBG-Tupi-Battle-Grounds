//! Antidote Brewer kit: it swirls its flask while idle, hurries with the
//! flask held up, and tosses the cure at the ally it cleanses (its brew is
//! its attack, so the flask leaves the hand on the cast Release). Pops up
//! out of the soil.

mod model;

use bevy::prelude::*;

use super::common::{moves, moves::*};
use super::{tools, Attack, Gait, Kit, Moment};
use crate::art::{
    anim::{Clip, Pose},
    rig::{Joint, Node, Proportions, Socket},
};
use crate::vfx::{impacts::Burst, shots::ShotKind};
use model::plan;

pub const KIT: Kit = Kit {
    figure: model::spec,
    pose,
    gait: Gait::LIGHT,
    attack: Attack::Shot {
        loaded: None,
        from: Node::Prop(Socket::WeaponR),
        muzzle: |_, _| model::FLASK,
        shot: |_, _| ShotKind::Flask,
        flash: |_, _| None,
    },
    evolve: Burst::Bloom,
    durations: moves::DURATIONS,
};

pub fn pose(m: &Moment, pose: &mut Pose) {
    let p = plan(m.tier);
    match m.clip {
        Clip::Idle => {
            moves::idle(m, pose, &p, 0.1);
            swirl(m, pose, &p);
        }
        Clip::Walk => {
            moves::walk(m, pose, 6.0, 3.0);
            tools::set(pose, Joint::UpperArmR, [-60.0, 0.0, -10.0]);
            tools::set(pose, Joint::ForearmR, [-70.0, 0.0, 0.0]);
        }
        Clip::Cast => moves::toss(m, pose, &p),
        Clip::Hit => moves::hit(m, pose, 1.0),
        Clip::Death => moves::death(m, pose, &p, Fall::Kneel),
        Clip::Spawn => moves::spawn(m, pose, &p, Entry::Pop),
        Clip::Evolve => moves::evolve(m, pose, &p, Power::Raise),
        Clip::Victory => moves::victory(m, pose, &p, Cheer::Raise),
        _ => {}
    }
}

/// The flask held up before the eyes and swirled in small circles.
fn swirl(m: &Moment, pose: &mut Pose, p: &Proportions) {
    let a = m.clock * 3.0;
    let at = Vec3::new(-0.1 + 0.025 * a.cos(), 0.3 + 0.02 * a.sin(), 0.22);
    tools::reach(pose, false, at, None, (p, Vec3::new(-1.0, -0.6, -0.3)));
    tools::add(pose, Joint::Head, [8.0, -10.0, 0.0]);
}
