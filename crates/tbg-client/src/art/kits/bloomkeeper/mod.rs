//! Bloomkeeper kit: a dryad who lifts a blessing hand while idle, walks
//! lightly with her staff, and thrusts the flower at the ally she heals so
//! the petals leave it on the cast Release; she sinks to her knees on
//! death and grows up out of the soil on arrival.

mod model;

use bevy::prelude::*;

use super::common::{moves, moves::*};
use super::{tools, Attack, Gait, Kit, Moment};
use crate::art::{
    anim::{Clip, Pose},
    rig::{Node, Socket},
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
        shot: |_, _| ShotKind::Petal,
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
            moves::staff(pose, &p, 0.0, 0.0, 0.0);
            let bless = (m.clock * 0.9).sin() * 0.5 + 0.5;
            let palm = Vec3::new(0.2, 0.2 + 0.05 * bless, 0.22);
            tools::reach(pose, true, palm, None, (&p, Vec3::new(1.0, -0.6, -0.3)));
        }
        Clip::Walk => {
            moves::walk(m, pose, 2.0, 3.0);
            moves::staff(pose, &p, 8.0, 0.0, 0.04);
        }
        Clip::Cast => moves::staff_cast(m, pose, &p),
        Clip::Hit => moves::hit(m, pose, 1.0),
        Clip::Death => moves::death(m, pose, &p, Fall::Kneel),
        Clip::Spawn => moves::spawn(m, pose, &p, Entry::Rise),
        Clip::Evolve => moves::evolve(m, pose, &p, Power::Spin),
        Clip::Victory => moves::victory(m, pose, &p, Cheer::Dance),
        _ => moves::staff(pose, &p, 0.0, 0.0, 0.0),
    }
}
