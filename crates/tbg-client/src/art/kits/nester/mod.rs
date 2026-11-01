//! Nester kit: a plump nest mother who rocks and pats her belly while
//! idle, waddles on the walk, and lobs a seed overhand on the cast
//! Release; she melts back into the soil on death and rises from it.

mod model;

use bevy::prelude::*;

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
        shot: |_, _| ShotKind::Seed,
        flash: |_, _| None,
    },
    evolve: Burst::Bloom,
    durations: moves::DURATIONS,
};

pub fn pose(m: &Moment, pose: &mut Pose) {
    let p = plan(m.tier);
    match m.clip {
        Clip::Idle => {
            moves::idle(m, pose, &p, 0.5);
            let rock = (m.clock * 1.4).sin();
            tools::add(pose, Joint::Spine, [0.0, 0.0, 5.0 * rock]);
            let pat = (m.clock * 5.0).sin().max(0.0)
                * tools::hold((m.clock * 0.25).rem_euclid(1.0), [0.0, 0.05, 0.25, 0.3]);
            let belly = Vec3::new(0.1, 0.12 + 0.03 * pat, 0.24);
            tools::reach(pose, true, belly, None, (&p, Vec3::new(1.0, -0.6, -0.2)));
        }
        Clip::Walk => moves::walk(m, pose, 2.0, 9.0),
        Clip::Cast => moves::toss(m, pose, &p),
        Clip::Hit => moves::hit(m, pose, 0.8),
        Clip::Death => moves::death(m, pose, &p, Fall::Sink),
        Clip::Spawn => moves::spawn(m, pose, &p, Entry::Rise),
        Clip::Evolve => moves::evolve(m, pose, &p, Power::Burst),
        Clip::Victory => moves::victory(m, pose, &p, Cheer::Both),
        _ => {}
    }
    let hatch = tools::hold(m.gesture, [0.0, 0.06, 0.18, 0.42]);
    tools::add(pose, Joint::Chest, [12.0 * hatch, 0.0, 0.0]);
    tools::add(pose, Joint::UpperArmL, [-20.0 * hatch, 0.0, 0.0]);
    // The existing back nest swells as the authoritative timer matures.
    tools::shift(pose, Joint::Chest, [0.0, 0.015 * m.growth, 0.0]);
}
