//! Rotcap kit: a waddling mushroom that rears back and spits a toxin glob
//! from its mouth along the glob's own arc, rises out of the soil on
//! arrival and wilts back into it on death.

mod model;

use bevy::prelude::*;

use super::common::{moves, moves::*};
use super::{tools, Attack, Gait, Kit, Moment};
use crate::art::{
    anim::{Clip, Pose},
    rig::{Joint, Node},
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
        from: Node::Body(Joint::Head),
        muzzle: |tier, _| model::mouth(&plan(tier)),
        shot: |_, _| ShotKind::Glob,
        flash: |_, _| None,
    },
    evolve: Burst::Bloom,
    durations: moves::DURATIONS,
};

pub fn pose(m: &Moment, pose: &mut Pose) {
    let p = plan(m.tier);
    match m.clip {
        Clip::Idle => {
            moves::idle(m, pose, &p, 0.4);
            // A slow wobble under the heavy cap.
            let w = (m.clock * 1.3).sin();
            tools::add(pose, Joint::Spine, [0.0, 0.0, 4.0 * w]);
            tools::add(pose, Joint::Head, [0.0, 0.0, -3.0 * w]);
            belly(pose, 0.0);
        }
        Clip::Walk => moves::walk(m, pose, 3.0, 7.0),
        Clip::AttackRanged => spit(m, pose, &p),
        Clip::Hit => moves::hit(m, pose, 1.0),
        Clip::Death => moves::death(m, pose, &p, Fall::Sink),
        Clip::Spawn => moves::spawn(m, pose, &p, Entry::Rise),
        Clip::Evolve => moves::evolve(m, pose, &p, Power::Burst),
        Clip::Victory => moves::victory(m, pose, &p, Cheer::Dance),
        _ => {}
    }
}

/// Hands on the belly; `squeeze` presses them in.
fn belly(pose: &mut Pose, squeeze: f32) {
    tools::set(
        pose,
        Joint::UpperArmL,
        [-25.0 - 10.0 * squeeze, 0.0, 18.0 - 8.0 * squeeze],
    );
    tools::set(
        pose,
        Joint::UpperArmR,
        [-25.0 - 10.0 * squeeze, 0.0, -18.0 + 8.0 * squeeze],
    );
    tools::set(pose, Joint::ForearmL, [-70.0, 0.0, 0.0]);
    tools::set(pose, Joint::ForearmR, [-70.0, 0.0, 0.0]);
}

/// Rear back drawing breath (0–0.15 s), snap forward and spit at 0.18 s,
/// shake the cap and settle.
fn spit(m: &Moment, pose: &mut Pose, p: &crate::art::rig::Proportions) {
    let draw = tools::window(m.t, 0.0, 0.15) * (1.0 - tools::snap(m.t, 0.16, 0.19));
    let lunge = tools::snap(m.t, 0.16, 0.19) * (1.0 - tools::window(m.t, 0.3, 0.6));
    tools::face(pose, m.aim, Vec3::ZERO);
    tools::add(pose, Joint::Spine, [-16.0 * draw + 14.0 * lunge, 0.0, 0.0]);
    tools::add(pose, Joint::Chest, [-8.0 * draw + 8.0 * lunge, 0.0, 0.0]);
    tools::add(
        pose,
        Joint::Head,
        [
            -14.0 * draw + 10.0 * lunge,
            0.0,
            6.0 * lunge * (m.t * 40.0).sin(),
        ],
    );
    tools::shift(
        pose,
        Joint::Hips,
        [0.0, -0.03 * draw, -0.03 * draw + 0.04 * lunge],
    );
    belly(pose, draw);
    tools::plant(pose, p, tools::stance(p, 1.3, 0.08));
}
