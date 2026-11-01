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
        }
        Clip::AttackMelee | Clip::AttackRanged => {
            moves::strike(m, p, &body, CHOP, 1.1);
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
        Clip::Victory => moves::victory(m, p, &body, Cheer::Raise),
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
    if matches!(m.clip, Clip::Idle | Clip::Walk) {
        tools::reach(
            p,
            true,
            Vec3::new(-0.12, 0.04, 0.16),
            None,
            (&body, Vec3::new(1.0, -0.6, 0.0)),
        );
    }
}
