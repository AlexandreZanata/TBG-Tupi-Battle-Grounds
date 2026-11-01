//! Seventeen native rig-v2 kits with distinct geometry, life motions and typed attack recipes.
mod blastbeetle;
mod cragclaw;
mod dawnmender;
mod eclipse_dancer;
mod forms;
mod gorgehowl;
mod gravecutter;
mod hushfang;
mod lastlight_cantor;
mod leafcutter;
mod mire_tender;
mod motion;
mod nectar_nurse;
mod needlewing;
mod prismancer;
mod spinehurler;
mod starbreaker;
mod stridulator;
mod sunlancer;
use super::{Anatomy, Attack, Gait, Kit};
use crate::art::{
    anim::Clip,
    rig::{
        Joint, Node,
        Node::{Body, Prop},
        Socket,
    },
};
use crate::vfx::{
    impacts::{Burst, Impact},
    shots::ShotKind,
};
use bevy::prelude::*;
use tbg_core::UnitKind;
const DURATIONS: &[(Clip, f32)] = &[
    (Clip::AttackMelee, 0.62),
    (Clip::AttackRanged, 0.62),
    (Clip::Cast, 0.62),
    (Clip::Spawn, 0.6),
    (Clip::Evolve, 0.9),
    (Clip::Death, 0.9),
    (Clip::Victory, 3.0),
];
static INSECT: Anatomy = Anatomy {
    rest: forms::insect_rest,
    parent: forms::insect_parent,
    springs: false,
    planted_legs: true,
};
static DRAGONFLY: Anatomy = Anatomy {
    rest: forms::dragonfly_rest,
    parent: forms::dragonfly_parent,
    springs: false,
    planted_legs: true,
};
static NEEDLE: Anatomy = Anatomy {
    rest: forms::needle_rest,
    parent: forms::needle_parent,
    springs: false,
    planted_legs: true,
};
static HUMANOID: Anatomy = Anatomy {
    rest: crate::art::rig::rest,
    parent: Node::parent,
    springs: false,
    planted_legs: true,
};
static QUAD: Anatomy = Anatomy {
    rest: hushfang::rest,
    parent: Node::parent,
    springs: false,
    planted_legs: false,
};
static GRAVECUTTER: Kit = Kit {
    anatomy: Some(&HUMANOID),
    figure: gravecutter::spec,
    pose: |m, p| motion::pose(UnitKind::Gravecutter, m, p),
    gait: Gait {
        stride: 1.1,
        lift: 0.2,
        bob: 0.015,
        sink: 0.08,
        stomp: false,
    },
    attack: Attack::Melee {
        from: Prop(Socket::WeaponR),
        tip: |_, _| Vec3::new(0.0, 0.0, 0.30),
        impact: |t, b| Impact::Champion(UnitKind::Gravecutter, t, b),
    },
    evolve: Burst::Champion(UnitKind::Gravecutter),
    durations: DURATIONS,
};
static LASTLIGHTCANTOR: Kit = Kit {
    anatomy: Some(&HUMANOID),
    figure: lastlight_cantor::spec,
    pose: |m, p| motion::pose(UnitKind::LastlightCantor, m, p),
    gait: Gait {
        stride: 0.86,
        lift: 0.16,
        bob: 0.014,
        sink: 0.07,
        stomp: false,
    },
    attack: Attack::Shot {
        from: Prop(Socket::WeaponR),
        muzzle: |_, _| Vec3::new(0.0, 0.0, 0.25),
        shot: |t, b| ShotKind::BellEcho(t, b),
        flash: |_, _| None,
        loaded: None,
    },
    evolve: Burst::Champion(UnitKind::LastlightCantor),
    durations: DURATIONS,
};
static CRAGCLAW: Kit = Kit {
    anatomy: Some(&HUMANOID),
    figure: cragclaw::spec,
    pose: |m, p| motion::pose(UnitKind::Cragclaw, m, p),
    gait: Gait {
        stride: 0.95,
        lift: 0.12,
        bob: 0.025,
        sink: 0.1,
        stomp: false,
    },
    attack: Attack::Melee {
        from: Prop(Socket::WeaponR),
        tip: |_, _| Vec3::new(0.0, 0.0, 0.30),
        impact: |t, b| Impact::Champion(UnitKind::Cragclaw, t, b),
    },
    evolve: Burst::Champion(UnitKind::Cragclaw),
    durations: DURATIONS,
};
static HUSHFANG: Kit = Kit {
    anatomy: Some(&QUAD),
    figure: hushfang::spec,
    pose: |m, p| motion::pose(UnitKind::Hushfang, m, p),
    gait: Gait {
        stride: 1.15,
        lift: 0.22,
        bob: 0.02,
        sink: 0.09,
        stomp: false,
    },
    attack: Attack::Melee {
        from: Body(Joint::Head),
        tip: |_, _| Vec3::new(0.0, 0.0, 0.30),
        impact: |t, b| Impact::Champion(UnitKind::Hushfang, t, b),
    },
    evolve: Burst::Champion(UnitKind::Hushfang),
    durations: DURATIONS,
};
static SPINEHURLER: Kit = Kit {
    anatomy: Some(&NEEDLE),
    figure: spinehurler::spec,
    pose: |m, p| motion::pose(UnitKind::Spinehurler, m, p),
    gait: Gait {
        stride: 0.9,
        lift: 0.18,
        bob: 0.02,
        sink: 0.1,
        stomp: false,
    },
    attack: Attack::Shot {
        from: Prop(Socket::WeaponR),
        muzzle: |_, _| Vec3::new(0.0, 0.0, 0.25),
        shot: |t, b| ShotKind::BoneQuill(t, b),
        flash: |_, _| None,
        loaded: Some((
            Node::Secondary(crate::art::rig::Spring::HeadGear),
            |clip, t| {
                !matches!(clip, Clip::AttackMelee | Clip::AttackRanged | Clip::Cast)
                    || t < 0.18
                    || t > 0.44
            },
        )),
    },
    evolve: Burst::Champion(UnitKind::Spinehurler),
    durations: DURATIONS,
};
static MIRETENDER: Kit = Kit {
    anatomy: Some(&HUMANOID),
    figure: mire_tender::spec,
    pose: |m, p| motion::pose(UnitKind::MireTender, m, p),
    gait: Gait {
        stride: 0.76,
        lift: 0.13,
        bob: 0.02,
        sink: 0.1,
        stomp: false,
    },
    attack: Attack::Shot {
        from: Prop(Socket::WeaponR),
        muzzle: |_, _| Vec3::new(0.0, 0.0, 0.25),
        shot: |t, b| ShotKind::Broth(t, b),
        flash: |_, _| None,
        loaded: None,
    },
    evolve: Burst::Champion(UnitKind::MireTender),
    durations: DURATIONS,
};
static GORGEHOWL: Kit = Kit {
    anatomy: Some(&HUMANOID),
    figure: gorgehowl::spec,
    pose: |m, p| motion::pose(UnitKind::Gorgehowl, m, p),
    gait: Gait {
        stride: 0.95,
        lift: 0.17,
        bob: 0.015,
        sink: 0.09,
        stomp: false,
    },
    attack: Attack::Melee {
        from: Body(Joint::Chest),
        tip: |_, _| Vec3::new(0.0, 0.0, 0.30),
        impact: |t, b| Impact::Champion(UnitKind::Gorgehowl, t, b),
    },
    evolve: Burst::Champion(UnitKind::Gorgehowl),
    durations: DURATIONS,
};
static SUNLANCER: Kit = Kit {
    anatomy: Some(&HUMANOID),
    figure: sunlancer::spec,
    pose: |m, p| motion::pose(UnitKind::Sunlancer, m, p),
    gait: Gait {
        stride: 1.18,
        lift: 0.19,
        bob: 0.018,
        sink: 0.08,
        stomp: false,
    },
    attack: Attack::Melee {
        from: Prop(Socket::WeaponR),
        tip: |_, _| Vec3::new(0.0, 0.0, 0.30),
        impact: |t, b| Impact::Champion(UnitKind::Sunlancer, t, b),
    },
    evolve: Burst::Champion(UnitKind::Sunlancer),
    durations: DURATIONS,
};
static ECLIPSEDANCER: Kit = Kit {
    anatomy: Some(&HUMANOID),
    figure: eclipse_dancer::spec,
    pose: |m, p| motion::pose(UnitKind::EclipseDancer, m, p),
    gait: Gait {
        stride: 1.06,
        lift: 0.24,
        bob: 0.018,
        sink: 0.09,
        stomp: false,
    },
    attack: Attack::Melee {
        from: Prop(Socket::WeaponR),
        tip: |_, _| Vec3::new(0.0, 0.0, 0.30),
        impact: |t, b| Impact::Champion(UnitKind::EclipseDancer, t, b),
    },
    evolve: Burst::Champion(UnitKind::EclipseDancer),
    durations: DURATIONS,
};
static STARBREAKER: Kit = Kit {
    anatomy: Some(&HUMANOID),
    figure: starbreaker::spec,
    pose: |m, p| motion::pose(UnitKind::Starbreaker, m, p),
    gait: Gait {
        stride: 0.9,
        lift: 0.14,
        bob: 0.025,
        sink: 0.1,
        stomp: false,
    },
    attack: Attack::Melee {
        from: Prop(Socket::WeaponR),
        tip: |_, _| Vec3::new(0.0, 0.0, 0.30),
        impact: |t, b| Impact::Champion(UnitKind::Starbreaker, t, b),
    },
    evolve: Burst::Champion(UnitKind::Starbreaker),
    durations: DURATIONS,
};
static DAWNMENDER: Kit = Kit {
    anatomy: Some(&HUMANOID),
    figure: dawnmender::spec,
    pose: |m, p| motion::pose(UnitKind::Dawnmender, m, p),
    gait: Gait {
        stride: 0.84,
        lift: 0.15,
        bob: 0.015,
        sink: 0.08,
        stomp: false,
    },
    attack: Attack::Shot {
        from: Prop(Socket::WeaponR),
        muzzle: |_, _| Vec3::new(0.0, 0.0, 0.25),
        shot: |t, b| ShotKind::DawnThread(t, b),
        flash: |_, _| None,
        loaded: None,
    },
    evolve: Burst::Champion(UnitKind::Dawnmender),
    durations: DURATIONS,
};
static PRISMANCER: Kit = Kit {
    anatomy: Some(&HUMANOID),
    figure: prismancer::spec,
    pose: |m, p| motion::pose(UnitKind::Prismancer, m, p),
    gait: Gait {
        stride: 0.92,
        lift: 0.16,
        bob: 0.014,
        sink: 0.08,
        stomp: false,
    },
    attack: Attack::Shot {
        from: Prop(Socket::WeaponR),
        muzzle: |_, _| Vec3::new(0.0, 0.0, 0.25),
        shot: |t, b| ShotKind::PrismShard(t, b),
        flash: |_, _| None,
        loaded: None,
    },
    evolve: Burst::Champion(UnitKind::Prismancer),
    durations: DURATIONS,
};
static LEAFCUTTER: Kit = Kit {
    anatomy: Some(&INSECT),
    figure: leafcutter::spec,
    pose: |m, p| motion::pose(UnitKind::Leafcutter, m, p),
    gait: Gait {
        stride: 1.02,
        lift: 0.19,
        bob: 0.015,
        sink: 0.09,
        stomp: false,
    },
    attack: Attack::Melee {
        from: Prop(Socket::WeaponR),
        tip: |_, _| Vec3::new(0.0, 0.0, 0.30),
        impact: |t, b| Impact::Champion(UnitKind::Leafcutter, t, b),
    },
    evolve: Burst::Champion(UnitKind::Leafcutter),
    durations: DURATIONS,
};
static BLASTBEETLE: Kit = Kit {
    anatomy: Some(&INSECT),
    figure: blastbeetle::spec,
    pose: |m, p| motion::pose(UnitKind::Blastbeetle, m, p),
    gait: Gait {
        stride: 0.92,
        lift: 0.18,
        bob: 0.025,
        sink: 0.1,
        stomp: false,
    },
    attack: Attack::Melee {
        from: Prop(Socket::WeaponR),
        tip: |_, _| Vec3::new(0.0, 0.0, 0.30),
        impact: |t, b| Impact::Champion(UnitKind::Blastbeetle, t, b),
    },
    evolve: Burst::Champion(UnitKind::Blastbeetle),
    durations: DURATIONS,
};
static NEEDLEWING: Kit = Kit {
    anatomy: Some(&DRAGONFLY),
    figure: needlewing::spec,
    pose: |m, p| motion::pose(UnitKind::Needlewing, m, p),
    gait: Gait {
        stride: 1.0,
        lift: 0.14,
        bob: 0.012,
        sink: 0.08,
        stomp: false,
    },
    attack: Attack::Shot {
        from: Body(Joint::HandR),
        muzzle: |_, _| Vec3::new(0.0, 0.0, 0.25),
        shot: |t, b| ShotKind::IvoryNeedle(t, b),
        flash: |_, _| None,
        loaded: Some((
            Node::Secondary(crate::art::rig::Spring::HeadGear),
            |clip, t| {
                !matches!(clip, Clip::AttackMelee | Clip::AttackRanged | Clip::Cast)
                    || t < 0.18
                    || t > 0.44
            },
        )),
    },
    evolve: Burst::Champion(UnitKind::Needlewing),
    durations: DURATIONS,
};
static NECTARNURSE: Kit = Kit {
    anatomy: Some(&INSECT),
    figure: nectar_nurse::spec,
    pose: |m, p| motion::pose(UnitKind::NectarNurse, m, p),
    gait: Gait {
        stride: 0.82,
        lift: 0.16,
        bob: 0.015,
        sink: 0.09,
        stomp: false,
    },
    attack: Attack::None,
    evolve: Burst::Champion(UnitKind::NectarNurse),
    durations: DURATIONS,
};
static STRIDULATOR: Kit = Kit {
    anatomy: Some(&INSECT),
    figure: stridulator::spec,
    pose: |m, p| motion::pose(UnitKind::Stridulator, m, p),
    gait: Gait {
        stride: 0.94,
        lift: 0.17,
        bob: 0.013,
        sink: 0.09,
        stomp: false,
    },
    attack: Attack::Melee {
        from: Body(Joint::Chest),
        tip: |_, _| Vec3::new(0.0, 0.0, 0.30),
        impact: |t, b| Impact::Champion(UnitKind::Stridulator, t, b),
    },
    evolve: Burst::Champion(UnitKind::Stridulator),
    durations: DURATIONS,
};
pub fn kit(k: UnitKind) -> Option<&'static Kit> {
    match k {
        UnitKind::Gravecutter => Some(&GRAVECUTTER),
        UnitKind::LastlightCantor => Some(&LASTLIGHTCANTOR),
        UnitKind::Cragclaw => Some(&CRAGCLAW),
        UnitKind::Hushfang => Some(&HUSHFANG),
        UnitKind::Spinehurler => Some(&SPINEHURLER),
        UnitKind::MireTender => Some(&MIRETENDER),
        UnitKind::Gorgehowl => Some(&GORGEHOWL),
        UnitKind::Sunlancer => Some(&SUNLANCER),
        UnitKind::EclipseDancer => Some(&ECLIPSEDANCER),
        UnitKind::Starbreaker => Some(&STARBREAKER),
        UnitKind::Dawnmender => Some(&DAWNMENDER),
        UnitKind::Prismancer => Some(&PRISMANCER),
        UnitKind::Leafcutter => Some(&LEAFCUTTER),
        UnitKind::Blastbeetle => Some(&BLASTBEETLE),
        UnitKind::Needlewing => Some(&NEEDLEWING),
        UnitKind::NectarNurse => Some(&NECTARNURSE),
        UnitKind::Stridulator => Some(&STRIDULATOR),
        _ => None,
    }
}
pub fn plan(k: UnitKind, t: u8) -> crate::art::rig::Proportions {
    match k {
        UnitKind::Gravecutter => gravecutter::plan(t),
        UnitKind::LastlightCantor => lastlight_cantor::plan(t),
        UnitKind::Cragclaw => cragclaw::plan(t),
        UnitKind::Hushfang => hushfang::plan(t),
        UnitKind::Spinehurler => spinehurler::plan(t),
        UnitKind::MireTender => mire_tender::plan(t),
        UnitKind::Gorgehowl => gorgehowl::plan(t),
        UnitKind::Sunlancer => sunlancer::plan(t),
        UnitKind::EclipseDancer => eclipse_dancer::plan(t),
        UnitKind::Starbreaker => starbreaker::plan(t),
        UnitKind::Dawnmender => dawnmender::plan(t),
        UnitKind::Prismancer => prismancer::plan(t),
        UnitKind::Leafcutter => leafcutter::plan(t),
        UnitKind::Blastbeetle => blastbeetle::plan(t),
        UnitKind::Needlewing => needlewing::plan(t),
        UnitKind::NectarNurse => nectar_nurse::plan(t),
        UnitKind::Stridulator => stridulator::plan(t),
        _ => super::common::body::plan(false, 1.0, t),
    }
}
pub fn is_insect(k: UnitKind) -> bool {
    matches!(
        k,
        UnitKind::Leafcutter
            | UnitKind::Blastbeetle
            | UnitKind::Needlewing
            | UnitKind::NectarNurse
            | UnitKind::Stridulator
    )
}

