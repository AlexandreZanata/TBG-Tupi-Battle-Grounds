//! Justiciar's cocked gauntlet basic, independent two-palm light stamp and powering-down collapse.
mod model;
use super::{
    common::{moves, moves::*},
    tools, Anatomy, Attack, Gait, Kit, Moment,
};
use crate::art::{
    anim::{Clip, Pose},
    rig::{
        Joint::*,
        Node::{Body, Secondary},
        Spring,
    },
};
use crate::vfx::{impacts::Burst, shots::ShotKind};
use bevy::prelude::*;
static ANATOMY: Anatomy = Anatomy {
    rest: model::rest,
    parent: model::parent,
    springs: false,
    planted_legs: true,
};
pub const KIT: Kit = Kit {
    anatomy: Some(&ANATOMY),
    figure: model::spec,
    pose,
    gait: Gait {
        stride: 0.85,
        lift: 0.12,
        ..Gait::HEAVY
    },
    attack: Attack::Shot {
        from: Body(HandR),
        muzzle: |_, _| Vec3::new(0.0, 0.0, 0.10),
        shot: |_, _| ShotKind::JudgmentBolt,
        flash: |_, _| None,
        loaded: None,
    },
    evolve: Burst::Valor,
    durations: moves::DURATIONS,
};
fn pose(m: &Moment, p: &mut Pose) {
    let body = model::plan(m.tier);
    match m.clip {
        Clip::Idle => moves::idle(m, p, &body, 0.3),
        Clip::Walk => moves::walk(m, p, 2.0, 1.0),
        Clip::Cast | Clip::AttackRanged | Clip::AttackMelee => {
            tools::set(p, UpperArmL, [-15.0, 0.0, 8.0]);
            tools::set(p, ForearmL, [-15.0, 0.0, 0.0]);
            let cock = tools::hold(m.t, [0.0, 0.13, 0.22, 0.30]);
            let punch = tools::hold(m.t, [0.15, 0.22, 0.30, 0.60]);
            tools::set(p, UpperArmR, [-100.0 * cock - 55.0 * punch, 0.0, -10.0]);
            tools::set(p, ForearmR, [-65.0 * cock, 0.0, 0.0]);
            tools::set(p, Chest, [-6.0 * cock + 8.0 * punch, 0.0, 0.0]);
        }
        Clip::Hit => {
            moves::hit(m, p, 0.5);
            p[Secondary(Spring::Back).index()].1.z -= 0.015;
        }
        Clip::Death => {
            moves::death(m, p, &body, Fall::Kneel);
            p[Secondary(Spring::Back).index()].1.z -= 0.05 * tools::window(m.t, 0.0, 0.8);
        }
        Clip::Spawn => moves::spawn(m, p, &body, Entry::Drop),
        Clip::Evolve => {
            moves::evolve(m, p, &body, Power::Burst);
            p[Secondary(Spring::Back).index()].1.z +=
                0.05 * tools::hold(m.t, [0.0, 0.25, 0.50, 0.9]);
        }
        Clip::Victory => moves::victory(m, p, &body, Cheer::Both),
    }
    if m.clip != Clip::Death {
        p[Secondary(Spring::Back).index()].1.z += (m.clock * 1.5).sin() * 0.008;
    }
    if let Some(t) = m.special {
        let raise = tools::hold(t, [0.0, 0.15, 0.20, 0.30]);
        let stamp = tools::hold(t, [0.18, 0.30, 0.45, 0.75]);
        for (j, s) in [(UpperArmL, 1.0), (UpperArmR, -1.0)] {
            tools::add(p, j, [-105.0 * raise - 40.0 * stamp, 0.0, s * 18.0 * stamp]);
        }
        tools::add(p, Chest, [12.0 * stamp, 0.0, 0.0]);
    }
    tools::face(p, m.aim, Vec3::new(0.0, 1.0, 0.3));
}
