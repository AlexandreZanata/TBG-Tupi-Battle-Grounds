//! Weighted furnace-bearing movement, bellows bracing and a restrained nozzle salute.

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

pub(super) fn pose(m: &Moment, pose: &mut Pose) {
    let body = model::plan(m.tier);
    let mut lift = 0.0;
    let mut recoil = 0.0;
    match m.clip {
        Clip::Idle => {
            // Aim is applied once below, after the nozzle grip is assembled.
            moves::idle(&Moment { aim: None, ..*m }, pose, &body, 1.1);
            let pressure = (m.clock * 1.5).sin();
            tools::set(pose, Chest, [9.0 + 2.0 * pressure, 0.0, 0.0]);
            tools::set(pose, Head, [-8.0, 8.0 * (m.clock * 0.45).sin(), 0.0]);
            lift = pressure * 0.012;
        }
        Clip::Walk => {
            moves::walk(m, pose, 11.0, 3.0);
            let step = (m.walk.unwrap_or(0.0) * std::f32::consts::TAU).sin();
            tools::set(pose, Chest, [11.0 + 2.0 * step.abs(), -2.5 * step, 0.0]);
            tools::set(pose, Head, [-10.0, 0.0, 2.0 * step]);
            lift = -0.018 * step.abs();
        }
        Clip::AttackMelee | Clip::AttackRanged | Clip::Cast => {
            let brace = tools::hold(m.t, [0.0, 0.20, 0.30, 0.48]);
            recoil = tools::hold(m.t, [0.30, 0.33, 0.36, 0.48]);
            tools::set(pose, Chest, [9.0 + 4.0 * brace - 6.0 * recoil, 0.0, 0.0]);
            tools::set(pose, Head, [-9.0 + 3.0 * recoil, 0.0, 0.0]);
            tools::shift(pose, Hips, [0.0, -0.035 * brace, -0.025 * recoil]);
            tools::plant(pose, &body, tools::stance(&body, 1.3, 0.08));
        }
        Clip::Hit => moves::hit(m, pose, 0.45),
        Clip::Death => {
            moves::death(m, pose, &body, Fall::Kneel);
            lift = -0.04 * tools::window(m.t, 0.0, 0.6);
        }
        Clip::Spawn => moves::spawn(m, pose, &body, Entry::Rise),
        Clip::Evolve => {
            moves::evolve(m, pose, &body, Power::Raise);
            lift = 0.06 * tools::hold(m.t, [0.0, 0.3, 0.6, 0.9]);
        }
        Clip::Victory => {
            let beat = (m.t / 0.8 * std::f32::consts::TAU).sin();
            tools::set(pose, Root, [0.0, 0.0, 0.0]);
            pose[Root.index()].1 = Vec3::ZERO;
            tools::set(pose, Chest, [6.0 - 3.0 * beat, 0.0, 2.0 * beat]);
            tools::set(pose, Head, [-10.0 - 4.0 * beat, 9.0 * beat, 0.0]);
            lift = 0.04 * (0.5 + 0.5 * beat);
            tools::plant(pose, &body, tools::stance(&body, 1.25, 0.05));
        }
    }
    grip(m, pose, &body, lift, recoil);
    tools::face(pose, m.aim, Vec3::new(0.0, 0.8, 0.3));
}

fn grip(m: &Moment, pose: &mut Pose, body: &Proportions, lift: f32, recoil: f32) {
    let elevation = m
        .aim
        .map_or(0.0, |aim| {
            (0.85 - aim.target.y).atan2(aim.target.xz().length().max(0.1)) * aim.weight
        })
        .clamp(-0.2, 0.3);
    let salute = if m.clip == Clip::Victory {
        0.5 - 0.5 * (m.t / 0.8 * std::f32::consts::TAU).cos()
    } else {
        0.0
    };
    let rotation = Quat::from_rotation_x((-8.0 - 24.0 * salute).to_radians() + elevation);
    moves::shoulder_arms(
        pose,
        body,
        Vec3::new(-0.03, 0.24 + lift, 0.17 - 0.025 * recoil),
        rotation,
        Vec3::new(0.0, 0.0, 0.20),
    );
}
