//! Oilpress kit: a walking engine. It idles with a piston pump in its
//! chest, plods with the heaviest stomp in the game, slams both press
//! plates down on its target, sinks to its knees on death and lands from
//! its drop with a quake.

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
    gait: Gait::HEAVY,
    attack: Attack::Melee {
        from: Node::Body(Joint::HandR),
        tip: |_, _| model::tip(),
        impact,
    },
    evolve: Burst::Forge,
    durations: moves::DURATIONS,
};

fn impact(tier: u8, branch: u8) -> Impact {
    if Form::of(tier, branch).b {
        Impact::Oil
    } else {
        Impact::Crush
    }
}

pub fn pose(m: &Moment, pose: &mut Pose) {
    let p = plan(m.tier);
    match m.clip {
        Clip::Idle => {
            moves::idle(m, pose, &p, 1.0);
            // The engine's pump: a short double beat in the chest.
            let pump = (m.clock * 4.5).sin().max(0.0).powi(4);
            tools::shift(pose, Joint::Chest, [0.0, 0.012 * pump, 0.0]);
            tools::add(pose, Joint::Head, [-3.0 * pump, 0.0, 0.0]);
            arms_out(pose, m.aim.map_or(0.0, |a| a.weight));
            tools::add(pose, Joint::ForearmL, [-5.0 * pump, 0.0, 0.0]);
            tools::add(pose, Joint::ForearmR, [-5.0 * pump, 0.0, 0.0]);
        }
        Clip::Walk => {
            moves::walk(m, pose, 2.0, 7.0);
            arms_out(pose, 0.0);
            let step = (m.walk.unwrap_or(0.0) * std::f32::consts::TAU).sin();
            tools::set(pose, Joint::UpperArmL, [-8.0 + 3.0 * step, 0.0, 16.0]);
            tools::set(pose, Joint::UpperArmR, [-8.0 - 3.0 * step, 0.0, -16.0]);
            tools::set(pose, Joint::Head, [-3.0, 0.0, -2.0 * step]);
        }
        Clip::AttackMelee => {
            let load = tools::hold(m.t, [0.0, 0.18, 0.23, 0.30]);
            let press = tools::hold(m.t, [0.23, 0.30, 0.40, 0.62]);
            for (upper, fore, sign) in [
                (Joint::UpperArmL, Joint::ForearmL, 1.0),
                (Joint::UpperArmR, Joint::ForearmR, -1.0),
            ] {
                tools::set(
                    pose,
                    upper,
                    [-25.0 - 100.0 * load - 35.0 * press, 0.0, 16.0 * sign],
                );
                tools::set(pose, fore, [-25.0 - 20.0 * load + 20.0 * press, 0.0, 0.0]);
            }
            tools::set(pose, Joint::Chest, [-5.0 * load + 10.0 * press, 0.0, 0.0]);
            tools::shift(pose, Joint::Hips, [0.0, -0.045 * press, 0.0]);
            tools::face(pose, m.aim, bevy::prelude::Vec3::ZERO);
            tools::plant(pose, &p, tools::stance(&p, 1.45, 0.04));
        }
        Clip::Hit => {
            moves::hit(m, pose, 0.35);
            arms_out(pose, 0.0);
        }
        Clip::Death => moves::death(m, pose, &p, Fall::Kneel),
        Clip::Spawn => moves::spawn(m, pose, &p, Entry::Drop),
        Clip::Evolve => moves::evolve(m, pose, &p, Power::Burst),
        Clip::Victory => {
            let cycle = (m.t / 0.8 * std::f32::consts::TAU).sin();
            tools::set(pose, Joint::Chest, [3.0, 0.0, 2.0 * cycle]);
            tools::set(pose, Joint::Head, [-4.0 * cycle.abs(), 0.0, 0.0]);
            tools::set(
                pose,
                Joint::UpperArmL,
                [-35.0 - 25.0 * cycle.max(0.0), 0.0, 20.0],
            );
            tools::set(
                pose,
                Joint::UpperArmR,
                [-35.0 - 25.0 * (-cycle).max(0.0), 0.0, -20.0],
            );
            tools::set(pose, Joint::ForearmL, [-40.0, 0.0, 0.0]);
            tools::set(pose, Joint::ForearmR, [-40.0, 0.0, 0.0]);
            tools::plant(pose, &p, tools::stance(&p, 1.35, 0.0));
        }
        _ => arms_out(pose, 0.0),
    }
}

/// The heavy arms hang clear of the boiler, plates forward when engaged.
fn arms_out(pose: &mut Pose, ready: f32) {
    tools::add(pose, Joint::UpperArmL, [-15.0 * ready, 0.0, 16.0]);
    tools::add(pose, Joint::UpperArmR, [-15.0 * ready, 0.0, -16.0]);
    tools::add(pose, Joint::ForearmL, [-25.0 - 30.0 * ready, 0.0, 0.0]);
    tools::add(pose, Joint::ForearmR, [-25.0 - 30.0 * ready, 0.0, 0.0]);
}
