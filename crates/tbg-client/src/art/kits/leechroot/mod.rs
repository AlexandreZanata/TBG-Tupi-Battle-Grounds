//! Leechroot kit: a hunched root-beast whose maw weaves as it hunts. It
//! lashes its whip root across the target, draining it, scuttles low on
//! the walk, claws up out of the soil and collapses back into it.

mod model;

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
    gait: Gait::LIGHT,
    attack: Attack::Melee {
        from: Node::Body(Joint::HandR),
        tip: |tier, _| model::tip(&plan(tier)),
        impact: |tier, branch| {
            if Form::of(tier, branch).a {
                Impact::Bite
            } else {
                Impact::Drain
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
            moves::idle(m, pose, &p, 0.2);
            hunch(pose, 1.0);
            // The maw weaves, searching.
            tools::add(
                pose,
                Joint::Head,
                [
                    8.0 * tools::wave(m.clock, 1.1, 0.3),
                    25.0 * tools::wave(m.clock, 0.7, 1.7),
                    10.0 * tools::wave(m.clock, 0.9, 2.4),
                ],
            );
        }
        Clip::Walk => {
            moves::walk(m, pose, 14.0, 5.0);
            hunch(pose, 0.6);
        }
        Clip::AttackMelee => {
            moves::strike(m, pose, &p, SLASH, 1.3);
            hunch(pose, 0.5);
        }
        Clip::Hit => {
            moves::hit(m, pose, 1.0);
            hunch(pose, 0.5);
        }
        Clip::Death => moves::death(m, pose, &p, Fall::Sink),
        Clip::Spawn => moves::spawn(m, pose, &p, Entry::Rise),
        Clip::Evolve => moves::evolve(m, pose, &p, Power::Burst),
        Clip::Victory => moves::victory(m, pose, &p, Cheer::Both),
        _ => hunch(pose, 1.0),
    }
}

/// A predatory crouch: back bent, head thrust forward.
fn hunch(pose: &mut Pose, w: f32) {
    tools::add(pose, Joint::Spine, [24.0 * w, 0.0, 0.0]);
    tools::add(pose, Joint::Head, [-14.0 * w, 0.0, 0.0]);
    tools::shift(pose, Joint::Hips, [0.0, -0.03 * w, 0.0]);
}
