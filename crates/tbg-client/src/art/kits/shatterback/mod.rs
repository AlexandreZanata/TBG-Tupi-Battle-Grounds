//! Shatterback: a heavy terrestrial turtle with a domed segmented shell,
//! grounded paw strikes and a retreat into its shell on death.

pub(crate) mod model;
use bevy::prelude::*;

use super::common::{moves, Form};
use super::{tools, Anatomy, Attack, Gait, Kit, Moment};
use crate::art::{
    anim::{Clip, Pose},
    rig::{Joint, Node},
};
use crate::vfx::impacts::{Burst, Impact};
use model::plan;

static ANATOMY: Anatomy = Anatomy {
    rest: model::rest,
    parent: Node::parent,
    springs: false,
    planted_legs: true,
};

pub const KIT: Kit = Kit {
    anatomy: Some(&ANATOMY),
    figure: model::spec,
    pose,
    gait: Gait::HEAVY,
    attack: Attack::Melee {
        from: Node::Body(Joint::HandR),
        tip: |_, _| model::tip(),
        impact: |tier, branch| {
            if Form::of(tier, branch).a {
                Impact::Shards
            } else {
                Impact::Crush
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
            moves::idle(m, pose, &p, 1.0);
            let heave = (m.clock * 1.2).sin();
            tools::set(pose, Joint::Spine, [2.0, 0.0, 0.0]);
            tools::set(pose, Joint::Chest, [1.5 * heave, 0.0, 0.0]);
            tools::set(pose, Joint::Head, [-3.0, 4.0 * (m.clock * 0.4).sin(), 0.0]);
        }
        Clip::Walk => {
            let step = (m.walk.unwrap_or(0.0) * std::f32::consts::TAU).sin();
            tools::set(pose, Joint::Spine, [3.0, 0.0, 2.0 * step]);
            tools::set(pose, Joint::Chest, [-2.0 * step.abs(), 0.0, -1.0 * step]);
            tools::set(pose, Joint::Head, [-4.0, 0.0, -2.0 * step]);
        }
        Clip::AttackMelee => {
            let load = tools::hold(m.t, [0.0, 0.18, 0.23, 0.30]);
            let drive = tools::hold(m.t, [0.23, 0.30, 0.38, 0.62]);
            tools::set(pose, Joint::Spine, [2.0 + 4.0 * drive, 0.0, -3.0 * load]);
            tools::set(pose, Joint::Chest, [0.0, 5.0 * drive, 0.0]);
            tools::set(pose, Joint::Head, [-5.0, 0.0, 0.0]);
            tools::face(pose, m.aim, Vec3::ZERO);
            tools::plant(pose, &p, tools::stance(&p, 1.5, 0.02));
        }
        Clip::Hit => {
            pose.fill((Vec3::ZERO, Vec3::ZERO));
            let recoil = tools::hold(m.t, [0.0, 0.04, 0.08, 0.25]);
            tools::set(pose, Joint::Chest, [-4.0 * recoil, 0.0, 0.0]);
            tools::shift(pose, Joint::Head, [0.0, 0.0, -0.06 * recoil]);
        }
        Clip::Death => {
            pose.fill((Vec3::ZERO, Vec3::ZERO));
            let retreat = tools::window(m.t, 0.0, 0.65);
            tools::shift(pose, Joint::Hips, [0.0, -0.08 * retreat, 0.0]);
            tools::shift(pose, Joint::Head, [0.0, -0.02 * retreat, -0.18 * retreat]);
            tools::set(pose, Joint::Head, [8.0 * retreat, 0.0, 0.0]);
        }
        Clip::Spawn => {
            pose.fill((Vec3::ZERO, Vec3::ZERO));
            let emerge = tools::window(m.t, 0.0, 0.60);
            tools::shift(pose, Joint::Root, [0.0, -0.25 * (1.0 - emerge), 0.0]);
            tools::shift(pose, Joint::Head, [0.0, 0.0, -0.12 * (1.0 - emerge)]);
        }
        Clip::Evolve => {
            pose.fill((Vec3::ZERO, Vec3::ZERO));
            let shake = (m.t * 18.0).sin() * tools::hold(m.t, [0.0, 0.15, 0.45, 0.9]);
            tools::set(pose, Joint::Chest, [0.0, 0.0, 3.0 * shake]);
            tools::set(pose, Joint::Head, [-3.0, -4.0 * shake, 0.0]);
        }
        Clip::Victory => {
            let proud = 0.5 - 0.5 * (m.t / 0.8 * std::f32::consts::TAU).cos();
            pose[Joint::Root.index()] = (Vec3::ZERO, Vec3::ZERO);
            tools::set(
                pose,
                Joint::Spine,
                [1.0, 0.0, 2.0 * (m.t / 0.8 * std::f32::consts::TAU).sin()],
            );
            tools::set(pose, Joint::Chest, [-3.0 * proud, 0.0, 0.0]);
            tools::set(
                pose,
                Joint::Head,
                [
                    -6.0 * proud,
                    5.0 * (m.t / 0.8 * std::f32::consts::TAU).sin(),
                    0.0,
                ],
            );
            tools::plant(pose, &p, tools::stance(&p, 1.5, 0.0));
        }
        _ => stoop(pose, 1.0),
    }
    forepaws(m, pose, &p);
}

/// Front paws use the existing arm IK; their targets stay on the local ground.
fn forepaws(m: &Moment, pose: &mut Pose, p: &crate::art::rig::Proportions) {
    let chest = tools::chest(pose, p);
    let (_, turn, _) = chest.to_scale_rotation_translation();
    let step = m
        .walk
        .map_or(0.0, |phase| (phase * std::f32::consts::TAU).sin());
    let drive = if m.clip == Clip::AttackMelee {
        tools::hold(m.t, [0.23, 0.30, 0.38, 0.62])
    } else {
        0.0
    };
    for (left, sign) in [(true, 1.0), (false, -1.0)] {
        let stride = if m.clip == Clip::Walk {
            step * sign
        } else {
            0.0
        };
        let strike = if left { 0.0 } else { drive };
        let ground = Vec3::new(
            sign * 0.34,
            0.09 + 0.025 * stride.max(0.0) + 0.06 * strike,
            0.25 + 0.055 * stride + 0.10 * strike,
        );
        tools::reach(
            pose,
            left,
            chest.inverse().transform_point3(ground),
            Some(turn.inverse()),
            (p, Vec3::new(sign, -0.5, -0.5)),
        );
    }
}

/// A low turtle stance keeps the domed shell above its short legs.
fn stoop(pose: &mut Pose, w: f32) {
    tools::add(pose, Joint::Spine, [14.0 * w, 0.0, 0.0]);
    tools::add(pose, Joint::Head, [-12.0 * w, 0.0, 0.0]);
    tools::add(pose, Joint::UpperArmL, [-10.0 * w, 0.0, 12.0 * w]);
    tools::add(pose, Joint::UpperArmR, [-10.0 * w, 0.0, -12.0 * w]);
}
