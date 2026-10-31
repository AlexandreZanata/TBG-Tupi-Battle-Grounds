//! Blood Furnace kit: a burning skeleton knight. Its fire heaves while it
//! idles, it marches with the hammer at its side, brings the hammer down
//! two-handed in a burning blow, and crumbles to ash on death.

mod model;

use super::common::{moves, moves::*};
use super::{tools, Attack, Gait, Kit, Moment};
use crate::art::{
    anim::{Clip, Pose},
    rig::{Joint, Node, Socket},
};
use crate::vfx::impacts::{Burst, Impact};
use model::plan;

pub const KIT: Kit = Kit {
    anatomy: None,
    figure: model::spec,
    pose,
    gait: Gait::HEAVY,
    attack: Attack::Melee {
        from: Node::Prop(Socket::WeaponR),
        tip: |_, _| model::tip(),
        impact: |_, _| Impact::Burn,
    },
    evolve: Burst::Soul,
    durations: &[
        (Clip::AttackMelee, 0.52),
        (Clip::Spawn, 0.6),
        (Clip::Evolve, 0.9),
        (Clip::Death, 0.9),
    ],
};

pub fn pose(m: &Moment, pose: &mut Pose) {
    let p = plan(m.tier);
    match m.clip {
        Clip::Idle => {
            moves::idle(m, pose, &p, 0.6);
            let ready = m.aim.map_or(0.0, |a| a.weight);
            moves::guard(pose, 0.3 + 0.7 * ready);
            // The fire heaves the ribcage.
            let flare = (m.clock * 2.6).sin().max(0.0).powi(2);
            tools::add(pose, Joint::Chest, [-4.0 * flare, 0.0, 0.0]);
        }
        Clip::Walk => {
            moves::walk(m, pose, 6.0, 4.0);
            moves::guard(pose, 0.3);
        }
        Clip::AttackMelee => moves::strike(m, pose, &p, CHOP, 1.3),
        Clip::Hit => moves::hit(m, pose, 0.8),
        Clip::Death => moves::death(m, pose, &p, Fall::Sink),
        Clip::Spawn => moves::spawn(m, pose, &p, Entry::Rise),
        Clip::Evolve => moves::evolve(m, pose, &p, Power::Raise),
        Clip::Victory => moves::victory(m, pose, &p, Cheer::Raise),
        _ => moves::guard(pose, 0.3),
    }
}
