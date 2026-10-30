//! Cargo Crab kit: a scuttling carrier. Pincers ride forward and snap now
//! and then, the walk is a quick low scuttle, the attack is a pincer
//! lunge, it flips onto its back on death and pops out on arrival.

mod model;

use super::common::{moves, moves::*};
use super::{tools, Attack, Gait, Kit, Moment};
use crate::art::{
    anim::{Clip, Pose},
    rig::{Joint, Node},
};
use crate::vfx::impacts::{Burst, Impact};
use model::plan;

/// Many short, quick steps close to the ground.
const SCUTTLE: Gait = Gait {
    stride: 0.9,
    lift: 0.3,
    bob: 0.012,
    sink: 0.04,
    stomp: false,
};

pub const KIT: Kit = Kit {
    figure: model::spec,
    pose,
    gait: SCUTTLE,
    attack: Attack::Melee {
        from: Node::Body(Joint::HandR),
        tip: |_, _| model::tip(),
        impact: |_, _| Impact::Bite,
    },
    evolve: Burst::Forge,
    durations: moves::DURATIONS,
};

pub fn pose(m: &Moment, pose: &mut Pose) {
    let p = plan(m.tier);
    match m.clip {
        Clip::Idle => {
            moves::idle(m, pose, &p, 0.4);
            let snip = tools::hold((m.clock * 0.4).rem_euclid(1.0), [0.0, 0.04, 0.08, 0.14]);
            claws(pose, 1.0, snip);
        }
        Clip::Walk => {
            moves::walk(m, pose, 0.0, 3.0);
            claws(pose, 1.0, 0.0);
        }
        Clip::AttackMelee => {
            moves::strike(m, pose, &p, THRUST, 1.0);
            let k = (m.t / ATTACK).clamp(0.0, 1.0);
            tools::set(
                pose,
                Joint::UpperArmL,
                [
                    -75.0 - 30.0 * tools::hold(k, [0.2, 0.4, 0.6, 0.9]),
                    0.0,
                    22.0,
                ],
            );
            tools::set(pose, Joint::ForearmL, [-30.0, 0.0, 0.0]);
        }
        Clip::Hit => {
            moves::hit(m, pose, 0.6);
            claws(pose, 0.7, 0.0);
        }
        Clip::Death => {
            moves::death(m, pose, &p, Fall::Back);
            claws(pose, 1.0, 1.0);
        }
        Clip::Spawn => {
            moves::spawn(m, pose, &p, Entry::Pop);
            claws(pose, 1.0, 0.0);
        }
        Clip::Evolve => {
            moves::evolve(m, pose, &p, Power::Spin);
            claws(pose, 1.3, 0.0);
        }
        Clip::Victory => {
            moves::victory(m, pose, &p, Cheer::Both);
            claws(pose, 1.9, (m.t * 6.0).sin() * 0.5 + 0.5);
        }
        _ => claws(pose, 1.0, 0.0),
    }
    if m.clip != Clip::Walk {
        // The legs ride the thighs: keep them upright so every foot lands.
        tools::plant(pose, &p, tools::stance(&p, 1.0, 0.0));
    }
}

/// Pincers forward (`up` scales how high they are held), `snip` dips them
/// for a snap. Set, not added, so both claws match whatever the clip did.
fn claws(pose: &mut Pose, up: f32, snip: f32) {
    tools::set(
        pose,
        Joint::UpperArmL,
        [-70.0 * up + 15.0 * snip, 0.0, 12.0],
    );
    tools::set(
        pose,
        Joint::UpperArmR,
        [-70.0 * up + 15.0 * snip, 0.0, -12.0],
    );
    tools::set(pose, Joint::ForearmL, [-35.0, 0.0, 0.0]);
    tools::set(pose, Joint::ForearmR, [-35.0, 0.0, 0.0]);
}
