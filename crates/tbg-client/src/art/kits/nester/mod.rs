//! Nester kit: a plump nest mother who rocks and pats her belly while
//! idle, waddles on the walk, and lobs a seed overhand on the cast
//! Release; she melts back into the soil on death and rises from it.

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
        shot: |_, _| ShotKind::Seed,
        flash: |_, _| None,
    },
    evolve: Burst::Bloom,
    durations: moves::DURATIONS,
};

pub fn pose(m: &Moment, pose: &mut Pose) {
    let p = plan(m.tier);
    match m.clip {
        Clip::Idle => {
            moves::idle(m, pose, &p, 0.5);
            let rock = (m.clock * 1.4).sin();
            tools::add(pose, Joint::Spine, [0.0, 0.0, 5.0 * rock]);
            let pat = (m.clock * 5.0).sin().max(0.0)
                * tools::hold((m.clock * 0.25).rem_euclid(1.0), [0.0, 0.05, 0.25, 0.3]);
            let belly = Vec3::new(0.1, 0.12 + 0.03 * pat, 0.24);
            tools::reach(pose, true, belly, None, (&p, Vec3::new(1.0, -0.6, -0.2)));
            let check = tools::hold((m.clock * 0.2).fract(), [0.15, 0.28, 0.40, 0.60]);
            tools::add(pose, Joint::Head, [-5.0 * check, 22.0 * check, 0.0]);
            tools::set(pose, Joint::ForearmR, [-22.0, 0.0, 0.0]);
        }
        Clip::Walk => {
            moves::walk(m, pose, 2.0, 7.0);
            let step = (m.walk.unwrap_or(0.0) * std::f32::consts::TAU).sin();
            tools::reach(
                pose,
                true,
                Vec3::new(0.17, 0.10, 0.22),
                None,
                (&p, Vec3::new(1.0, -0.7, -0.3)),
            );
            tools::set(pose, Joint::UpperArmR, [-12.0 + 8.0 * step, 0.0, -18.0]);
            tools::set(pose, Joint::ForearmR, [-30.0, 0.0, 0.0]);
            tools::set(pose, Joint::Head, [-3.0, 0.0, -3.0 * step]);
        }
        Clip::Cast => {
            let gather = tools::hold(m.t, [0.0, 0.14, 0.17, 0.25]);
            let offer = tools::hold(m.t, [0.17, 0.22, 0.32, 0.62]);
            tools::set(
                pose,
                Joint::Chest,
                [-4.0 * gather + 6.0 * offer, -7.0 * gather, 0.0],
            );
            tools::reach(
                pose,
                false,
                Vec3::new(
                    -0.22 + 0.07 * offer,
                    0.18 + 0.11 * gather + 0.06 * offer,
                    0.15 + 0.14 * offer,
                ),
                Some(Quat::from_rotation_x((-15.0 + 40.0 * offer).to_radians())),
                (&p, Vec3::new(-1.0, -0.6, -0.3)),
            );
            tools::reach(
                pose,
                true,
                Vec3::new(0.13, 0.13, 0.23),
                None,
                (&p, Vec3::new(1.0, -0.7, -0.3)),
            );
            tools::face(pose, m.aim, Vec3::ZERO);
            tools::plant(pose, &p, tools::stance(&p, 1.2, 0.04));
        }
        Clip::Hit => moves::hit(m, pose, 0.8),
        Clip::Death => moves::death(m, pose, &p, Fall::Sink),
        Clip::Spawn => moves::spawn(m, pose, &p, Entry::Rise),
        Clip::Evolve => moves::evolve(m, pose, &p, Power::Burst),
        Clip::Victory => {
            let sway = (m.t / 0.8 * std::f32::consts::TAU).sin();
            tools::set(pose, Joint::Chest, [2.0, 0.0, 5.0 * sway]);
            tools::set(pose, Joint::Head, [-6.0, 12.0 * sway, -3.0 * sway]);
            tools::reach(
                pose,
                true,
                Vec3::new(0.12, 0.13, 0.23),
                None,
                (&p, Vec3::new(1.0, -0.7, -0.3)),
            );
            tools::reach(
                pose,
                false,
                Vec3::new(-0.24, 0.23, 0.10),
                None,
                (&p, Vec3::new(-1.0, -0.5, -0.5)),
            );
            tools::plant(pose, &p, tools::stance(&p, 1.15, 0.0));
        }
        _ => {}
    }
    let hatch = tools::hold(m.gesture, [0.0, 0.06, 0.18, 0.42]);
    tools::add(pose, Joint::Chest, [12.0 * hatch, 0.0, 0.0]);
    tools::add(pose, Joint::UpperArmL, [-20.0 * hatch, 0.0, 0.0]);
    // The existing back nest swells as the authoritative timer matures.
    tools::shift(pose, Joint::Chest, [0.0, 0.015 * m.growth, 0.0]);
}
