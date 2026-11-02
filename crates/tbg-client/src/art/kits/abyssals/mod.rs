//! Eight Abyssal native kits and the Plant replacement for the transferred Trident.
mod abyss_harpooner;
mod common;
mod foam_weaver;
mod motion;
mod nacre_duelist;
mod pearl_slinger;
mod pod_snapper;
mod reef_lancer;
mod trench_fist;
mod trench_lamplighter;
mod undertow_herald;
use super::{Anatomy, Attack, Gait, Kit};
use crate::art::{
    anim::Clip,
    rig::{self, Node, Socket},
};
use crate::vfx::{
    impacts::{Burst, Impact},
    shots::ShotKind,
};
use bevy::prelude::*;
use tbg_core::UnitKind;
fn parent(n: Node) -> Option<Node> {
    match n {
        Node::Prop(Socket::StringTop | Socket::StringBottom | Socket::Nock) => {
            Some(Node::Prop(Socket::WeaponR))
        }
        _ => n.parent(),
    }
}
static ANATOMY: Anatomy = Anatomy {
    rest: rig::rest,
    parent,
    springs: false,
    planted_legs: true,
};
const DURATIONS: &[(Clip, f32)] = &[
    (Clip::AttackMelee, 0.62),
    (Clip::AttackRanged, 0.62),
    (Clip::Cast, 0.62),
    (Clip::Spawn, 0.65),
    (Clip::Evolve, 0.9),
    (Clip::Death, 0.9),
    (Clip::Victory, 3.0),
];
static NACRE_DUELIST: Kit = Kit {
    anatomy: Some(&ANATOMY),
    figure: nacre_duelist::spec,
    pose: |m, p| motion::pose(UnitKind::NacreDuelist, m, p),
    gait: Gait {
        stride: 0.99,
        lift: 0.13,
        bob: 0.012,
        sink: 0.08,
        stomp: false,
    },
    attack: Attack::Melee {
        from: Node::Prop(Socket::WeaponR),
        tip: |_, _| Vec3::new(0.0, 0.0, 0.25),
        impact: |t, b| Impact::Champion(UnitKind::NacreDuelist, t, b),
    },
    evolve: Burst::Champion(UnitKind::NacreDuelist),
    durations: DURATIONS,
};
static REEF_LANCER: Kit = Kit {
    anatomy: Some(&ANATOMY),
    figure: reef_lancer::spec,
    pose: |m, p| motion::pose(UnitKind::ReefLancer, m, p),
    gait: Gait {
        stride: 0.96,
        lift: 0.12,
        bob: 0.012,
        sink: 0.08,
        stomp: false,
    },
    attack: Attack::Melee {
        from: Node::Prop(Socket::WeaponR),
        tip: |_, _| Vec3::new(0.0, 0.0, 0.25),
        impact: |t, b| Impact::Champion(UnitKind::ReefLancer, t, b),
    },
    evolve: Burst::Champion(UnitKind::ReefLancer),
    durations: DURATIONS,
};
static TRENCH_FIST: Kit = Kit {
    anatomy: Some(&ANATOMY),
    figure: trench_fist::spec,
    pose: |m, p| motion::pose(UnitKind::TrenchFist, m, p),
    gait: Gait {
        stride: 0.78,
        lift: 0.16,
        bob: 0.02,
        sink: 0.08,
        stomp: true,
    },
    attack: Attack::Melee {
        from: Node::Prop(Socket::WeaponR),
        tip: |_, _| Vec3::new(0.0, 0.0, 0.25),
        impact: |t, b| Impact::Champion(UnitKind::TrenchFist, t, b),
    },
    evolve: Burst::Champion(UnitKind::TrenchFist),
    durations: DURATIONS,
};
static PEARL_SLINGER: Kit = Kit {
    anatomy: Some(&ANATOMY),
    figure: pearl_slinger::spec,
    pose: |m, p| motion::pose(UnitKind::PearlSlinger, m, p),
    gait: Gait {
        stride: 0.8,
        lift: 0.18,
        bob: 0.018,
        sink: 0.08,
        stomp: false,
    },
    attack: Attack::Shot {
        from: Node::Prop(Socket::WeaponR),
        muzzle: |_, _| Vec3::new(0.0, 0.12, 0.28),
        shot: |t, b| ShotKind::Abyssal(UnitKind::PearlSlinger, t, b),
        flash: |_, _| None,
        loaded: None,
    },
    evolve: Burst::Champion(UnitKind::PearlSlinger),
    durations: DURATIONS,
};
static ABYSS_HARPOONER: Kit = Kit {
    anatomy: Some(&ANATOMY),
    figure: abyss_harpooner::spec,
    pose: |m, p| motion::pose(UnitKind::AbyssHarpooner, m, p),
    gait: Gait {
        stride: 0.92,
        lift: 0.12,
        bob: 0.012,
        sink: 0.08,
        stomp: false,
    },
    attack: Attack::Shot {
        from: Node::Prop(Socket::WeaponR),
        muzzle: |_, _| Vec3::new(0.0, 0.12, 0.28),
        shot: |t, b| ShotKind::Abyssal(UnitKind::AbyssHarpooner, t, b),
        flash: |_, _| None,
        loaded: None,
    },
    evolve: Burst::Champion(UnitKind::AbyssHarpooner),
    durations: DURATIONS,
};
static UNDERTOW_HERALD: Kit = Kit {
    anatomy: Some(&ANATOMY),
    figure: undertow_herald::spec,
    pose: |m, p| motion::pose(UnitKind::UndertowHerald, m, p),
    gait: Gait {
        stride: 0.82,
        lift: 0.12,
        bob: 0.016,
        sink: 0.08,
        stomp: false,
    },
    attack: Attack::Shot {
        from: Node::Prop(Socket::WeaponR),
        muzzle: |_, _| Vec3::new(0.0, 0.12, 0.28),
        shot: |t, b| ShotKind::Abyssal(UnitKind::UndertowHerald, t, b),
        flash: |_, _| None,
        loaded: None,
    },
    evolve: Burst::Champion(UnitKind::UndertowHerald),
    durations: DURATIONS,
};
static TRENCH_LAMPLIGHTER: Kit = Kit {
    anatomy: Some(&ANATOMY),
    figure: trench_lamplighter::spec,
    pose: |m, p| motion::pose(UnitKind::TrenchLamplighter, m, p),
    gait: Gait {
        stride: 0.92,
        lift: 0.11,
        bob: 0.008,
        sink: 0.08,
        stomp: false,
    },
    attack: Attack::Shot {
        from: Node::Prop(Socket::WeaponR),
        muzzle: |_, _| Vec3::new(0.0, 0.12, 0.28),
        shot: |t, b| ShotKind::Abyssal(UnitKind::TrenchLamplighter, t, b),
        flash: |_, _| None,
        loaded: None,
    },
    evolve: Burst::Champion(UnitKind::TrenchLamplighter),
    durations: DURATIONS,
};
static FOAM_WEAVER: Kit = Kit {
    anatomy: Some(&ANATOMY),
    figure: foam_weaver::spec,
    pose: |m, p| motion::pose(UnitKind::FoamWeaver, m, p),
    gait: Gait {
        stride: 0.9,
        lift: 0.1,
        bob: 0.01,
        sink: 0.08,
        stomp: false,
    },
    attack: Attack::Shot {
        from: Node::Prop(Socket::WeaponR),
        muzzle: |_, _| Vec3::new(0.0, 0.12, 0.28),
        shot: |t, b| ShotKind::Abyssal(UnitKind::FoamWeaver, t, b),
        flash: |_, _| None,
        loaded: None,
    },
    evolve: Burst::Champion(UnitKind::FoamWeaver),
    durations: DURATIONS,
};
static POD_SNAPPER: Kit = Kit {
    anatomy: Some(&ANATOMY),
    figure: pod_snapper::spec,
    pose: |m, p| motion::pose(UnitKind::PodSnapper, m, p),
    gait: Gait {
        stride: 0.74,
        lift: 0.18,
        bob: 0.02,
        sink: 0.08,
        stomp: false,
    },
    attack: Attack::Shot {
        from: Node::Prop(Socket::WeaponR),
        muzzle: |_, _| Vec3::new(0.0, 0.12, 0.28),
        shot: |t, b| ShotKind::Abyssal(UnitKind::PodSnapper, t, b),
        flash: |_, _| None,
        loaded: None,
    },
    evolve: Burst::Champion(UnitKind::PodSnapper),
    durations: DURATIONS,
};
pub fn kit(k: UnitKind) -> Option<&'static Kit> {
    match k {
        UnitKind::NacreDuelist => Some(&NACRE_DUELIST),
        UnitKind::ReefLancer => Some(&REEF_LANCER),
        UnitKind::TrenchFist => Some(&TRENCH_FIST),
        UnitKind::PearlSlinger => Some(&PEARL_SLINGER),
        UnitKind::AbyssHarpooner => Some(&ABYSS_HARPOONER),
        UnitKind::UndertowHerald => Some(&UNDERTOW_HERALD),
        UnitKind::TrenchLamplighter => Some(&TRENCH_LAMPLIGHTER),
        UnitKind::FoamWeaver => Some(&FOAM_WEAVER),
        UnitKind::PodSnapper => Some(&POD_SNAPPER),
        _ => None,
    }
}
pub fn contains(k: UnitKind) -> bool {
    kit(k).is_some()
}
pub fn radius(k: UnitKind, t: u8) -> Option<f32> {
    contains(k).then_some(
        (if k == UnitKind::TrenchFist {
            0.54
        } else {
            0.48
        }) * (1.0 + 0.08 * f32::from(t.min(2))),
    )
}
