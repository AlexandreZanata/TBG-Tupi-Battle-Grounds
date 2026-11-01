//! Bulwark's compact plod, two-arm tool blow and server-expiry-bound brace.

mod model;
use super::{
    common::{moves, moves::*},
    tools, Attack, Gait, Kit, Moment,
};
use crate::art::{
    anim::{Clip, Pose},
    rig::{Joint::*, Node::Prop, Socket},
};
use crate::vfx::impacts::{Burst, Impact};
use bevy::prelude::*;
pub const KIT: Kit = Kit {
    anatomy: None,
    figure: model::spec,
    pose,
    gait: Gait::HEAVY,
    attack: Attack::Melee {
        from: Prop(Socket::WeaponR),
        tip: model::tip,
        impact: |_, _| Impact::Crush,
    },
    evolve: Burst::Forge,
    durations: moves::DURATIONS,
};
fn pose(m: &Moment, p: &mut Pose) {
    let body = model::plan(m.tier);
    match m.clip {
        Clip::Idle => {
            moves::idle(m, p, &body, 0.5);
            tools::add(p, Head, [0.0, (m.clock * 0.7).sin() * 5.0, 0.0]);
            let settle = tools::hold((m.clock * 0.24).fract(), [0.0, 0.12, 0.20, 0.40]);
            tools::shift(p, Head, [0.0, -0.012 * settle, 0.012 * settle]);
            tool(p, &body, -8.0 + 5.0 * settle, 0.0);
        }
        Clip::Walk => {
            moves::walk(m, p, 5.0, 3.0);
            tools::add(
                p,
                Spine,
                [
                    0.0,
                    0.0,
                    (m.walk.unwrap_or(0.0) * std::f32::consts::TAU).sin() * 3.0,
                ],
            );
            tool(
                p,
                &body,
                -12.0,
                0.008 * (m.walk.unwrap_or(0.0) * std::f32::consts::TAU * 2.0).cos(),
            );
        }
        Clip::AttackMelee | Clip::AttackRanged => {
            let load = tools::hold(m.t, [0.0, 0.17, 0.23, 0.30]);
            let strike = tools::hold(m.t, [0.23, 0.30, 0.40, 0.62]);
            tools::set(p, Chest, [-4.0 * load + 8.0 * strike, 0.0, 0.0]);
            tool(p, &body, -15.0 - 115.0 * load - 75.0 * strike, 0.045 * load);
            tools::face(p, m.aim, Vec3::ZERO);
            tools::plant(p, &body, tools::stance(&body, 1.3, 0.04));
            tools::shift(
                p,
                Head,
                [0.0, -0.035 * tools::hold(m.t, [0.0, 0.15, 0.25, 0.5]), 0.0],
            );
        }
        Clip::Hit => {
            moves::hit(m, p, 0.35);
            tools::shift(p, Head, [0.0, -0.035, -0.04]);
        }
        Clip::Death => moves::death(m, p, &body, Fall::Kneel),
        Clip::Spawn => moves::spawn(m, p, &body, Entry::Drop),
        Clip::Evolve => {
            moves::evolve(m, p, &body, Power::Raise);
            tools::shift(
                p,
                Chest,
                [0.0, 0.08 * tools::hold(m.t, [0.0, 0.25, 0.45, 0.8]), 0.0],
            );
        }
        Clip::Victory => {
            let salute = 0.5 - 0.5 * (m.t / 0.8 * std::f32::consts::TAU).cos();
            tools::set(p, Chest, [-3.0 * salute, 0.0, 0.0]);
            tools::set(p, Head, [-8.0 * salute, 6.0, 0.0]);
            tool(p, &body, -20.0 - 45.0 * salute, 0.035 * salute);
            tools::plant(p, &body, tools::stance(&body, 1.25, 0.0));
        }
        Clip::Cast => {}
    }
    let preparing = m
        .special
        .map_or(0.0, |t| tools::hold(t, [0.0, 0.15, 0.25, 0.5]));
    let held = if m.reduction > 0 { 1.0 } else { 0.0 };
    let brace = preparing.max(held);
    if brace > 0.0 && m.clip != Clip::Death {
        tools::shift(p, Hips, [0.0, -0.035 * brace, 0.0]);
        tools::plant(p, &body, tools::stance(&body, 1.0 + brace * 0.25, 0.0));
        tools::add(p, Chest, [5.0 * brace, 0.0, 0.0]);
    }
}

/// Both palms follow points on the same tool shaft throughout its lift.
fn tool(p: &mut Pose, body: &crate::art::rig::Proportions, pitch: f32, lift: f32) {
    moves::shoulder_arms(
        p,
        body,
        Vec3::new(0.0, 0.10 + lift, 0.10),
        Quat::from_rotation_x(pitch.to_radians()),
        Vec3::Y * 0.12,
    );
}