pub fn auxiliary_rest(n: Node, p: &crate::art::rig::Proportions) -> Vec3 {
    forms::insect_rest(n, p)
}

#[cfg(test)]
mod tests;

pub fn snapshot_pose(
    k: UnitKind,
    m: &super::Moment,
    p: &mut crate::art::anim::Pose,
    f: Option<tbg_core::completion::state::View>,
) {
    let Some(f) = f else {
        return;
    };
    if matches!(
        m.clip,
        Clip::Death | Clip::Victory | Clip::Evolve | Clip::Spawn
    ) {
        return;
    }
    if k == UnitKind::NectarNurse && f.channel_target.is_some() {
        motion::channel_pose(m, p);
    }
    if k == UnitKind::Hushfang && f.charged {
        p[Body(Joint::Head).index()].0.x -= 5.0;
        p[Node::Secondary(crate::art::rig::Spring::Back).index()]
            .0
            .x -= 10.0;
    }
}

/// Measured idle/walk half-width plus reserved clearance; equipment folds between attacks.
pub fn crowd_radius(kind: UnitKind, tier: u8) -> Option<f32> {
    const BASE: [f32; 17] = [
        0.47, 0.46, 0.51, 0.29, 0.47, 0.47, 0.47, 0.46, 0.48, 0.54, 0.42, 0.51, 0.46, 0.37, 0.46,
        0.48, 0.43,
    ];
    let index = (kind as usize).checked_sub(UnitKind::Gravecutter as usize)?;
    BASE.get(index)
        .map(|r| r * (1.0 + 0.08 * f32::from(tier.min(2))))
}
