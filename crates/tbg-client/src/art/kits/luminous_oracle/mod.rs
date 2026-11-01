//! Oracle's small planted robe steps, staff alignment, closing offhand and lowering hood/cape fall.
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
    gait: Gait {
        stride: 0.8,
        lift: 0.15,
        ..Gait::LIGHT
    },
    attack: Attack::Shot {
        from: Prop(Socket::WeaponR),
        muzzle: model::muzzle,
        shot: |_, _| ShotKind::SunSpark,
        flash: |_, _| None,
        loaded: None,
    },
    evolve: Burst::Valor,
    durations: moves::DURATIONS,
};
fn pose(m: &Moment, p: &mut Pose) {
    let body = model::plan(m.tier);
    match m.clip {
        Clip::Idle => {
            moves::idle(m, p, &body, 0.3);
            tools::set(p, Head, [6.0, 0.0, 0.0]);
        }
        Clip::Walk => moves::walk(m, p, 2.0, 1.0),
        Clip::Cast | Clip::AttackRanged | Clip::AttackMelee => {
            let k = tools::hold(m.t, [0.1, 0.22, 0.30, 0.60]);
            tools::set(p, Chest, [-5.0 * k, 0.0, 0.0]);
        }
        Clip::Hit => {
            moves::hit(m, p, 0.7);
            tools::set(p, Chest, [0.0, 12.0, 0.0]);
        }
        Clip::Death => {
            moves::death(m, p, &body, Fall::Kneel);
            p[Prop(Socket::WeaponR).index()].0.x = 35.0 * tools::window(m.t, 0.2, 0.8);
        }
        Clip::Spawn => moves::spawn(m, p, &body, Entry::Rise),
        Clip::Evolve => moves::evolve(m, p, &body, Power::Raise),
        Clip::Victory => moves::victory(m, p, &body, Cheer::Raise),
    }
    if m.clip != Clip::Death {
        let close = m
            .special
            .map_or(0.0, |t| tools::hold(t, [0.1, 0.30, 0.40, 0.70]));
        tools::reach(
            p,
            false,
            Vec3::new(-0.14, 0.10, 0.15),
            Some(Quat::IDENTITY),
            (&body, Vec3::new(-1.0, -0.2, 0.0)),
        );
        tools::reach(
            p,
            true,
            Vec3::new(0.08 - 0.12 * close, 0.16, 0.25 - 0.07 * close),
            Some(Quat::IDENTITY),
            (&body, Vec3::new(1.0, -0.2, 0.0)),
        );
        p[Prop(Socket::OffhandL).index()].0.x = -30.0 * close;
    }
    p[Secondary(Spring::HeadGear).index()].0.y = (m.clock * 0.9).sin() * 12.0;
    p[Secondary(Spring::Back).index()].0.x = (m.clock * 1.0).sin() * 2.0;
    tools::face(p, m.aim, Vec3::new(0.0, 0.9, 0.3));
}
