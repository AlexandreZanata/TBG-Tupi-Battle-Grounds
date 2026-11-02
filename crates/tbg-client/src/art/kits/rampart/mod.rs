//! Rampart kit: an ogre who plants his wall before him, plods behind it,
//! and punches (or rams) round its edge while keeping it up; he drops to
//! his knees behind it when he falls and lands from his drop with a quake.

mod model;

use bevy::prelude::*;

use super::common::{moves, moves::*};
use super::{tools, Attack, Gait, Kit, Moment};
use crate::art::{
    anim::{Clip, Pose},
    rig::{Joint, Node, Proportions},
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
        impact: |_, _| Impact::Crush,
    },
    evolve: Burst::Forge,
    durations: moves::DURATIONS,
};

pub fn pose(m: &Moment, pose: &mut Pose) {
    let p = plan(m.tier);
    match m.clip {
        Clip::Idle => {
            moves::idle(m, pose, &p, 0.9);
            let peek = tools::hold((m.clock * 0.22).fract(), [0.1, 0.25, 0.45, 0.65]);
            tools::add(pose, Joint::Head, [-4.0, -16.0 * peek, -4.0 * peek]);
            tools::set(pose, Joint::ForearmR, [-35.0, 0.0, 0.0]);
            tools::add(pose, Joint::Chest, [4.0, -3.0 * peek, 0.0]);
        }
        Clip::Walk => {
            let step = (m.walk.unwrap_or(0.0) * std::f32::consts::TAU).sin();
            moves::walk(m, pose, 5.0, 2.0);
            tools::set(pose, Joint::Chest, [3.0, 0.0, -1.5 * step]);
            tools::set(pose, Joint::UpperArmR, [-12.0 + 5.0 * step, 0.0, -10.0]);
            tools::set(pose, Joint::ForearmR, [-35.0, 0.0, 0.0]);
        }
        Clip::AttackMelee => {
            let load = tools::hold(m.t, [0.0, 0.18, 0.23, 0.30]);
            let drive = tools::hold(m.t, [0.23, 0.30, 0.37, 0.62]);
            tools::set(pose, Joint::Chest, [5.0 + 5.0 * drive, 5.0 * drive, 0.0]);
            tools::reach(
                pose,
                false,
                Vec3::new(-0.30, 0.16 + 0.05 * load, 0.10 - 0.05 * load + 0.16 * drive),
                Some(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                (&p, Vec3::new(-1.0, -0.4, -0.4)),
            );
            tools::face(pose, m.aim, Vec3::ZERO);
            tools::plant(pose, &p, tools::stance(&p, 1.35, 0.06));
        }
        Clip::Hit => moves::hit(m, pose, 0.4),
        Clip::Death => moves::death(m, pose, &p, Fall::Kneel),
        Clip::Spawn => moves::spawn(m, pose, &p, Entry::Drop),
        Clip::Evolve => moves::evolve(m, pose, &p, Power::Burst),
        Clip::Victory => {
            let nod = 0.5 - 0.5 * (m.t * std::f32::consts::TAU / 0.8).cos();
            tools::set(pose, Joint::Chest, [3.0 - 3.0 * nod, 0.0, 0.0]);
            tools::set(pose, Joint::Head, [-8.0 * nod, -8.0, 0.0]);
            tools::set(pose, Joint::UpperArmR, [-35.0 - 40.0 * nod, 0.0, -18.0]);
            tools::set(pose, Joint::ForearmR, [-75.0, 0.0, 0.0]);
            tools::plant(pose, &p, tools::stance(&p, 1.25, 0.04));
        }
        _ => {}
    }
    if m.clip != Clip::Death {
        hold_wall(pose, &p);
    }
}

/// The left hand stays on the wall's grip whatever the body does.
fn hold_wall(pose: &mut Pose, p: &Proportions) {
    tools::reach(
        pose,
        true,
        model::grip(p),
        None,
        (p, Vec3::new(1.0, -0.6, -0.4)),
    );
}
