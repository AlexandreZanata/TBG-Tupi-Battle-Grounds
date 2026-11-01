//! Leechroot kit: a hunched root-beast whose maw weaves as it hunts. It
//! lashes its whip root across the target, draining it, scuttles low on
//! the walk, claws up out of the soil and collapses back into it.

mod model;

use super::common::{moves, moves::*, Form};
use super::{tools, Attack, Gait, Kit, Moment};
use crate::art::{
    anim::{Clip, Pose},
    rig::{Joint, Node},
};
use crate::vfx::impacts::{Burst, Impact};
use model::plan;

pub const KIT: Kit = Kit {
    anatomy: None,
    figure: model::spec,
    pose,
    gait: Gait::LIGHT,
    attack: Attack::Melee {
        from: Node::Body(Joint::HandR),
        tip: |tier, _| model::tip(&plan(tier)),
        impact: |tier, branch| {
            if Form::of(tier, branch).a {
                Impact::Bite
            } else {
                Impact::Drain
            }
        },
    },
    evolve: Burst::Bloom,
    durations: moves::DURATIONS,
};

pub fn pose(m: &Moment, pose: &mut Pose) {
    let p = plan(m.tier);
    match m.clip {
        Clip::Idle => {
            moves::idle(m, pose, &p, 0.2);
            hunch(pose, 1.0);
            // The maw weaves, searching.
            tools::add(
                pose,
                Joint::Head,
                [
                    8.0 * tools::wave(m.clock, 1.1, 0.3),
                    25.0 * tools::wave(m.clock, 0.7, 1.7),
                    10.0 * tools::wave(m.clock, 0.9, 2.4),
                ],
            );
            tools::set(
                pose,
                Joint::HandL,
                [12.0 * (m.clock * 0.8).sin(), 0.0, 14.0],
            );
            tools::set(
                pose,
                Joint::HandR,
                [-8.0 * (m.clock * 0.8).sin(), 0.0, -8.0],
            );
        }
        Clip::Walk => {
            moves::walk(m, pose, 14.0, 5.0);
            hunch(pose, 0.6);
            let step = (m.walk.unwrap_or(0.0) * std::f32::consts::TAU).sin();
            tools::set(pose, Joint::UpperArmL, [-5.0 + 8.0 * step, 0.0, 16.0]);
            tools::set(pose, Joint::UpperArmR, [-5.0 - 8.0 * step, 0.0, -16.0]);
            tools::set(pose, Joint::ForearmL, [-12.0, 0.0, 0.0]);
            tools::set(pose, Joint::ForearmR, [-12.0, 0.0, 0.0]);
        }
        Clip::AttackMelee => {
            let load = tools::hold(m.t, [0.0, 0.17, 0.23, 0.30]);
            let lash = tools::hold(m.t, [0.23, 0.30, 0.38, 0.62]);
            tools::set(pose, Joint::Spine, [18.0 + 6.0 * lash, 0.0, 0.0]);
            tools::set(pose, Joint::Head, [-12.0, 5.0 * load, 0.0]);
            tools::set(
                pose,
                Joint::UpperArmR,
                [-55.0 - 25.0 * lash, 25.0 * load - 30.0 * lash, -22.0 * load],
            );
            tools::set(pose, Joint::ForearmR, [-65.0 + 45.0 * lash, 0.0, 0.0]);
            tools::set(pose, Joint::HandR, [25.0 * load - 10.0 * lash, 0.0, -10.0]);
            tools::set(pose, Joint::UpperArmL, [-20.0, 0.0, 18.0]);
            tools::set(pose, Joint::ForearmL, [-20.0 - 25.0 * lash, 0.0, 0.0]);
            tools::face(pose, m.aim, bevy::prelude::Vec3::ZERO);
            tools::plant(pose, &p, tools::stance(&p, 1.25, 0.06));
        }
        Clip::Hit => {
            moves::hit(m, pose, 1.0);
            hunch(pose, 0.5);
        }
        Clip::Death => moves::death(m, pose, &p, Fall::Sink),
        Clip::Spawn => moves::spawn(m, pose, &p, Entry::Rise),
        Clip::Evolve => moves::evolve(m, pose, &p, Power::Burst),
        Clip::Victory => {
            let curl = 0.5 - 0.5 * (m.t / 0.8 * std::f32::consts::TAU).cos();
            hunch(pose, 0.85);
            tools::set(
                pose,
                Joint::Head,
                [-12.0, 10.0 * (m.t / 0.8 * std::f32::consts::TAU).sin(), 0.0],
            );
            tools::set(pose, Joint::UpperArmL, [-25.0, 0.0, 18.0]);
            tools::set(pose, Joint::UpperArmR, [-30.0, 0.0, -18.0]);
            tools::set(pose, Joint::ForearmL, [-35.0 - 25.0 * curl, 0.0, 0.0]);
            tools::set(
                pose,
                Joint::ForearmR,
                [-35.0 - 25.0 * (1.0 - curl), 0.0, 0.0],
            );
            tools::set(pose, Joint::HandL, [30.0 * curl, 0.0, 25.0]);
            tools::set(pose, Joint::HandR, [30.0 * (1.0 - curl), 0.0, -25.0]);
            tools::plant(pose, &p, tools::stance(&p, 1.2, 0.02));
        }
        _ => hunch(pose, 1.0),
    }
}

/// A predatory crouch: back bent, head thrust forward.
fn hunch(pose: &mut Pose, w: f32) {
    tools::add(pose, Joint::Spine, [24.0 * w, 0.0, 0.0]);
    tools::add(pose, Joint::Head, [-14.0 * w, 0.0, 0.0]);
    tools::shift(pose, Joint::Hips, [0.0, -0.03 * w, 0.0]);
}
