//! Drillnest's grounded treads, feed-hatch production, barrel recoil and three summon libraries.
mod drills;
mod model;
use super::{common::moves, tools, Anatomy, Attack, Gait, Kit, Moment};
use crate::{
    art::{
        anim::{Clip, Pose},
        rig::{
            Joint::*,
            Node::{Body, Prop, Secondary},
            Socket, Spring,
        },
    },
    vfx::impacts::Burst,
};
use bevy::prelude::*;
pub use drills::{DUO, MEGA, MICRO};
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
    gait: Gait::HEAVY,
    attack: Attack::None,
    evolve: Burst::Forge,
    durations: moves::DURATIONS,
};
fn pose(m: &Moment, p: &mut Pose) {
    for j in [Hips, Chest, Head] {
        p[Body(j).index()] = (Vec3::ZERO, Vec3::ZERO);
    }
    if m.clip == Clip::Walk {
        let k = (m.walk.unwrap_or(0.0) * std::f32::consts::TAU).sin();
        tools::set(p, Hips, [2.0 * k, 0.0, 0.0]);
        tools::set(p, Chest, [-1.5 * k, 0.0, 0.8 * k]);
        for (s, n) in [(-1.0, Spring::Tabard), (1.0, Spring::HeadGear)] {
            p[Secondary(n).index()].1.z = s * k * 0.025;
        }
    }
    if m.clip == Clip::Idle {
        let feed = tools::hold((m.clock * 0.24).fract(), [0.0, 0.10, 0.18, 0.35]);
        tools::shift(p, Chest, [0.0, 0.006 * feed, -0.012 * feed]);
        tools::set(p, Head, [0.0, 4.0 * feed, 0.0]);
    }
    if let Some(t) = m.special {
        let load = tools::hold(t, [0.0, 0.20, 0.30, 0.62]);
        tools::shift(p, Chest, [0.0, 0.04 * load, -0.025 * load]);
        p[Prop(Socket::WeaponR).index()].1.z = -0.08 * load;
        tools::set(p, Head, [-5.0 * load, 0.0, 0.0]);
        tools::shift(p, Hips, [0.0, -0.012 * load, 0.0]);
    }
    match m.clip {
        Clip::Victory => {
            let ready = 0.5 - 0.5 * (m.t / 0.8 * std::f32::consts::TAU).cos();
            tools::set(p, Root, [0.0, 0.0, 0.0]);
            p[Root.index()].1 = Vec3::ZERO;
            tools::set(p, Chest, [-4.0 * ready, 0.0, 0.0]);
            tools::shift(p, Chest, [0.0, 0.018 * ready, 0.0]);
            p[Prop(Socket::WeaponR).index()].0.x = -5.0 * (1.0 - ready);
            tools::set(
                p,
                Head,
                [0.0, 7.0 * (m.t / 0.8 * std::f32::consts::TAU).sin(), 0.0],
            );
        }
        Clip::Death => {
            let k = tools::window(m.t, 0.0, 0.8);
            tools::set(p, Chest, [25.0 * k, 0.0, 0.0]);
            tools::shift(p, Hips, [0.0, -0.15 * k, 0.0]);
        }
        Clip::Spawn => tools::shift(
            p,
            Hips,
            [0.0, 0.20 * (1.0 - tools::window(m.t, 0.0, 0.6)), 0.0],
        ),
        Clip::Evolve => tools::shift(
            p,
            Chest,
            [0.0, 0.06 * tools::hold(m.t, [0.0, 0.3, 0.5, 0.9]), 0.0],
        ),
        Clip::Hit => tools::set(
            p,
            Hips,
            [-4.0 * tools::hold(m.t, [0.0, 0.05, 0.1, 0.25]), 0.0, 0.0],
        ),
        _ => {}
    }
    p[Secondary(Spring::Back).index()].0.x = (m.clock * 1.7).sin();
}
