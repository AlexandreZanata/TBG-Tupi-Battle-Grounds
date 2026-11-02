//! Ogre expansion kits and Machine replacement, sharing the native rig and event timing.
mod anatomy;
mod basalt_bulwark;
mod cinder_maul;
mod clay_binder;
mod common;
mod flint_runner;
mod gust_drummer;
mod hide_mender;
mod hook_brawler;
mod motion;
mod pebble_slinger;
mod rivet_ballista;
use super::{Anatomy, Attack, Gait, Kit};
use crate::art::{
    anim::Clip,
    rig::{Node, Socket},
};
use crate::vfx::{
    impacts::{Burst, Impact},
    shots::ShotKind,
};
use bevy::prelude::*;
use tbg_core::UnitKind;
static ANATOMY: Anatomy = Anatomy {
    rest: crate::art::rig::rest,
    parent: Node::parent,
    springs: false,
    planted_legs: true,
};
static BARREL_ANATOMY: Anatomy = Anatomy {
    rest: gust_drummer::rest,
    parent: gust_drummer::parent,
    springs: false,
    planted_legs: true,
};
const DURATIONS: &[(Clip, f32)] = &[
    (Clip::AttackMelee, 0.62),
    (Clip::AttackRanged, 0.62),
    (Clip::Cast, 0.62),
    (Clip::Spawn, 0.6),
    (Clip::Evolve, 0.9),
    (Clip::Death, 0.9),
    (Clip::Victory, 3.0),
];
static BASALT_BULWARK: Kit = Kit {
    anatomy: Some(&ANATOMY),
    figure: basalt_bulwark::spec,
    pose: |m, p| motion::pose(UnitKind::BasaltBulwark, m, p),
    gait: Gait {
        stride: 0.82,
        lift: 0.16,
        bob: 0.018,
        sink: 0.09,
        stomp: true,
    },
    attack: Attack::Melee {
        from: Node::Prop(Socket::WeaponR),
        tip: |_, _| Vec3::new(0.0, 0.0, 0.25),
        impact: |t, b| Impact::Champion(UnitKind::BasaltBulwark, t, b),
    },
    evolve: Burst::Champion(UnitKind::BasaltBulwark),
    durations: DURATIONS,
};
static CINDER_MAUL: Kit = Kit {
    anatomy: Some(&ANATOMY),
    figure: cinder_maul::spec,
    pose: |m, p| motion::pose(UnitKind::CinderMaul, m, p),
    gait: Gait {
        stride: 0.845,
        lift: 0.16,
        bob: 0.018,
        sink: 0.09,
        stomp: false,
    },
    attack: Attack::Melee {
        from: Node::Prop(Socket::WeaponR),
        tip: |_, _| Vec3::new(0.0, 0.0, 0.25),
        impact: |t, b| Impact::Champion(UnitKind::CinderMaul, t, b),
    },
    evolve: Burst::Champion(UnitKind::CinderMaul),
    durations: DURATIONS,
};
static HOOK_BRAWLER: Kit = Kit {
    anatomy: Some(&ANATOMY),
    figure: hook_brawler::spec,
    pose: |m, p| motion::pose(UnitKind::HookBrawler, m, p),
    gait: Gait {
        stride: 0.87,
        lift: 0.16,
        bob: 0.018,
        sink: 0.09,
        stomp: false,
    },
    attack: Attack::Melee {
        from: Node::Prop(Socket::WeaponR),
        tip: |_, _| Vec3::new(0.0, 0.0, 0.25),
        impact: |t, b| Impact::Champion(UnitKind::HookBrawler, t, b),
    },
    evolve: Burst::Champion(UnitKind::HookBrawler),
    durations: DURATIONS,
};
static FLINT_RUNNER: Kit = Kit {
    anatomy: Some(&ANATOMY),
    figure: flint_runner::spec,
    pose: |m, p| motion::pose(UnitKind::FlintRunner, m, p),
    gait: Gait {
        stride: 0.895,
        lift: 0.16,
        bob: 0.018,
        sink: 0.09,
        stomp: false,
    },
    attack: Attack::Melee {
        from: Node::Prop(Socket::WeaponR),
        tip: |_, _| Vec3::new(0.0, 0.0, 0.25),
        impact: |t, b| Impact::Champion(UnitKind::FlintRunner, t, b),
    },
    evolve: Burst::Champion(UnitKind::FlintRunner),
    durations: DURATIONS,
};
static CLAY_BINDER: Kit = Kit {
    anatomy: Some(&ANATOMY),
    figure: clay_binder::spec,
    pose: |m, p| motion::pose(UnitKind::ClayBinder, m, p),
    gait: Gait {
        stride: 0.9199999999999999,
        lift: 0.16,
        bob: 0.018,
        sink: 0.09,
        stomp: false,
    },
    attack: Attack::Shot {
        from: Node::Prop(Socket::WeaponR),
        muzzle: |_, _| Vec3::new(0.0, 0.0, 0.27),
        shot: |t, b| ShotKind::Ogre(UnitKind::ClayBinder, t, b),
        flash: |_, _| None,
        loaded: None,
    },
    evolve: Burst::Champion(UnitKind::ClayBinder),
    durations: DURATIONS,
};
static GUST_DRUMMER: Kit = Kit {
    anatomy: Some(&BARREL_ANATOMY),
    figure: gust_drummer::spec,
    pose: |m, p| motion::pose(UnitKind::GustDrummer, m, p),
    gait: Gait {
        stride: 0.945,
        lift: 0.16,
        bob: 0.018,
        sink: 0.09,
        stomp: false,
    },
    attack: Attack::None,
    evolve: Burst::Champion(UnitKind::GustDrummer),
    durations: DURATIONS,
};
static HIDE_MENDER: Kit = Kit {
    anatomy: Some(&ANATOMY),
    figure: hide_mender::spec,
    pose: |m, p| motion::pose(UnitKind::HideMender, m, p),
    gait: Gait {
        stride: 0.97,
        lift: 0.16,
        bob: 0.018,
        sink: 0.09,
        stomp: false,
    },
    attack: Attack::Shot {
        from: Node::Prop(Socket::WeaponR),
        muzzle: |t, b| Vec3::new(0.0, 0.0, if t > 0 && b == 0 { 0.40 } else { 0.33 }),
        shot: |t, b| ShotKind::Ogre(UnitKind::HideMender, t, b),
        flash: |_, _| None,
        loaded: None,
    },
    evolve: Burst::Champion(UnitKind::HideMender),
    durations: DURATIONS,
};
static PEBBLE_SLINGER: Kit = Kit {
    anatomy: Some(&ANATOMY),
    figure: pebble_slinger::spec,
    pose: |m, p| motion::pose(UnitKind::PebbleSlinger, m, p),
    gait: Gait {
        stride: 0.995,
        lift: 0.16,
        bob: 0.018,
        sink: 0.09,
        stomp: false,
    },
    attack: Attack::Shot {
        from: Node::Prop(Socket::WeaponR),
        muzzle: |_, _| Vec3::new(0.0, 0.25, 0.03),
        shot: |t, b| ShotKind::Ogre(UnitKind::PebbleSlinger, t, b),
        flash: |_, _| None,
        loaded: None,
    },
    evolve: Burst::Champion(UnitKind::PebbleSlinger),
    durations: DURATIONS,
};
static RIVET_BALLISTA: Kit = Kit {
    anatomy: Some(&ANATOMY),
    figure: rivet_ballista::spec,
    pose: |m, p| motion::pose(UnitKind::RivetBallista, m, p),
    gait: Gait {
        stride: 1.02,
        lift: 0.16,
        bob: 0.018,
        sink: 0.09,
        stomp: false,
    },
    attack: Attack::Shot {
        from: Node::Prop(Socket::WeaponR),
        muzzle: |t, b| Vec3::new(0.0, 0.0, if t > 0 && b == 0 { 0.42 } else { 0.35 }),
        shot: |t, b| ShotKind::Ogre(UnitKind::RivetBallista, t, b),
        flash: |_, _| None,
        loaded: None,
    },
    evolve: Burst::Champion(UnitKind::RivetBallista),
    durations: DURATIONS,
};
pub fn kit(k: UnitKind) -> Option<&'static Kit> {
    match k {
        UnitKind::BasaltBulwark => Some(&BASALT_BULWARK),
        UnitKind::CinderMaul => Some(&CINDER_MAUL),
        UnitKind::HookBrawler => Some(&HOOK_BRAWLER),
        UnitKind::FlintRunner => Some(&FLINT_RUNNER),
        UnitKind::ClayBinder => Some(&CLAY_BINDER),
        UnitKind::GustDrummer => Some(&GUST_DRUMMER),
        UnitKind::HideMender => Some(&HIDE_MENDER),
        UnitKind::PebbleSlinger => Some(&PEBBLE_SLINGER),
        UnitKind::RivetBallista => Some(&RIVET_BALLISTA),
        _ => None,
    }
}
pub fn contains(k: UnitKind) -> bool {
    kit(k).is_some()
}
pub fn plan(k: UnitKind, t: u8) -> crate::art::rig::Proportions {
    match k {
        UnitKind::BasaltBulwark => basalt_bulwark::plan(t),
        UnitKind::CinderMaul => cinder_maul::plan(t),
        UnitKind::HookBrawler => hook_brawler::plan(t),
        UnitKind::FlintRunner => flint_runner::plan(t),
        UnitKind::ClayBinder => clay_binder::plan(t),
        UnitKind::GustDrummer => gust_drummer::plan(t),
        UnitKind::HideMender => hide_mender::plan(t),
        UnitKind::PebbleSlinger => pebble_slinger::plan(t),
        UnitKind::RivetBallista => rivet_ballista::plan(t),
        _ => common::body::plan(false, 1.0, t),
    }
}
pub fn radius(k: UnitKind, t: u8) -> Option<f32> {
    let base = match k {
        UnitKind::BasaltBulwark => 0.58,
        UnitKind::CinderMaul => 0.54,
        UnitKind::HookBrawler => 0.5,
        UnitKind::FlintRunner => 0.46,
        UnitKind::ClayBinder => 0.53,
        UnitKind::GustDrummer => 0.56,
        UnitKind::HideMender => 0.5,
        UnitKind::PebbleSlinger => 0.46,
        UnitKind::RivetBallista => 0.48,
        _ => return None,
    };
    Some(base * (1.0 + 0.08 * f32::from(t.min(2))))
}
