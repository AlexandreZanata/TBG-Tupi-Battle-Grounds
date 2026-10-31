//! Shatterback: a heavy terrestrial turtle with a domed segmented shell,
//! grounded paw strikes and a retreat into its shell on death.

pub(crate) mod model;

use super::common::{moves, moves::*, Form};
use super::{tools, Attack, Gait, Kit, Moment};
use crate::art::{
    anim::{Clip, Pose},
    rig::{Joint, Node},
};
use crate::vfx::impacts::{Burst, Impact};
use model::plan;

pub const KIT: Kit = Kit {
    figure: model::spec,
    pose,
    gait: Gait::HEAVY,
    attack: Attack::Melee {
        from: Node::Body(Joint::HandR),
        tip: |_, _| model::tip(),
        impact: |tier, branch| {
            if Form::of(tier, branch).a {
                Impact::Shards
            } else {
                Impact::Crush
            }
        },
    },
    evolve: Burst::Bloom,
    durations: moves::DURATIONS,
};

pub fn pose(m: &Moment, pose: &mut Pose) {
    let p = plan(m.tier);
    match m.clip {
        Clip::Idle => {
            moves::idle(m, pose, &p, 1.0);
            stoop(pose, 1.0);
            let heave = (m.clock * 1.2).sin();
            tools::add(pose, Joint::Chest, [3.0 * heave, 0.0, 0.0]);
        }
        Clip::Walk => {
            moves::walk(m, pose, 10.0, 6.0);
            stoop(pose, 0.6);
        }
        Clip::AttackMelee => moves::strike(m, pose, &p, THRUST, 1.1),
        Clip::Hit => {
            moves::hit(m, pose, 0.45);
            stoop(pose, 0.8);
        }
        Clip::Death => moves::death(m, pose, &p, Fall::Kneel),
        Clip::Spawn => moves::spawn(m, pose, &p, Entry::Rise),
        Clip::Evolve => moves::evolve(m, pose, &p, Power::Burst),
        Clip::Victory => moves::victory(m, pose, &p, Cheer::Both),
        _ => stoop(pose, 1.0),
    }
}

/// A low turtle stance keeps the domed shell above its short legs.
fn stoop(pose: &mut Pose, w: f32) {
    tools::add(pose, Joint::Spine, [14.0 * w, 0.0, 0.0]);
    tools::add(pose, Joint::Head, [-12.0 * w, 0.0, 0.0]);
    tools::add(pose, Joint::UpperArmL, [-10.0 * w, 0.0, 12.0 * w]);
    tools::add(pose, Joint::UpperArmR, [-10.0 * w, 0.0, -12.0 * w]);
}
