//! Farwatch kit: a watchman who kneels to aim his arbalest across the
//! lane, the bolt leaving along its own flight; recoil, then he rises and
//! cranks the next bolt.

mod model;

use bevy::prelude::*;

use super::common::{moves, moves::*};
use super::{tools, Attack, Gait, Kit, Moment};
use crate::art::{
    anim::{Clip, Pose},
    rig::{Joint, Node, Proportions, Socket},
};
use crate::vfx::{
    impacts::{Burst, Muzzle},
    shots::{self, ShotKind},
};
use model::{plan, FORE, MUZZLE};

pub const KIT: Kit = Kit {
    anatomy: None,
    figure: model::spec,
    pose,
    gait: Gait::LIGHT,
    attack: Attack::Shot {
        loaded: None,
        from: Node::Prop(Socket::WeaponR),
        muzzle: |_, _| MUZZLE,
        shot: |_, _| ShotKind::Arrow,
        flash: |_, _| Some((Muzzle::Bolt, None)),
    },
    evolve: Burst::Valor,
    durations: moves::DURATIONS,
};

pub fn pose(m: &Moment, pose: &mut Pose) {
    let p = plan(m.tier);
    match m.clip {
        Clip::Idle => {
            moves::idle(m, pose, &p, 0.2);
            let ready = m.aim.map_or(0.0, |a| a.weight);
            let crank = tools::hold(m.since_attack, [0.6, 0.75, 1.6, 1.8]) * ready;
            let sight = tools::hold((m.clock * 0.2).rem_euclid(1.0), [0.2, 0.35, 0.5, 0.72])
                * (1.0 - ready);
            tools::add(pose, Joint::Head, [7.0 * sight, -6.0 * sight, 0.0]);
            let turn = Quat::from_rotation_x((25.0 - 35.0 * ready + 40.0 * crank).to_radians());
            moves::shoulder_arms(
                pose,
                &p,
                Vec3::new(0.02, 0.23 + 0.02 * ready, 0.23),
                turn,
                FORE,
            );
        }
        Clip::Walk => {
            moves::walk(m, pose, 5.0, 2.0);
            let step = (m.walk.unwrap_or(0.0) * std::f32::consts::TAU).sin();
            let turn = Quat::from_rotation_x((24.0 + 2.0 * step).to_radians());
            moves::shoulder_arms(pose, &p, Vec3::new(0.02, 0.23, 0.23), turn, FORE);
            tools::add(pose, Joint::Head, [2.0, -4.0 * step, 0.0]);
        }
        Clip::AttackRanged => aim(m, pose, &p),
        Clip::Hit => {
            moves::hit(m, pose, 1.0);
            low(pose, &p);
        }
        Clip::Death => {
            moves::death(m, pose, &p, Fall::Back);
            low(pose, &p);
        }
        Clip::Spawn => {
            moves::spawn(m, pose, &p, Entry::March);
            low(pose, &p);
        }
        Clip::Evolve => {
            moves::evolve(m, pose, &p, Power::Raise);
            let up = Quat::from_rotation_x(-80f32.to_radians());
            tools::reach(
                pose,
                false,
                Vec3::new(-0.15, 0.45, 0.1),
                Some(up),
                (&p, Vec3::new(-1.0, 0.0, -0.3)),
            );
        }
        Clip::Victory => {
            pose.fill((Vec3::ZERO, Vec3::ZERO));
            let nod = tools::hold(m.t.rem_euclid(0.8), [0.08, 0.25, 0.42, 0.74]);
            tools::set(pose, Joint::Head, [10.0 * nod, 0.0, 0.0]);
            tools::set(pose, Joint::Spine, [4.0 * nod, 0.0, 0.0]);
            low(pose, &p);
            tools::plant(pose, &p, tools::stance(&p, 1.2, 0.05));
        }
        _ => low(pose, &p),
    }
}

/// The arbalest held low across the body.
fn low(pose: &mut Pose, p: &Proportions) {
    let turn = Quat::from_rotation_x(25f32.to_radians());
    moves::shoulder_arms(pose, p, Vec3::new(0.02, 0.23, 0.23), turn, FORE);
}

/// Kneel and shoulder the arbalest (0–0.12 s), Release at 0.18 s with a
/// recoil, hold, rise by 0.62 s.
fn aim(m: &Moment, pose: &mut Pose, p: &Proportions) {
    let k = (m.t / ATTACK).clamp(0.0, 1.0);
    let kneel = tools::hold(k, [0.0, 0.18, 0.7, 1.0]);
    let kick = tools::snap(m.t, 0.18, 0.21) * (1.0 - tools::window(m.t, 0.22, 0.45));
    let up = moves::aim_level(m, pose, 0.9, shots::arc(ShotKind::Arrow));
    tools::shift(pose, Joint::Hips, [0.0, -p.leg * 0.32 * kneel, 0.0]);
    tools::add(pose, Joint::Chest, [-8.0 * kick, 0.0, 0.0]);
    tools::add(
        pose,
        Joint::Head,
        [10.0 * kneel, -4.0 * kneel * (1.0 - kick), 0.0],
    );
    tools::add(pose, Joint::Spine, [2.0 * kneel, 0.0, 0.0]);
    let turn = Quat::from_rotation_x(-up - 12f32.to_radians() * kick);
    let at = Vec3::new(0.02, 0.25, 0.23) - turn * Vec3::Z * 0.05 * kick;
    moves::shoulder_arms(pose, p, at, turn, FORE);
    tools::plant(pose, p, tools::stance(p, 1.3, 0.18 * kneel));
}
