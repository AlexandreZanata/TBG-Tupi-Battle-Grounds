//! Whisperer's grounded float cadence and independent fear pulse overlay.

mod model;
use super::{common::moves, tools, Anatomy, Attack, Gait, Kit, Moment};
use crate::art::{
    anim::{Clip, Pose},
    rig::{Joint::*, Node::Prop, Socket::*},
};
use crate::vfx::{impacts::Burst, shots::ShotKind};
use bevy::prelude::*;

static ANATOMY: Anatomy = Anatomy {
    rest: model::rest,
    parent: model::parent,
    springs: false,
    planted_legs: false,
};
pub const KIT: Kit = Kit {
    anatomy: Some(&ANATOMY),
    figure: model::spec,
    pose,
    gait: Gait::LIGHT,
    attack: Attack::Shot {
        from: Prop(Back),
        muzzle: |_, _| Vec3::new(0.0, 0.0, 0.15),
        shot: |_, _| ShotKind::Thought,
        flash: |_, _| None,
        loaded: None,
    },
    evolve: Burst::Soul,
    durations: moves::DURATIONS,
};
fn pose(m: &Moment, p: &mut Pose) {
    p.fill((Vec3::ZERO, Vec3::ZERO));
    let breath = (m.clock * 1.8).sin();
    tools::shift(p, Chest, [0.0, breath * -0.018, 0.0]);
    tendrils(p, breath * 5.0);
    match m.clip {
        Clip::Idle | Clip::Victory => tools::set(p, Head, [0.0, (m.clock * 0.9).sin() * 5.0, 0.0]),
        Clip::Walk => {
            let phase = m.walk.unwrap_or(0.0) * std::f32::consts::TAU;
            tools::set(p, Spine, [phase.cos() * 3.0, 0.0, phase.sin() * 6.0]);
            tools::shift(p, Hips, [0.0, phase.cos() * 0.035, 0.0]);
            tendrils(p, phase.sin() * 15.0);
        }
        Clip::Cast | Clip::AttackRanged | Clip::AttackMelee => pulse(p, m.t, 0.65),
        Clip::Hit => {
            let h = tools::hold(m.t, [0.0, 0.04, 0.10, 0.25]);
            tools::shift(p, Chest, [0.0, -0.07 * h, -0.05 * h]);
            tools::set(p, Head, [0.0, 0.0, 15.0 * h]);
        }
        Clip::Death => {
            let sink = tools::window(m.t, 0.0, 0.8);
            tools::shift(p, Hips, [0.0, -0.60 * sink, 0.0]);
            tendrils(p, -70.0 * sink);
            p[Prop(Back).index()].1.z -= 0.17 * sink;
            tools::set(p, Chest, [20.0 * sink, 0.0, 0.0]);
        }
        Clip::Spawn | Clip::Evolve => {
            let open = tools::hold(m.t, [0.0, 0.3, 0.45, 0.9]);
            tendrils(p, 35.0 * open);
            tools::shift(p, Head, [0.0, open * 0.07, 0.0]);
        }
    }
    if m.special_ready {
        tendrils(p, 5.0);
    }
    if let Some(t) = m.special {
        pulse(p, t, 1.0);
    }
    tools::face(p, m.aim, Vec3::new(0.0, 1.0, 0.2));
}
fn tendrils(p: &mut Pose, spread: f32) {
    for (j, s) in [
        (UpperArmL, 1.0),
        (UpperArmR, -1.0),
        (ThighL, 1.0),
        (ThighR, -1.0),
    ] {
        tools::add(p, j, [0.0, 0.0, s * spread]);
    }
    for (n, s) in [(WeaponR, 1.0), (OffhandL, -1.0)] {
        p[Prop(n).index()].0.z += s * spread;
    }
}
fn pulse(p: &mut Pose, t: f32, weight: f32) {
    let fold = tools::hold(t, [0.0, 0.18, 0.22, 0.30]) * weight;
    let spread = tools::hold(t, [0.22, 0.30, 0.38, 0.65]) * weight;
    tendrils(p, -22.0 * fold + 35.0 * spread);
    tools::shift(p, Chest, [0.0, -0.04 * fold + 0.03 * spread, 0.0]);
    p[Prop(Back).index()].1.z += 0.035 * spread;
}
