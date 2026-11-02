//! Volthar: the Storm God's rune hammer throws server-counted lightning (GD-04).

mod hammer;
mod model;
pub(crate) mod motion;

use super::{Anatomy, Attack, Gait, Kit};
use crate::{
    art::{
        anim::Clip,
        rig::{Node::Prop, Socket},
    },
    vfx::{impacts::Burst, shots::ShotKind},
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
        stride: 0.75,
        lift: 0.11,
        bob: 0.02,
        sink: 0.06,
        stomp: true,
    },
    attack: Attack::CountedShot {
        from: Prop(Socket::WeaponR),
        muzzle: |_, _| hammer::MUZZLE,
        normal: ShotKind::StormBolt,
        powered: ShotKind::StormSurge,
        every: |_, _| 4,
        loaded: None,
    },
    evolve: Burst::Champion(tbg_core::UnitKind::Volthar),
    durations: &[
        (Clip::AttackMelee, 0.75),
        (Clip::AttackRanged, 0.75),
        (Clip::Cast, 0.75),
        (Clip::Spawn, 0.8),
        (Clip::Evolve, 1.0),
        (Clip::Death, 1.1),
        (Clip::Victory, 2.7),
    ],
};
