//! Blood Furnace kit: a burning skeleton knight. Its fire heaves while it
//! idles, it marches with the hammer at its side, brings the hammer down
//! two-handed in a burning blow, and crumbles to ash on death.

mod model;
mod motion;

use super::{Attack, Gait, Kit};
use crate::art::{
    anim::Clip,
    rig::{Node, Socket},
};
use crate::vfx::impacts::{Burst, Impact};

pub const KIT: Kit = Kit {
    anatomy: None,
    figure: model::spec,
    pose: motion::pose,
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
