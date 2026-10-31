//! Hexer kit: a hunched witch who leans on her skull staff, hobbles on the
//! walk, and thrusts the staff at her target so the hex leaves the skull's
//! mouth on the cast Release; she crumbles into the ground on death.

mod model;

use super::common::{moves, moves::*};
use super::{tools, Attack, Gait, Kit, Moment};
use crate::art::{
    anim::{Clip, Pose},
    rig::{Joint, Node, Socket},
};
use crate::vfx::{impacts::Burst, shots::ShotKind};
use model::plan;

pub const KIT: Kit = Kit {
    anatomy: None,
    figure: model::spec,
    pose,
    gait: Gait::LIGHT,
    attack: Attack::Shot {
        loaded: None,
        from: Node::Prop(Socket::WeaponR),
        muzzle: |_, _| model::muzzle(),
        shot: |_, _| ShotKind::Hex,
        flash: |_, _| None,
    },
    evolve: Burst::Soul,
    durations: moves::DURATIONS,
};

pub fn pose(m: &Moment, pose: &mut Pose) {
    let p = plan(m.tier);
    match m.clip {
        Clip::Idle => {
            moves::idle(m, pose, &p, 0.2);
            hunch(pose, 1.0);
            moves::staff(pose, &p, 0.0, 0.0, 0.0);
            // A crooked claw that beckons now and then.
            let beckon = tools::hold((m.clock * 0.3).rem_euclid(1.0), [0.0, 0.08, 0.16, 0.24]);
            tools::set(pose, Joint::UpperArmL, [-40.0 - 30.0 * beckon, 0.0, 15.0]);
            tools::set(
                pose,
                Joint::ForearmL,
                [
                    -60.0 - 40.0 * beckon * (m.clock * 14.0).sin().abs(),
                    0.0,
                    0.0,
                ],
            );
        }
        Clip::Walk => {
            moves::walk(m, pose, 10.0, 6.0);
            hunch(pose, 0.6);
            moves::staff(pose, &p, 10.0, 0.0, 0.05);
        }
        Clip::Cast => moves::staff_cast(m, pose, &p),
        Clip::Hit => moves::hit(m, pose, 1.0),
        Clip::Death => moves::death(m, pose, &p, Fall::Sink),
        Clip::Spawn => moves::spawn(m, pose, &p, Entry::Rise),
        Clip::Evolve => moves::evolve(m, pose, &p, Power::Spin),
        Clip::Victory => moves::victory(m, pose, &p, Cheer::Dance),
        _ => moves::staff(pose, &p, 0.0, 0.0, 0.0),
    }
}

fn hunch(pose: &mut Pose, w: f32) {
    tools::add(pose, Joint::Spine, [16.0 * w, 0.0, 0.0]);
    tools::add(pose, Joint::Head, [-10.0 * w, 0.0, 0.0]);
}
