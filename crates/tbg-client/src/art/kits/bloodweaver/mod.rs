//! Bloodweaver kit: she idles weaving slow circles with her free hand,
//! glides on the walk, and strikes the enemy with a blood spindle; stolen
//! vitality streams from that enemy to the wounded ally (`sync/actions.rs`). She rises out of the
//! ground and sinks back into it.

mod model;

use bevy::prelude::*;

use super::common::{moves, moves::*, Form};
use super::{tools, Attack, Gait, Kit, Moment};
use crate::art::{
    anim::{Clip, Pose},
    rig::{Joint::*, Node, Proportions, Socket},
};
use crate::vfx::{impacts::Burst, shots::ShotKind};
use model::plan;

pub const KIT: Kit = Kit {
    anatomy: None,
    figure: model::spec,
    pose,
    gait: Gait::LIGHT,
    attack: Attack::Shot {
        loaded: None,
        from: Node::Prop(Socket::WeaponR),
        muzzle: |_, _| model::spindle_top(),
        shot: |_, _| ShotKind::BloodBolt,
        flash: |_, _| None,
    },
    evolve: Burst::Soul,
    durations: moves::DURATIONS,
};

pub fn pose(m: &Moment, pose: &mut Pose) {
    let p = plan(m.tier);
    let form = Form::of(m.tier, m.branch);
    match m.clip {
        Clip::Idle => {
            moves::idle(m, pose, &p, 0.1);
            staff(pose, &p);
            let a = m.clock * 1.6;
            let at = Vec3::new(0.2 + 0.06 * a.cos(), 0.22 + 0.06 * a.sin(), 0.24);
            let turn = if form.b && form.t2 {
                Quat::from_rotation_z((-5.0 * a.sin()).to_radians())
            } else {
                Quat::IDENTITY
            };
            tools::reach(pose, true, at, Some(turn), (&p, Vec3::new(1.0, -0.6, -0.3)));
            tools::add(pose, HandR, [0.0, 8.0 * a.sin(), 0.0]);
            tools::add(pose, Head, [3.0 * a.cos(), 5.0 * a.sin(), 0.0]);
        }
        Clip::Walk => {
            moves::walk(m, pose, 3.0, 2.0);
            staff(pose, &p);
            let step = (m.walk.unwrap_or(0.0) * std::f32::consts::TAU).sin();
            tools::reach(
                pose,
                true,
                Vec3::new(0.22, 0.13 + 0.01 * step, 0.15),
                Some(Quat::IDENTITY),
                (&p, Vec3::new(1.0, -0.6, -0.3)),
            );
            tools::add(pose, Head, [-2.0, -4.0 * step, 0.0]);
        }
        Clip::Cast => weave(m, pose, &p, form),
        Clip::Hit => moves::hit(m, pose, 1.0),
        Clip::Death => moves::death(m, pose, &p, Fall::Sink),
        Clip::Spawn => moves::spawn(m, pose, &p, Entry::Rise),
        Clip::Evolve => moves::evolve(m, pose, &p, Power::Spin),
        Clip::Victory => {
            pose.fill((Vec3::ZERO, Vec3::ZERO));
            let t = m.t.rem_euclid(0.8);
            let bow = tools::hold(t, [0.15, 0.34, 0.50, 0.76]);
            let circle = t / 0.8 * std::f32::consts::TAU;
            tools::set(pose, Spine, [6.0 * bow, 0.0, 0.0]);
            tools::set(pose, Head, [8.0 * bow, 0.0, 0.0]);
            staff(pose, &p);
            let turn = Quat::from_rotation_x((-6.0 * bow).to_radians());
            tools::reach(
                pose,
                true,
                Vec3::new(
                    0.20 + 0.035 * circle.sin(),
                    0.20 + 0.025 * (circle * 2.0).sin(),
                    0.18,
                ),
                Some(turn),
                (&p, Vec3::new(1.0, -0.6, -0.3)),
            );
            tools::plant(pose, &p, tools::stance(&p, 1.1, 0.02));
        }
        _ => staff(pose, &p),
    }
}

/// The spindle staff held upright at the right side.
fn staff(pose: &mut Pose, p: &Proportions) {
    tools::reach(
        pose,
        false,
        Vec3::new(-0.24, 0.08, 0.12),
        Some(Quat::IDENTITY),
        (p, Vec3::new(-1.0, -0.5, -0.4)),
    );
}

/// Gather at the spindle, release the Siphon at 0.22, then wind the free hand back in.
fn weave(m: &Moment, pose: &mut Pose, p: &Proportions, form: Form) {
    pose.fill((Vec3::ZERO, Vec3::ZERO));
    let gather = tools::hold(m.t, [0.0, 0.14, 0.18, 0.28]);
    let send = tools::hold(m.t, [0.16, 0.22, 0.32, 0.62]);
    tools::set(pose, Chest, [4.0 * send, 0.0, 0.0]);
    tools::set(pose, Head, [-3.0 * send, -4.0 * gather, 0.0]);
    let right = Quat::from_rotation_x((-8.0 * gather + 48.0 * send).to_radians());
    tools::reach(
        pose,
        false,
        Vec3::new(-0.22, 0.10 + 0.07 * gather, 0.14 + 0.045 * send),
        Some(right),
        (p, Vec3::new(-1.0, -0.5, -0.4)),
    );
    let left = if form.b && form.t2 {
        Quat::from_rotation_x((22.0 * send - 4.0 * gather).to_radians())
    } else {
        Quat::from_rotation_x((-4.0 * send).to_radians())
    };
    tools::reach(
        pose,
        true,
        Vec3::new(
            0.21 + 0.025 * send,
            0.16 + 0.04 * gather,
            0.16 + 0.07 * send,
        ),
        Some(left),
        (p, Vec3::new(1.0, -0.6, -0.3)),
    );
    tools::face(pose, m.aim, Vec3::ZERO);
    tools::plant(pose, p, tools::stance(p, 1.1, 0.04));
}
