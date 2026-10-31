//! Smuggler: a one-handed revolver, a shoulder crate and a torso-only rear placement gesture.

mod model;

use bevy::prelude::*;

use super::common::{moves, moves::*, Form};
use super::{tools, Attack, Gait, Kit, Moment};
use crate::art::{
    anim::{Clip, Pose},
    rig::{Joint, Node, Socket},
};
use crate::vfx::{
    impacts::{Burst, Muzzle},
    shots::ShotKind,
};
use model::plan;

pub const KIT: Kit = Kit {
    figure: model::spec,
    pose,
    gait: Gait::LIGHT,
    attack: Attack::Shot {
        from: Node::Prop(Socket::WeaponR),
        muzzle: |_, _| Vec3::new(0.0, 0.04, 0.30),
        shot: |_, _| ShotKind::Bullet,
        flash: |_, _| Some((Muzzle::Bolt, None)),
    },
    evolve: Burst::Valor,
    durations: moves::DURATIONS,
};

pub fn pose(m: &Moment, pose: &mut Pose) {
    let f = Form::of(m.tier, m.branch);
    let p = plan(m.tier);
    match m.clip {
        Clip::Idle => {
            moves::idle(m, pose, &p, 0.1);
            // Darting glances; the pistol rests near the holster.
            let glance = (m.clock * 0.9).sin().signum() * tools::wave(m.clock, 0.5, 3.0).abs();
            tools::add(pose, Joint::Head, [0.0, 35.0 * glance, 0.0]);
            tools::set(pose, Joint::UpperArmR, [5.0, 0.0, -10.0]);
            tools::set(pose, Joint::ForearmR, [-15.0, 0.0, 0.0]);
        }
        Clip::Walk => {
            // A sneaking lean; the runner branch leans harder.
            let lean = if f.b { 14.0 } else { 9.0 };
            moves::walk(m, pose, lean, 4.0);
        }
        Clip::Cast => {
            let recoil = tools::hold(m.t, [0.20, 0.22, 0.25, 0.32]);
            tools::reach(
                pose,
                false,
                Vec3::new(-0.18, 0.20, 0.42 - recoil * 0.06),
                Some(Quat::from_rotation_x((-8.0 * recoil).to_radians())),
                (&p, Vec3::new(-1.0, -0.4, -0.4)),
            );
            tools::face(pose, m.aim, Vec3::ZERO);
        }
        Clip::Hit => moves::hit(m, pose, 1.0),
        Clip::Death => moves::death(m, pose, &p, Fall::Forward),
        Clip::Spawn => moves::spawn(m, pose, &p, Entry::Pop),
        Clip::Evolve => moves::evolve(m, pose, &p, Power::Spin),
        Clip::Victory => moves::victory(m, pose, &p, Cheer::Dance),
        _ => {}
    }
    // Carry and lower with the free arm: the leg walk and pistol arm remain independent.
    let drop = tools::hold(m.gesture, [0.0, 0.14, 0.35, 0.6]);
    tools::reach(
        pose,
        true,
        Vec3::new(0.20, 0.30 - 0.50 * drop, -0.12 - 0.18 * drop),
        None,
        (&p, Vec3::new(1.0, -0.6, -0.6)),
    );
    tools::add(pose, Joint::Chest, [10.0 * drop, -12.0 * drop, 0.0]);
}
