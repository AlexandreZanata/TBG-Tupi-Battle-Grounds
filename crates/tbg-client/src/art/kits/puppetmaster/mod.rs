//! Puppetmaster kit: it dances its marionette with small jerks of the
//! control while idle, strolls with a showman's swagger, and points its
//! control at the enemy it turns, the strings' hex leaving the cross on
//! the cast Release (confusing is its attack). Rises from the ground;
//! falls back on death.

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
        muzzle: |_, _| model::CROSS,
        shot: |_, _| ShotKind::Puppet,
        flash: |_, _| None,
    },
    evolve: Burst::Soul,
    durations: moves::DURATIONS,
};

pub fn pose(m: &Moment, pose: &mut Pose) {
    let f = Form::of(m.tier, m.branch);
    let p = plan(m.tier);
    match m.clip {
        Clip::Idle => {
            moves::idle(m, pose, &p, 0.1);
            work(m, pose, &p, f.b, 1.0);
            let greeting = tools::hold((m.clock * 0.25).rem_euclid(1.0), [0.15, 0.30, 0.44, 0.66]);
            tools::add(pose, Head, [7.0 * greeting, -10.0 * greeting, 0.0]);
        }
        Clip::Walk => {
            moves::walk(m, pose, 2.0, 5.0);
            work(m, pose, &p, f.b, 0.35);
            let step = (m.walk.unwrap_or(0.0) * std::f32::consts::TAU).sin();
            tools::add(pose, Head, [-3.0, -5.0 * step, 0.0]);
            tools::add(pose, Chest, [0.0, 3.0 * step, 0.0]);
        }
        Clip::Cast => command(m, pose, &p, f.b),
        Clip::Hit => moves::hit(m, pose, 1.0),
        Clip::Death => moves::death(m, pose, &p, Fall::Back),
        Clip::Spawn => moves::spawn(m, pose, &p, Entry::Rise),
        Clip::Evolve => moves::evolve(m, pose, &p, Power::Spin),
        Clip::Victory => {
            pose.fill((Vec3::ZERO, Vec3::ZERO));
            let bow = tools::hold(m.t.rem_euclid(0.8), [0.12, 0.30, 0.48, 0.76]);
            tools::set(pose, Spine, [8.0 * bow, 0.0, 0.0]);
            tools::set(pose, Head, [10.0 * bow, 0.0, 0.0]);
            let turn = Quat::from_rotation_x((-8.0 * bow).to_radians());
            for left in [false, true] {
                if !left || f.b {
                    tools::reach(
                        pose,
                        left,
                        Vec3::new(if left { 0.16 } else { -0.16 }, 0.26 + 0.025 * bow, 0.28),
                        Some(turn),
                        (&p, Vec3::new(if left { 1.0 } else { -1.0 }, -0.5, -0.3)),
                    );
                } else {
                    tools::set(pose, UpperArmL, [-25.0, 0.0, 20.0]);
                    tools::set(pose, ForearmL, [-40.0, 0.0, 0.0]);
                }
            }
            tools::plant(pose, &p, tools::stance(&p, 1.05, 0.05));
        }
        _ => work(m, pose, &p, f.b, 0.0),
    }
}

/// Hold the control before the chest and jerk it so the puppet dances;
/// the second control (branch B) answers on the off beat.
fn work(m: &Moment, pose: &mut Pose, p: &Proportions, both: bool, dance: f32) {
    let jerk = |phase: f32| {
        let a = m.clock * 3.2 + phase;
        Quat::from_rotation_z((12.0 * a.sin() * dance).to_radians())
            * Quat::from_rotation_x((10.0 * (a * 1.7).sin() * dance).to_radians())
    };
    let greeting = tools::hold((m.clock * 0.25).rem_euclid(1.0), [0.15, 0.30, 0.44, 0.66]);
    let bob = (0.02 * (m.clock * 6.4).sin().abs() + 0.035 * greeting) * dance;
    tools::reach(
        pose,
        false,
        Vec3::new(-0.16, 0.3 + bob, 0.3),
        Some(jerk(0.0)),
        (p, Vec3::new(-1.0, -0.5, -0.3)),
    );
    if both {
        tools::reach(
            pose,
            true,
            Vec3::new(0.16, 0.3 + bob, 0.3),
            Some(jerk(1.6)),
            (p, Vec3::new(1.0, -0.5, -0.3)),
        );
    }
}

/// The control leads the existing 0.22 s hex; the marionette remains below its cross.
fn command(m: &Moment, pose: &mut Pose, p: &Proportions, both: bool) {
    pose.fill((Vec3::ZERO, Vec3::ZERO));
    let gather = tools::hold(m.t, [0.0, 0.14, 0.18, 0.30]);
    let point = tools::hold(m.t, [0.14, 0.22, 0.32, 0.62]);
    tools::set(pose, Head, [4.0 * gather - 3.0 * point, -5.0 * gather, 0.0]);
    for left in [false, true] {
        if left && !both {
            tools::reach(
                pose,
                true,
                Vec3::new(0.20, 0.20, 0.18),
                Some(Quat::IDENTITY),
                (p, Vec3::new(1.0, -0.5, -0.3)),
            );
            continue;
        }
        let turn = Quat::from_rotation_z((if left { -5.0 } else { 5.0 } * gather).to_radians());
        tools::reach(
            pose,
            left,
            Vec3::new(
                if left { 0.16 } else { -0.16 },
                0.25 + 0.04 * gather,
                0.25 + 0.05 * point,
            ),
            Some(turn),
            (p, Vec3::new(if left { 1.0 } else { -1.0 }, -0.5, -0.3)),
        );
    }
    tools::face(pose, m.aim, Vec3::ZERO);
    tools::plant(pose, p, tools::stance(p, 1.1, 0.06));
}
