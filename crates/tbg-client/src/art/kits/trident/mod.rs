//! Trident's shark anatomy, shared weapon anchors and two-handed spear performance.

mod model;
pub(crate) mod motion;
mod weapon;

#[cfg(test)]
mod tests;

use super::{common::moves, Anatomy, Attack, Gait, Kit};
use crate::{
    art::rig::{Node::Prop, Socket},
    vfx::impacts::{Burst, Impact},
};

static ANATOMY: Anatomy = Anatomy {
    rest: model::rest,
    parent: model::parent,
    springs: false,
    planted_legs: true,
};

pub const KIT: Kit = Kit {
    anatomy: Some(&ANATOMY),
    figure: model::spec,
    pose: motion::pose,
    gait: Gait::HEAVY,
    attack: Attack::Melee {
        from: Prop(Socket::WeaponR),
        tip: |_, _| weapon::TIP,
        impact: |_, _| Impact::Bite,
    },
    evolve: Burst::Bloom,
    durations: moves::DURATIONS,
};
