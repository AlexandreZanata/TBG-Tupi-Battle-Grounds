//! The Smuggler's crate kit: a banded crate with no body. It drops in and
//! bounces, rocks as if something inside wants out, shakes when hit, and
//! breaks apart when it opens or is destroyed. Forms follow the cargo:
//! coin sacks and a gilded chest (A), stacked crates and a lantern (B).

use bevy::prelude::*;

use super::common::{body, gear, moves, Form};
use super::{tools, Attack, Gait, Kit, Moment};
use crate::art::{
    anim::{Clip, Pose},
    figure::{part, FigureSpec},
    mesh::Shape::*,
    palette::*,
    rig::{Joint, Node::Body, Proportions},
};
use crate::vfx::impacts::Burst;

pub const KIT: Kit = Kit {
    figure: spec,
    pose,
    gait: Gait::HEAVY,
    attack: Attack::None,
    evolve: Burst::Valor,
    durations: moves::DURATIONS,
};

fn plan(tier: u8) -> Proportions {
    body::plan(false, 0.85, tier)
}

fn spec(tier: u8, branch: u8) -> FigureSpec {
    let f = Form::of(tier, branch);
    let r = Body(Joint::Root);
    let mut v = gear::crate_box(r, [0.0, 0.22, 0.0], 0.46);
    v.extend([
        // Lid slats, a rope lashing and the team's shipping mark.
        part(r, Box(0.48, 0.04, 0.14), WOOD, [0.0, 0.41, -0.15]),
        part(r, Box(0.48, 0.04, 0.14), WOOD, [0.0, 0.41, 0.15]),
        part(r, Ring(0.014, 0.232, 360.0), LINEN, [0.0, 0.235, 0.0]).rot([0.0, 0.0, 90.0]),
        part(r, Box(0.14, 0.1, 0.02), ALLY, [-0.12, 0.3, 0.235]),
    ]);
    if f.a {
        v.extend(gear::sack(r, [0.1, 0.55, 0.02], GOLD, 0.8));
    }
    if f.a && f.t2 {
        // Black Market: gilded corners and a glowing lock.
        for (x, z) in [(-1.0, -1.0), (-1.0, 1.0), (1.0, -1.0), (1.0, 1.0)] {
            v.push(part(
                r,
                Box(0.07, 0.4, 0.07),
                GOLD,
                [x * 0.215, 0.22, z * 0.215],
            ));
        }
        v.push(part(r, Box(0.09, 0.09, 0.03), SAP, [0.08, 0.33, 0.24]).glow());
    }
    if f.b {
        v.extend(gear::crate_box(r, [-0.04, 0.57, 0.02], 0.28));
    }
    if f.b && f.t2 {
        v.extend(gear::lantern(r, [0.17, 0.5, 0.15], 1.4));
    }
    FigureSpec {
        plan: plan(tier),
        parts: v,
    }
}

fn pose(m: &Moment, pose: &mut Pose) {
    let root = &mut pose[Joint::Root.index()];
    match m.clip {
        Clip::Spawn => {
            let k = (m.t / 0.6).clamp(0.0, 1.0);
            let fall = (1.0 - k / 0.5).max(0.0);
            let bounce =
                (((k - 0.5) / 0.5).clamp(0.0, 1.0) * std::f32::consts::PI).sin() * 0.12 * (1.0 - k);
            *root = (
                Vec3::new(0.0, 0.0, 8.0 * fall),
                Vec3::Y * (1.2 * fall * fall + bounce),
            );
        }
        Clip::Hit => {
            let s = (m.t * 60.0).sin() * (1.0 - m.t / 0.25).max(0.0);
            *root = (Vec3::new(0.0, 6.0 * s, 4.0 * s), Vec3::ZERO);
        }
        Clip::Death => {
            let k = (m.t / 0.9).clamp(0.0, 1.0);
            *root = (
                Vec3::new(25.0 * k, 0.0, -30.0 * k),
                Vec3::Y * -0.5 * tools::window(k, 0.3, 1.0),
            );
        }
        _ => {
            // Every few seconds the cargo stirs: a rock and a little hop.
            let stir = tools::hold((m.clock * 0.35).rem_euclid(1.0), [0.0, 0.05, 0.12, 0.2]);
            let rock = (m.clock * 18.0).sin() * stir;
            *root = (Vec3::new(0.0, 0.0, 5.0 * rock), Vec3::Y * 0.04 * stir);
        }
    }
}
