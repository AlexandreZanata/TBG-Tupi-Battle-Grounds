//! Astreon: the Rupture God's long verdict lance, two hands on one shaft (GD-04).

mod lance;
mod model;
pub(crate) mod motion;

#[cfg(test)]
pub(crate) use model::plan;

use super::{common::Form, Anatomy, Attack, Gait, Kit};
use crate::{
    art::{
        anim::Clip,
        rig::{Node::Prop, Socket},
    },
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
    gait: Gait {
        stride: 0.8,
        lift: 0.11,
        bob: 0.02,
        sink: 0.06,
        stomp: true,
    },
    attack: Attack::Melee {
        from: Prop(Socket::WeaponR),
        tip: |t, b| lance::tip(Form::of(t, b)),
        impact: |t, b| Impact::Champion(tbg_core::UnitKind::Astreon, t, b),
    },
    evolve: Burst::Champion(tbg_core::UnitKind::Astreon),
    durations: &[
        (Clip::AttackMelee, 0.7),
        (Clip::AttackRanged, 0.7),
        (Clip::Cast, 0.7),
        (Clip::Spawn, 0.7),
        (Clip::Evolve, 1.0),
        (Clip::Death, 1.1),
        (Clip::Victory, 3.0),
    ],
};
