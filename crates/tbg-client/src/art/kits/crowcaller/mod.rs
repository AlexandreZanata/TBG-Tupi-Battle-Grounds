//! Crowcaller's stalking gait, latch reload and opposed raven glance/marked-shot wing.

mod model;
use super::{
    common::{moves, moves::*},
    tools, Anatomy, Attack, Gait, Kit, Moment,
};
use crate::art::{
    anim::{Clip, Pose},
    rig::{
        Joint::*,
        Node::{Prop, Secondary},
        Socket, Spring,
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
    gait: Gait::LIGHT,
    attack: Attack::Shot {
        from: Prop(Socket::WeaponR),
        muzzle: |_, _| model::MUZZLE,
        shot: |_, _| ShotKind::PlagueBolt,
        flash: |_, _| None,
        loaded: Some((Prop(Socket::Nock), loaded)),
    },
    evolve: Burst::Soul,
    durations: moves::DURATIONS,
};
fn loaded(clip: Clip, t: f32) -> bool {
    !matches!(clip, Clip::AttackRanged | Clip::Cast) || t < 0.18 || t > 0.52
}
fn pose(m: &Moment, p: &mut Pose) {
    let body = model::plan(m.tier);
    match m.clip {
        Clip::Idle => moves::idle(m, p, &body, 0.25),
        Clip::Walk => moves::walk(m, p, 5.0, 2.0),
        Clip::AttackRanged | Clip::AttackMelee | Clip::Cast => {
            let recoil = tools::hold(m.t, [0.15, 0.18, 0.23, 0.48]);
            tools::add(p, Chest, [-3.0 * recoil, 0.0, -2.0 * recoil]);
            tools::set(p, Head, [-6.0, 0.0, 0.0]);
        }
        Clip::Hit => moves::hit(m, p, 0.9),
        Clip::Death => {
            moves::death(m, p, &body, Fall::Kneel);
            tools::set(p, Head, [15.0, 30.0, 0.0]);
        }
        Clip::Spawn => moves::spawn(m, p, &body, Entry::Rise),
        Clip::Evolve => {
            moves::evolve(m, p, &body, Power::Raise);
            tools::add(
                p,
                Chest,
                [0.0, 15.0 * tools::hold(m.t, [0.0, 0.25, 0.4, 0.85]), 0.0],
            );
        }
        Clip::Victory => moves::victory(m, p, &body, Cheer::Raise),
    }
    if m.clip != Clip::Death {
        grip(m, p, &body);
    }
    p[Secondary(Spring::HeadGear).index()].0.y = -(m.clock * 0.7).sin() * 18.0;
    p[Secondary(Spring::Tabard).index()].0.x = (m.clock * 1.2).sin() * 3.0;
    let marked = m
        .special
        .map_or(0.0, |t| tools::hold(t, [0.25, 0.30, 0.45, 0.85]));
    p[Prop(Socket::Back).index()].0.z -= 75.0 * marked;
    if m.special_ready {
        p[Secondary(Spring::HeadGear).index()].0.y *= 0.35;
    }
}
fn grip(m: &Moment, p: &mut Pose, body: &crate::art::rig::Proportions) {
    let firing = matches!(m.clip, Clip::AttackRanged | Clip::Cast);
    let latch = if firing {
        tools::hold(m.t, [0.0, 0.12, 0.15, 0.18])
    } else {
        0.0
    };
    let recoil = if firing {
        tools::hold(m.t, [0.15, 0.18, 0.23, 0.48])
    } else {
        0.0
    };
    tools::reach(
        p,
        false,
        Vec3::new(-0.14, 0.18, 0.22 - 0.04 * recoil),
        Some(Quat::IDENTITY),
        (body, Vec3::new(-1.0, -0.3, 0.0)),
    );
    tools::reach(
        p,
        true,
        Vec3::new(0.10, 0.16, 0.31 - 0.20 * latch),
        Some(Quat::IDENTITY),
        (body, Vec3::new(1.0, -0.3, 0.0)),
    );
    tools::face(p, m.aim, Vec3::new(0.0, 0.7, 0.3));
}
