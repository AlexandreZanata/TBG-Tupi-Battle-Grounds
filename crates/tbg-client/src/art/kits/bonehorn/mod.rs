//! Bonehorn's straight-lane quadruped gait, braced tusk shove and tightening mantle.

mod model;

use super::{common::moves, tools, Anatomy, Attack, Gait, Kit, Moment};
use crate::art::{
    anim::{Clip, Pose},
    rig::{Joint::*, Node, Spring},
};
use crate::vfx::impacts::{Burst, Impact};
use bevy::prelude::*;

static ANATOMY: Anatomy = Anatomy {
    rest: model::rest,
    parent: Node::parent,
    springs: false,
    planted_legs: false,
};
pub const KIT: Kit = Kit {
    anatomy: Some(&ANATOMY),
    figure: model::spec,
    pose,
    gait: Gait::HEAVY,
    attack: Attack::Melee {
        from: Node::Body(Head),
        tip: |_, _| Vec3::new(0.0, 0.1, 0.58),
        impact: |_, _| Impact::Shards,
    },
    evolve: Burst::Soul,
    durations: moves::DURATIONS,
};

fn pose(m: &Moment, p: &mut Pose) {
    p.fill((Vec3::ZERO, Vec3::ZERO));
    tools::set(p, Head, [5.0, 0.0, 0.0]);
    match m.clip {
        Clip::Idle => {
            let breath = (m.clock * 1.4).sin();
            let sniff = tools::wave(m.clock, 0.65, 0.8);
            tools::shift(p, Chest, [0.0, breath * 0.012, -0.015 * sniff]);
            tools::add(p, Head, [5.0 + sniff * 5.0, sniff * 3.0, -breath]);
            tools::set(p, Spine, [0.0, 0.0, breath * 1.2]);
        }
        Clip::Victory => {
            let show = 0.5 - 0.5 * (m.t / 0.8 * std::f32::consts::TAU).cos();
            tools::set(p, Head, [-16.0 * show, 9.0 * show, 0.0]);
            tools::shift(p, Chest, [0.0, 0.025 * show, -0.025 * show]);
            tools::set(p, Spine, [-3.0 * show, 0.0, 0.0]);
            for j in [ForearmL, ForearmR] {
                tools::set(p, j, [-5.0 * show, 0.0, 0.0]);
            }
        }
        Clip::Walk => walk(m, p),
        Clip::AttackMelee | Clip::AttackRanged => shove(m, p),
        Clip::Hit => {
            let h = tools::hold(m.t, [0.0, 0.04, 0.10, 0.25]);
            tools::shift(p, Chest, [0.0, -0.05 * h, -0.04 * h]);
            tools::add(p, Head, [-12.0 * h, 0.0, 0.0]);
        }
        Clip::Death => {
            let front = tools::window(m.t, 0.0, 0.32);
            let rear = tools::window(m.t, 0.28, 0.7);
            tools::shift(p, Hips, [0.0, -0.43 * rear, 0.0]);
            tools::set(p, Spine, [15.0 * front * (1.0 - rear), 0.0, 6.0 * rear]);
            for j in [ForearmL, ForearmR] {
                tools::set(p, j, [-85.0 * front, 0.0, 0.0]);
            }
            for j in [ShinL, ShinR] {
                tools::set(p, j, [-80.0 * rear, 0.0, 0.0]);
            }
            tools::add(p, Head, [25.0 * rear, 0.0, 0.0]);
        }
        Clip::Spawn | Clip::Evolve => {
            let rise = 1.0 - tools::window(m.t, 0.0, 0.5);
            tools::shift(p, Hips, [0.0, -0.25 * rise, 0.0]);
            tools::add(p, Head, [-20.0 * (m.t * 5.0).sin().max(0.0), 0.0, 0.0]);
        }
        Clip::Cast => {}
    }
    // Public armor survives reconnect; a short squeeze also marks each new stack.
    let plate = Node::Secondary(Spring::Back).index();
    p[plate].1.y -= m.reduction as f32 / 10_000.0 * 0.10 + m.armor_flash * 0.035;
    p[plate].0.x += m.armor_flash * 5.0;
    // The full quadruped footprint follows the lane. A nearby target can be
    // behind the long muzzle pivot; turning the Root there inverted the body.
    if let Some(aim) = m.aim {
        let yaw = super::aim::yaw(Vec3::ZERO, aim.target).to_degrees();
        tools::add(p, Head, [0.0, yaw.clamp(-15.0, 15.0) * aim.weight, 0.0]);
    }
}

fn walk(m: &Moment, p: &mut Pose) {
    let phase = m.walk.unwrap_or(0.0) * std::f32::consts::TAU;
    for (upper, lower, offset) in [
        (UpperArmL, ForearmL, 0.0),
        (ThighR, ShinR, 0.0),
        (UpperArmR, ForearmR, std::f32::consts::PI),
        (ThighL, ShinL, std::f32::consts::PI),
    ] {
        let swing = (phase + offset).sin();
        tools::set(p, upper, [swing * 20.0, 0.0, 0.0]);
        tools::set(p, lower, [-swing.max(0.0) * 28.0, 0.0, 0.0]);
    }
    tools::shift(p, Hips, [0.0, (phase * 2.0).cos() * 0.018, 0.0]);
    tools::add(p, Head, [phase.sin() * 3.0, -phase.sin() * 2.0, 0.0]);
    tools::set(
        p,
        Chest,
        [(phase * 2.0).sin() * 2.0, 0.0, phase.sin() * -1.5],
    );
}

fn shove(m: &Moment, p: &mut Pose) {
    let coil = tools::hold(m.t, [0.0, 0.20, 0.22, 0.30]);
    let hit = tools::hold(m.t, [0.22, 0.30, 0.34, 0.60]);
    tools::shift(
        p,
        Chest,
        [0.0, -0.08 * coil + 0.08 * hit, -0.09 * coil + 0.12 * hit],
    );
    tools::add(p, Head, [26.0 * coil - 36.0 * hit, 0.0, 0.0]);
    tools::shift(p, Head, [0.0, -0.02 * coil, -0.025 * coil + 0.035 * hit]);
    for j in [UpperArmL, UpperArmR] {
        tools::set(p, j, [-12.0 * coil + 8.0 * hit, 0.0, 0.0]);
    }
    for j in [ForearmL, ForearmR] {
        tools::set(p, j, [-18.0 * coil, 0.0, 0.0]);
    }
}
