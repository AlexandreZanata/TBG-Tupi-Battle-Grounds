//! Hexer kit: a hunched witch who leans on her skull staff, hobbles on the
//! walk, and thrusts the staff at her target so the hex leaves the skull's
//! mouth on the cast Release; she crumbles into the ground on death.

mod model;

use bevy::prelude::*;

use super::common::{moves, moves::*};
use super::{tools, Attack, Gait, Kit, Moment};
use crate::art::{
    anim::{Clip, Pose},
    rig::{Joint, Node, Socket},
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
        muzzle: |_, _| model::muzzle(),
        shot: |_, _| ShotKind::Hex,
        flash: |_, _| None,
    },
    evolve: Burst::Soul,
    durations: moves::DURATIONS,
};

pub fn pose(m: &Moment, pose: &mut Pose) {
    let p = plan(m.tier);
    match m.clip {
        Clip::Idle => {
            moves::idle(m, pose, &p, 0.2);
            hunch(pose, 1.0);
            let lantern = (m.clock * 0.9).sin();
            moves::staff(
                pose,
                &p,
                3.0 * lantern,
                0.008 * (m.clock * 0.9).cos(),
                0.012 * lantern,
            );
            // A crooked claw that beckons now and then.
            let beckon = tools::hold((m.clock * 0.3).rem_euclid(1.0), [0.0, 0.08, 0.16, 0.24]);
            tools::set(pose, Joint::UpperArmL, [-40.0 - 30.0 * beckon, 0.0, 15.0]);
            tools::set(
                pose,
                Joint::ForearmL,
                [
                    -60.0 - 40.0 * beckon * (m.clock * 14.0).sin().abs(),
                    0.0,
                    0.0,
                ],
            );
        }
        Clip::Walk => {
            moves::walk(m, pose, 10.0, 6.0);
            hunch(pose, 0.6);
            let step = (m.walk.unwrap_or(0.0) * std::f32::consts::TAU).sin();
            moves::staff(pose, &p, 8.0 + 3.0 * step, 0.015 * step.max(0.0), 0.04);
            tools::add(pose, Joint::Chest, [0.0, 3.0 * step, 0.0]);
            tools::set(pose, Joint::UpperArmL, [-25.0, 0.0, 18.0]);
            tools::set(pose, Joint::ForearmL, [-55.0, 0.0, 0.0]);
        }
        Clip::Cast => curse(m, pose, &p),
        Clip::Hit => moves::hit(m, pose, 1.0),
        Clip::Death => moves::death(m, pose, &p, Fall::Sink),
        Clip::Spawn => moves::spawn(m, pose, &p, Entry::Rise),
        Clip::Evolve => moves::evolve(m, pose, &p, Power::Spin),
        Clip::Victory => {
            pose.fill((Vec3::ZERO, Vec3::ZERO));
            let turn = tools::hold(m.t.rem_euclid(0.8), [0.0, 0.20, 0.36, 0.72]);
            hunch(pose, 1.0);
            moves::staff(pose, &p, 8.0 + 12.0 * turn, 0.06 * turn, 0.03);
            tools::add(pose, Joint::HandR, [0.0, 0.0, 18.0 * turn]);
            tools::set(pose, Joint::UpperArmL, [-30.0 - 15.0 * turn, 0.0, 20.0]);
            tools::set(pose, Joint::ForearmL, [-55.0, 0.0, 0.0]);
            tools::add(pose, Joint::Head, [6.0 * turn, -12.0 * turn, 0.0]);
            tools::plant(pose, &p, tools::stance(&p, 1.15, 0.04));
        }
        _ => moves::staff(pose, &p, 0.0, 0.0, 0.0),
    }
}

fn hunch(pose: &mut Pose, w: f32) {
    tools::add(pose, Joint::Spine, [16.0 * w, 0.0, 0.0]);
    tools::add(pose, Joint::Head, [-10.0 * w, 0.0, 0.0]);
}

/// A skull-first curse at the existing 0.22 s Cast release, followed by a crooked recovery.
fn curse(m: &Moment, pose: &mut Pose, p: &crate::art::rig::Proportions) {
    pose.fill((Vec3::ZERO, Vec3::ZERO));
    let raise = tools::hold(m.t, [0.0, 0.15, 0.24, 0.62]);
    let thrust = tools::hold(m.t, [0.15, 0.22, 0.30, 0.62]);
    hunch(pose, 1.0 - 0.35 * thrust);
    moves::staff(
        pose,
        p,
        -12.0 * raise + 58.0 * thrust,
        0.10 * raise,
        0.08 * thrust,
    );
    tools::reach(
        pose,
        true,
        Vec3::new(0.18, 0.18 + 0.05 * raise, 0.22 + 0.03 * thrust),
        Some(Quat::from_rotation_x((-15.0 * thrust).to_radians())),
        (p, Vec3::new(1.0, -0.6, -0.3)),
    );
    tools::set(pose, Joint::Head, [-8.0, -5.0 * raise, 0.0]);
    tools::face(pose, m.aim, Vec3::ZERO);
    tools::plant(pose, p, tools::stance(p, 1.2, 0.08));
}
