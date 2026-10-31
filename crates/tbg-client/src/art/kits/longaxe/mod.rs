//! Longaxe kit: a halberdier who rests his poleaxe upright, carries it on
//! the shoulder, and brings it over his own front line in a two-handed
//! chop and a ground cut reaching three tiles (branch B sweeps in a fan).

mod model;

use bevy::prelude::*;

use super::common::{moves, moves::*, Form};
use super::{tools, Attack, Gait, Kit, Moment};
use crate::art::{
    anim::{Clip, Pose},
    rig::{Joint, Node, Proportions, Socket},
};
use crate::vfx::impacts::{Burst, Impact};
use model::plan;

pub const KIT: Kit = Kit {
    figure: model::spec,
    pose,
    gait: Gait::LIGHT,
    attack: Attack::Melee {
        from: Node::Prop(Socket::WeaponR),
        tip: |tier, branch| model::tip(Form::of(tier, branch)),
        impact,
    },
    evolve: Burst::Valor,
    durations: moves::DURATIONS,
};

fn impact(tier: u8, branch: u8) -> Impact {
    let f = Form::of(tier, branch);
    match (f.a, f.b, f.t2) {
        (true, _, true) | (_, true, true) => Impact::Crush,
        _ => Impact::Cleave,
    }
}

pub fn pose(m: &Moment, pose: &mut Pose) {
    let f = Form::of(m.tier, m.branch);
    let p = plan(m.tier);
    match m.clip {
        Clip::Idle => {
            moves::idle(m, pose, &p, 0.3);
            let ready = m.aim.map_or(0.0, |a| a.weight);
            if ready > 0.0 {
                moves::pole(pose, &p, (-15.0, 20.0 * ready), 0.0, 0.32);
            } else {
                rest(pose, &p);
            }
        }
        Clip::Walk => {
            moves::walk(m, pose, 4.0, 3.0);
            shoulder(pose, &p);
        }
        Clip::AttackMelee => attack(m, pose, &p, f),
        Clip::Hit => {
            moves::hit(m, pose, 0.9);
            rest(pose, &p);
        }
        Clip::Death => {
            moves::death(m, pose, &p, Fall::Forward);
            rest(pose, &p);
        }
        Clip::Spawn => {
            moves::spawn(m, pose, &p, Entry::March);
            shoulder(pose, &p);
        }
        Clip::Evolve => {
            moves::evolve(m, pose, &p, Power::Raise);
            raised(pose, &p, (m.t / 0.9).clamp(0.0, 1.0));
        }
        Clip::Victory => {
            moves::victory(m, pose, &p, Cheer::Raise);
            raised(pose, &p, 1.0);
        }
        _ => rest(pose, &p),
    }
}

/// The poleaxe planted upright at the right side.
fn rest(pose: &mut Pose, p: &Proportions) {
    tools::reach(
        pose,
        false,
        Vec3::new(-0.26, 0.02, 0.12),
        Some(Quat::IDENTITY),
        (p, Vec3::new(-1.0, -0.6, -0.3)),
    );
}

/// Carried on the shoulder, head behind.
fn shoulder(pose: &mut Pose, p: &Proportions) {
    let grip = Quat::from_rotation_x(-55f32.to_radians());
    tools::reach(
        pose,
        false,
        Vec3::new(-0.2, 0.06, 0.18),
        Some(grip),
        (p, Vec3::new(-1.0, -0.6, -0.3)),
    );
}

/// Held high overhead (Evolve, Victory), `w` 0..1.
fn raised(pose: &mut Pose, p: &Proportions, w: f32) {
    let wrist = Vec3::new(-0.2, 0.05 + 0.45 * w, 0.1);
    tools::reach(
        pose,
        false,
        wrist,
        Some(Quat::IDENTITY),
        (p, Vec3::new(-1.0, 0.0, -0.3)),
    );
}

/// Windup (head behind the shoulder), chop over the front line at 0.30 s;
/// branch B alternates a wide horizontal sweep.
fn attack(m: &Moment, pose: &mut Pose, p: &Proportions, f: Form) {
    let k = (m.t / ATTACK).clamp(0.0, 1.0);
    let wind = tools::window(k, 0.0, 0.32);
    let cut = tools::snap(k, 0.32, 0.484);
    let back = tools::window(k, 0.645, 1.0);
    let sweep = f.b && m.variant == 1;
    let (swing, tilt) = if sweep {
        (
            (-70.0 * wind + 140.0 * cut) * (1.0 - back),
            20.0 + 65.0 * wind.max(cut) * (1.0 - back),
        )
    } else {
        (
            -10.0 * (1.0 - back),
            ((10.0 - 60.0 * wind) * (1.0 - cut) + 110.0 * cut) * (1.0 - back) + 15.0 * back,
        )
    };
    tools::add(
        pose,
        Joint::Spine,
        [(-12.0 * wind + 30.0 * cut) * (1.0 - back), swing * 0.3, 0.0],
    );
    tools::shift(
        pose,
        Joint::Hips,
        [0.0, -0.06 * cut * (1.0 - back), 0.08 * cut * (1.0 - back)],
    );
    moves::pole(pose, p, (swing, tilt), 0.12 * cut * (1.0 - back), 0.34);
    tools::face(pose, m.aim, Vec3::ZERO);
    tools::plant(pose, p, tools::stance(p, 1.35, 0.1));
}
