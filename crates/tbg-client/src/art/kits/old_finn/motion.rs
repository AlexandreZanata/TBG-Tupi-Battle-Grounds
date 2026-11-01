//! Finn's line checks, boot-weighted walk, rod cast and reel-and-creel celebration.

use bevy::prelude::*;

use super::model::plan;
use crate::art::{
    anim::{Clip, Pose},
    kits::{
        common::moves::{self, Entry, Fall, Power},
        tools, Moment,
    },
    rig::{Joint::*, Proportions},
};

pub fn pose(m: &Moment, pose: &mut Pose) {
    let p = plan(m.tier);
    // His stoop and lifted chin are anatomical, not an attack anticipation.
    tools::add(pose, Spine, [9.0, 0.0, 0.0]);
    tools::add(pose, Head, [-9.0, 0.0, 0.0]);
    match m.clip {
        Clip::Idle => idle(m, pose, &p),
        Clip::Walk => walk(m, pose, &p),
        Clip::Cast => cast(m, pose, &p),
        Clip::Victory => victory(m, pose, &p),
        Clip::Hit => {
            moves::hit(m, pose, 0.8);
            rod(pose, &p, 24.0, Vec3::new(-0.23, 0.12, 0.20));
            reel(pose, &p, Vec3::new(-0.23, 0.12, 0.20), 24.0, 0.0);
        }
        Clip::Death => {
            moves::death(m, pose, &p, Fall::Kneel);
            rod(pose, &p, 65.0, Vec3::new(-0.26, 0.03, 0.23));
        }
        Clip::Spawn => {
            moves::spawn(m, pose, &p, Entry::March);
            rod(pose, &p, -22.0, Vec3::new(-0.27, 0.14, 0.18));
            reel(pose, &p, Vec3::new(-0.27, 0.14, 0.18), -22.0, 0.0);
        }
        Clip::Evolve => {
            moves::evolve(m, pose, &p, Power::Raise);
            let pride = tools::hold(m.t, [0.08, 0.28, 0.6, 0.9]);
            rod(pose, &p, -12.0, Vec3::new(-0.29, 0.12 + 0.16 * pride, 0.16));
            tools::reach(
                pose,
                true,
                Vec3::new(0.25, 0.12 + 0.12 * pride, 0.20),
                None,
                (&p, Vec3::new(1.0, -0.5, -0.4)),
            );
        }
        _ => rod(pose, &p, 24.0, Vec3::new(-0.23, 0.12, 0.20)),
    }
}

fn idle(m: &Moment, pose: &mut Pose, p: &Proportions) {
    moves::idle(m, pose, p, 0.4);
    let check = tools::hold((m.clock / 3.2).rem_euclid(1.0), [0.10, 0.28, 0.48, 0.72]);
    let nibble = (m.clock * 2.3).sin();
    tools::add(pose, Head, [5.0 * check, -16.0 * check, -3.0 * check]);
    rod(
        pose,
        p,
        22.0 - 12.0 * check + 3.0 * nibble,
        Vec3::new(-0.23, 0.12 + 0.025 * check, 0.20),
    );
    // The free hand pinches the line, then returns to the creel strap.
    tools::reach(
        pose,
        true,
        Vec3::new(
            0.18 - 0.14 * check,
            0.09 + 0.12 * check,
            0.16 + 0.06 * check,
        ),
        None,
        (p, Vec3::new(1.0, -0.6, -0.3)),
    );
}

fn walk(m: &Moment, pose: &mut Pose, p: &Proportions) {
    let phase = m.walk.unwrap_or(0.0) * std::f32::consts::TAU;
    let step = phase.sin();
    moves::walk(m, pose, 5.0, 3.0);
    tools::add(pose, Chest, [2.0 * (phase * 2.0).cos(), -3.0 * step, 0.0]);
    tools::add(pose, Head, [0.0, 4.0 * step, -2.0 * step]);
    rod(
        pose,
        p,
        -20.0 + 3.0 * step,
        Vec3::new(-0.27, 0.12 - 0.015 * step, 0.20),
    );
    tools::reach(
        pose,
        true,
        Vec3::new(0.21, 0.13 + 0.015 * step, 0.15),
        None,
        (p, Vec3::new(1.0, -0.6, -0.5)),
    );
}

