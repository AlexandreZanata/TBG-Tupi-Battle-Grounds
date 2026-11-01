//! Furnace breathing and a two-handed hammer blow that fully recovers in its existing 0.52 seconds.

use super::model;
use crate::art::{
    anim::{Clip, Pose},
    kits::{
        common::moves::{self, Entry, Fall, Power},
        tools, Moment,
    },
    rig::{Joint::*, Proportions},
};
use bevy::prelude::*;

pub(super) fn pose(m: &Moment, p: &mut Pose) {
    let body = model::plan(m.tier);
    let (mut pitch, mut at) = (35.0f32, Vec3::new(0.02, 0.25, 0.25));
    match m.clip {
        Clip::Idle => {
            moves::idle(m, p, &body, 0.6);
            let exhale = (m.clock * 2.6).sin().max(0.0).powi(2);
            tools::add(p, Chest, [-4.0 * exhale, 0.0, 0.0]);
            tools::add(p, Head, [3.0 * exhale, 0.0, 0.0]);
            pitch -= 4.0 * exhale;
        }
        Clip::Walk => {
            moves::walk(m, p, 6.0, 4.0);
            let step = (m.walk.unwrap_or(0.0) * std::f32::consts::TAU).sin();
            tools::add(p, Head, [-2.0, -5.0 * step, 0.0]);
            at.y -= 0.01 * step;
            pitch += 3.0 * step;
        }
        Clip::AttackMelee => {
            p.fill((Vec3::ZERO, Vec3::ZERO));
            let load = tools::hold(m.t, [0.0, 0.18, 0.20, 0.30]);
            let strike = tools::hold(m.t, [0.20, 0.30, 0.34, 0.52]);
            pitch = 35.0 - 100.0 * load + 15.0 * strike;
            at.y += 0.06 * load;
            tools::set(p, Chest, [-6.0 * load + 9.0 * strike, 0.0, 0.0]);
            tools::set(p, Head, [5.0 * load - 6.0 * strike, 0.0, 0.0]);
            tools::shift(p, Hips, [0.0, -0.025 * strike, 0.025 * strike]);
            tools::face(p, m.aim, Vec3::ZERO);
            tools::plant(p, &body, tools::stance(&body, 1.3, 0.08));
        }
        Clip::Hit => moves::hit(m, p, 0.8),
        Clip::Death => moves::death(m, p, &body, Fall::Sink),
        Clip::Spawn => moves::spawn(m, p, &body, Entry::Rise),
        Clip::Evolve => {
            moves::evolve(m, p, &body, Power::Raise);
            pitch -= 55.0 * tools::hold(m.t, [0.15, 0.35, 0.65, 0.9]);
        }
        Clip::Victory => {
            p.fill((Vec3::ZERO, Vec3::ZERO));
            let flare = tools::hold(m.t.rem_euclid(0.8), [0.05, 0.25, 0.43, 0.75]);
            tools::set(p, Chest, [-5.0 * flare, 0.0, 0.0]);
            tools::set(p, Head, [-8.0 * flare, 0.0, 0.0]);
            pitch -= 60.0 * flare;
            tools::plant(p, &body, tools::stance(&body, 1.3, 0.03));
        }
        _ => {}
    }
    hammer(p, &body, at, Quat::from_rotation_x(pitch.to_radians()));
}

fn hammer(p: &mut Pose, body: &Proportions, at: Vec3, turn: Quat) {
    // Both targets lie on the same actual +Z haft; the wraps mark these grip anchors.
    for (left, z, side) in [(false, 0.0, -1.0), (true, 0.14, 1.0)] {
        tools::reach(
            p,
            left,
            at + turn * Vec3::Z * z,
            Some(turn),
            (body, Vec3::new(side, -0.5, 0.0)),
        );
    }
}
