//! Colossus: a monumental stone deity carrying one physical obelisk from back to impact.
mod model;
mod motion;
use super::{Anatomy, Attack, Gait, Kit};
use crate::art::{
    anim::Clip,
    rig::{self, Joint, Node, Socket},
};
use crate::vfx::impacts::{Burst, Impact};
use bevy::prelude::*;
fn parent(n: Node) -> Option<Node> {
    if matches!(n, Node::Prop(Socket::WeaponR)) {
        Some(Node::Body(Joint::Chest))
    } else {
        n.parent()
    }
}
fn rest(n: Node, p: &rig::Proportions) -> Vec3 {
    if matches!(n, Node::Prop(Socket::WeaponR)) {
        Vec3::new(0.0, 0.10, -0.27)
    } else {
        rig::rest(n, p)
    }
}
static ANATOMY: Anatomy = Anatomy {
    rest,
    parent,
    springs: false,
    planted_legs: true,
};
pub const KIT: Kit = Kit {
    anatomy: Some(&ANATOMY),
    figure: model::spec,
    pose: motion::pose,
    gait: Gait {
        stride: 0.70,
        lift: 0.10,
        bob: 0.015,
        sink: 0.06,
        stomp: true,
    },
    attack: Attack::Melee {
        from: Node::Prop(Socket::WeaponR),
        tip: |_, _| Vec3::new(0.0, 1.27, 0.0),
        impact: |t, b| Impact::Champion(tbg_core::UnitKind::Colossus, t, b),
    },
    evolve: Burst::Champion(tbg_core::UnitKind::Colossus),
    durations: &[
        (Clip::AttackMelee, 0.85),
        (Clip::AttackRanged, 0.85),
        (Clip::Cast, 0.85),
        (Clip::Spawn, 1.0),
        (Clip::Evolve, 1.1),
        (Clip::Death, 1.2),
        (Clip::Victory, 3.0),
    ],
};
