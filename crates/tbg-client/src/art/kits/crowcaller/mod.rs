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
        Clip::Idle => {
            moves::idle(m, p, &body, 0.25);
            let look = tools::hold((m.clock * 0.23).rem_euclid(1.0), [0.12, 0.30, 0.45, 0.68]);
            tools::add(p, Head, [3.0 * look, 15.0 * look, 0.0]);
        }
        Clip::Walk => {
            moves::walk(m, p, 5.0, 2.0);
            let step = (m.walk.unwrap_or(0.0) * std::f32::consts::TAU).sin();
            tools::add(p, Head, [-3.0, -4.0 * step, 0.0]);
            p[Secondary(Spring::Back).index()].0.z = -3.0 * step;
        }
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
        Clip::Victory => {
            p.fill((Vec3::ZERO, Vec3::ZERO));
            let bow = tools::hold(m.t.rem_euclid(0.8), [0.10, 0.28, 0.44, 0.74]);
            tools::set(p, Spine, [6.0 * bow, 0.0, 0.0]);
            tools::set(p, Head, [8.0 * bow, 8.0 * bow, 0.0]);
            p[Prop(Socket::Back).index()].0.z = -28.0 * bow;
            p[Secondary(Spring::HeadGear).index()].0.x = -6.0 * bow;
            tools::plant(p, &body, tools::stance(&body, 1.1, 0.04));
        }
    }
    if m.clip != Clip::Death {
        grip(m, p, &body);
    }
    let reply = if m.clip == Clip::Idle {
        tools::hold((m.clock * 0.23).rem_euclid(1.0), [0.12, 0.30, 0.45, 0.68])
    } else {
        0.0
    };
    p[Secondary(Spring::HeadGear).index()].0.y = -(m.clock * 0.7).sin() * 12.0 - 18.0 * reply;
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
    let bow = if m.clip == Clip::Victory {
        tools::hold(m.t.rem_euclid(0.8), [0.10, 0.28, 0.44, 0.74])
    } else {
        0.0
    };
    let turn = Quat::from_rotation_x((10.0 * bow - 6.0 * recoil).to_radians());
    let at = Vec3::new(0.0, 0.23 - 0.045 * bow, 0.23 - 0.025 * recoil);
    tools::reach(p, false, at, Some(turn), (body, Vec3::new(-1.0, -0.3, 0.0)));
    let support = Vec3::new(0.0, -0.02, 0.09 - 0.145 * latch);
    tools::reach(
        p,
        true,
        at + turn * support,
        Some(turn),
        (body, Vec3::new(1.0, -0.3, 0.0)),
    );
    tools::face(p, m.aim, Vec3::new(0.0, 0.7, 0.3));
}