/// The shaft stays outside his right shoulder; the hand remains its socket owner.
fn rod(pose: &mut Pose, p: &Proportions, tilt: f32, wrist: Vec3) {
    tools::reach(
        pose,
        false,
        wrist,
        Some(Quat::from_rotation_x(tilt.to_radians())),
        (p, Vec3::new(-1.0, -0.6, -0.3)),
    );
}

fn reel(pose: &mut Pose, p: &Proportions, wrist: Vec3, tilt: f32, phase: f32) {
    let handle = Vec3::new(0.192, 0.07 + 0.015 * phase.sin(), 0.015 * phase.cos());
    tools::reach(
        pose,
        true,
        wrist + Quat::from_rotation_x(tilt.to_radians()) * handle,
        None,
        (p, Vec3::new(1.0, -0.5, -0.4)),
    );
}

/// Load over the shoulder, feed with the free hand, whip at the existing 0.22 s Release.
fn cast(m: &Moment, pose: &mut Pose, p: &Proportions) {
    let load = tools::window(m.t, 0.0, 0.15);
    let whip = tools::snap(m.t, 0.15, 0.22);
    let settle = tools::window(m.t, 0.38, 0.62);
    let active = 1.0 - settle;
    let tilt = ((24.0 - 86.0 * load) * (1.0 - whip) + 70.0 * whip) * active + 24.0 * settle;
    let lift = (0.18 * load * (1.0 - whip) + 0.06 * whip) * active;
    tools::set(
        pose,
        Chest,
        [
            (-8.0 * load + 13.0 * whip) * active,
            -18.0 * load * (1.0 - whip) * active,
            0.0,
        ],
    );
    tools::add(
        pose,
        Head,
        [-7.0 * load * (1.0 - whip), 12.0 * load * (1.0 - whip), 0.0],
    );
    tools::shift(
        pose,
        Hips,
        [0.0, -0.025 * whip * active, 0.025 * whip * active],
    );
    rod(
        pose,
        p,
        tilt,
        Vec3::new(-0.27, 0.12 + lift, 0.18 + 0.07 * whip * active),
    );
    tools::reach(
        pose,
        true,
        Vec3::new(
            0.08 + 0.10 * whip * active,
            0.15 + 0.05 * load * active,
            0.23,
        ),
        None,
        (p, Vec3::new(1.0, -0.5, -0.3)),
    );
    tools::face(pose, m.aim, Vec3::ZERO);
    tools::plant(pose, p, tools::stance(p, 1.3, 0.07));
}

fn victory(m: &Moment, pose: &mut Pose, p: &Proportions) {
    let phase = m.t / 0.8 * std::f32::consts::TAU;
    let sway = phase.sin();
    let nod = (phase * 2.0).sin();
    pose[Root.index()] = (Vec3::ZERO, Vec3::ZERO);
    tools::set(pose, Spine, [7.0, 0.0, 4.0 * sway]);
    tools::set(pose, Chest, [0.0, 6.0 * sway, 0.0]);
    tools::set(pose, Head, [-11.0 - 4.0 * nod, -8.0 * sway, -4.0 * sway]);
    tools::shift(pose, Hips, [0.025 * sway, -0.015 * nod.abs(), 0.0]);
    let wrist = Vec3::new(-0.23, 0.18 + 0.025 * nod, 0.17);
    let tilt = -12.0 + 5.0 * sway;
    rod(pose, p, tilt, wrist);
    // Rock over planted boots while the free hand works at the actual reel handle.
    reel(pose, p, wrist, tilt, phase);
    tools::plant(pose, p, tools::stance(p, 1.25, 0.04));
}
