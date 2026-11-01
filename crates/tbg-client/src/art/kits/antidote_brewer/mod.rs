//! Antidote Brewer kit: it swirls its flask while idle, hurries with the
//! flask held up, and tosses the cure at the ally it cleanses (its brew is
//! its attack, so the flask leaves the hand on the cast Release). Pops up
//! out of the soil.

mod model;

use bevy::prelude::*;

use super::common::{moves, moves::*};
use super::{tools, Attack, Gait, Kit, Moment};
use crate::art::{
    anim::{Clip, Pose},
    rig::{Joint, Node, Proportions, Socket},
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
        muzzle: |_, _| model::FLASK,
        shot: |_, _| ShotKind::Flask,
        flash: |_, _| None,
    },
    evolve: Burst::Bloom,
    durations: moves::DURATIONS,
};

pub fn pose(m: &Moment, pose: &mut Pose) {
    let p = plan(m.tier);
    match m.clip {
        Clip::Idle => {
            moves::idle(m, pose, &p, 0.1);
            swirl(m, pose, &p);
            tools::reach(
                pose,
                true,
                Vec3::new(0.16, 0.10, 0.16),
                None,
                (&p, Vec3::new(1.0, -0.6, -0.3)),
            );
        }
        Clip::Walk => {
            moves::walk(m, pose, 6.0, 3.0);
            tools::set(pose, Joint::UpperArmR, [-60.0, 0.0, -10.0]);
            tools::set(pose, Joint::ForearmR, [-70.0, 0.0, 0.0]);
            let step = (m.walk.unwrap_or(0.0) * std::f32::consts::TAU).sin();
            tools::set(pose, Joint::UpperArmL, [-15.0 + 10.0 * step, 0.0, 14.0]);
            tools::set(pose, Joint::ForearmL, [-25.0, 0.0, 0.0]);
            tools::set(pose, Joint::Head, [4.0, -8.0, -2.0 * step]);
        }
        Clip::Cast => {
            let measure = tools::hold(m.t, [0.0, 0.12, 0.17, 0.26]);
            let offer = tools::hold(m.t, [0.17, 0.22, 0.33, 0.62]);
            tools::set(
                pose,
                Joint::Chest,
                [-3.0 * measure + 6.0 * offer, -5.0 * measure, 0.0],
            );
            tools::set(pose, Joint::Head, [6.0 * measure, -8.0 * measure, 0.0]);
            tools::reach(
                pose,
                false,
                Vec3::new(
                    -0.18 + 0.04 * offer,
                    0.27 + 0.03 * measure,
                    0.14 + 0.12 * offer,
                ),
                Some(Quat::from_rotation_x(
                    (15.0 * measure + 30.0 * offer).to_radians(),
                )),
                (&p, Vec3::new(-1.0, -0.5, -0.3)),
            );
            tools::reach(
                pose,
                true,
                Vec3::new(0.17, 0.12, 0.15),
                None,
                (&p, Vec3::new(1.0, -0.6, -0.3)),
            );
            tools::face(pose, m.aim, Vec3::ZERO);
            tools::plant(pose, &p, tools::stance(&p, 1.1, 0.05));
        }
        Clip::Hit => moves::hit(m, pose, 1.0),
        Clip::Death => moves::death(m, pose, &p, Fall::Kneel),
        Clip::Spawn => moves::spawn(m, pose, &p, Entry::Pop),
        Clip::Evolve => moves::evolve(m, pose, &p, Power::Raise),
        Clip::Victory => {
            let toast = 0.5 - 0.5 * (m.t / 0.8 * std::f32::consts::TAU).cos();
            tools::set(
                pose,
                Joint::Chest,
                [-2.0 * toast, 0.0, 3.0 * (m.t * 7.85).sin()],
            );
            tools::set(pose, Joint::Head, [-10.0 * toast, -10.0, 0.0]);
            tools::reach(
                pose,
                false,
                Vec3::new(-0.20, 0.26 + 0.09 * toast, 0.17),
                Some(Quat::IDENTITY),
                (&p, Vec3::new(-1.0, -0.5, -0.3)),
            );
            tools::reach(
                pose,
                true,
                Vec3::new(0.16, 0.10 + 0.02 * toast, 0.16),
                None,
                (&p, Vec3::new(1.0, -0.6, -0.3)),
            );
            tools::plant(pose, &p, tools::stance(&p, 1.1, 0.0));
        }
        _ => {}
    }
}

/// The flask held up before the eyes and swirled in small circles.
fn swirl(m: &Moment, pose: &mut Pose, p: &Proportions) {
    let a = m.clock * 3.0;
    let at = Vec3::new(-0.1 + 0.025 * a.cos(), 0.3 + 0.02 * a.sin(), 0.22);
    tools::reach(pose, false, at, None, (p, Vec3::new(-1.0, -0.6, -0.3)));
    tools::add(pose, Joint::Head, [8.0, -10.0, 0.0]);
}
