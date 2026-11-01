//! Bonehorn's diagonal quadruped gait, braced tusk shove and tightening mantle.

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
        Clip::Idle | Clip::Victory => {
            let breath = (m.clock * 1.4).sin();
            tools::shift(p, Chest, [0.0, breath * 0.012, 0.0]);
            tools::add(p, Head, [breath * 2.0, 0.0, 0.0]);
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
    tools::add(p, Head, [phase.sin() * 3.0, 0.0, 0.0]);
}

fn shove(m: &Moment, p: &mut Pose) {
    let coil = tools::hold(m.t, [0.0, 0.20, 0.22, 0.30]);
    let hit = tools::hold(m.t, [0.22, 0.30, 0.34, 0.60]);
    tools::shift(
        p,
        Chest,
        [0.0, -0.08 * coil + 0.08 * hit, -0.09 * coil + 0.12 * hit],
    );
    tools::add(p, Head, [22.0 * coil - 32.0 * hit, 0.0, 0.0]);
    for j in [UpperArmL, UpperArmR] {
        tools::set(p, j, [-12.0 * coil + 8.0 * hit, 0.0, 0.0]);
    }
    for j in [ForearmL, ForearmR] {
        tools::set(p, j, [-18.0 * coil, 0.0, 0.0]);
    }
}
