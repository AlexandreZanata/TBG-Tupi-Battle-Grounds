//! Trigger's alternating muzzle/recoil, hat flourish, heel-toe gait and fourth-hit offhand cue.
mod model;
use super::{
    common::{moves, moves::*},
    tools, Anatomy, Attack, Gait, Kit, Moment,
};
use crate::art::{
    anim::{Clip, Pose},
    rig::{
        Joint::*,
        Node::{Prop, Secondary},
        Socket, Spring,
    },
};
use crate::vfx::{
    impacts::{Burst, Muzzle},
    shots::ShotKind,
};
use bevy::prelude::*;
static ANATOMY: Anatomy = Anatomy {
    rest: model::rest,
    parent: model::parent,
    springs: false,
    planted_legs: true,
};
pub const KIT: Kit = Kit {
    anatomy: Some(&ANATOMY),
    figure: model::spec,
    pose,
    gait: Gait::LIGHT,
    attack: Attack::TwinShot {
        hands: [Prop(Socket::WeaponR), Prop(Socket::OffhandL)],
        muzzle: model::MUZZLE,
        shot: ShotKind::Bullet,
        flash: Muzzle::Pistol,
    },
    evolve: Burst::Valor,
    durations: moves::DURATIONS,
};
fn pose(m: &Moment, p: &mut Pose) {
    let body = model::plan(m.tier);
    match m.clip {
        Clip::Idle => {
            moves::idle(m, p, &body, 0.6);
            let check = tools::hold((m.clock * 0.25).rem_euclid(1.0), [0.10, 0.26, 0.38, 0.62]);
            p[Prop(Socket::WeaponR).index()].0.z = 22.0 * check;
            tools::add(p, Head, [6.0 * check, -10.0 * check, 0.0]);
        }
        Clip::Walk => {
            moves::walk(m, p, 6.0, 4.0);
            tools::add(p, FootL, [8.0, 0.0, 0.0]);
            tools::add(p, FootR, [8.0, 0.0, 0.0]);
            let step = (m.walk.unwrap_or(0.0) * std::f32::consts::TAU).sin();
            tools::add(p, Chest, [0.0, 3.0 * step, -2.0 * step]);
            tools::add(p, Head, [-3.0, -4.0 * step, 0.0]);
        }
        Clip::AttackRanged | Clip::AttackMelee | Clip::Cast => tools::set(
            p,
            Chest,
            [0.0, if m.variant == 0 { 12.0 } else { -12.0 }, 0.0],
        ),
        Clip::Hit => {
            moves::hit(m, p, 1.0);
            p[Secondary(Spring::HeadGear).index()].0.x = 15.0;
        }
        Clip::Death => {
            let k = tools::window(m.t, 0.0, 0.7);
            tools::set(p, Root, [0.0, 0.0, 0.0]);
            tools::shift(p, Hips, [0.0, -body.leg * 1.1 * k, -0.08 * k]);
            tools::set(p, Chest, [15.0 * k, 0.0, 0.0]);
            tools::set(p, Head, [20.0 * k, 0.0, 0.0]);
            for (thigh, shin, arm, s) in [
                (ThighL, ShinL, UpperArmL, 1.0),
                (ThighR, ShinR, UpperArmR, -1.0),
            ] {
                tools::set(p, thigh, [-80.0 * k, 0.0, s * 10.0 * k]);
                tools::set(p, shin, [45.0 * k, 0.0, 0.0]);
                tools::set(p, arm, [20.0 * k, 0.0, s * 12.0 * k]);
            }
        }
        Clip::Spawn => moves::spawn(m, p, &body, Entry::Rise),
        Clip::Evolve => {
            moves::evolve(m, p, &body, Power::Raise);
            p[Secondary(Spring::HeadGear).index()].0.x =
                30.0 * tools::hold(m.t, [0.0, 0.2, 0.5, 0.9]);
        }
        Clip::Victory => {
            p.fill((Vec3::ZERO, Vec3::ZERO));
            let t = m.t.rem_euclid(0.8);
            let flourish = tools::hold(t, [0.0, 0.14, 0.52, 0.78]);
            tools::set(p, Head, [-4.0 * flourish, 7.0 * flourish, 0.0]);
            p[Secondary(Spring::HeadGear).index()].0.z = -5.0 * flourish;
            p[Prop(Socket::WeaponR).index()].0.x = 360.0 * tools::window(t, 0.16, 0.48);
            p[Prop(Socket::OffhandL).index()].0.x = -360.0 * tools::window(t, 0.23, 0.55);
            tools::plant(p, &body, tools::stance(&body, 1.25, 0.04));
        }
    }
    if m.clip != Clip::Death {
        arms(m, p, &body);
    }
    p[Secondary(Spring::Back).index()].0.y = (m.clock * 2.0).sin() * 18.0;
    p[Prop(Socket::Back).index()].0.x = (m.clock * 1.6).sin() * 12.0;
    tools::face(p, m.aim, Vec3::new(0.0, 0.7, 0.3));
}
fn arms(m: &Moment, p: &mut Pose, body: &crate::art::rig::Proportions) {
    let firing = matches!(m.clip, Clip::AttackRanged | Clip::AttackMelee | Clip::Cast);
    let cross = m
        .special
        .map_or(0.0, |t| tools::hold(t, [0.25, 0.30, 0.40, 0.62]));
    for left in [false, true] {
        let primary = usize::from(m.variant & 1) == usize::from(left);
        let recoil = if firing && primary {
            tools::hold(m.t, [0.15, 0.18, 0.23, 0.45])
        } else {
            0.0
        };
        let check = if m.clip == Clip::Idle && !left {
            tools::hold((m.clock * 0.25).rem_euclid(1.0), [0.10, 0.26, 0.38, 0.62])
        } else {
            0.0
        };
        let victory = if m.clip == Clip::Victory {
            tools::hold(m.t.rem_euclid(0.8), [0.0, 0.14, 0.52, 0.78])
        } else {
            0.0
        };
        let draw = tools::hold(m.t, [0.0, 0.12, 0.25, 0.60]);
        let raised = if firing && primary {
            draw
        } else {
            cross.max(check)
        };
        tools::reach(
            p,
            left,
            Vec3::new(
                if left {
                    0.18 + 0.08 * victory
                } else {
                    -0.18 - 0.08 * victory
                },
                0.02 + 0.13 * raised + 0.12 * victory,
                0.12 + 0.15 * raised - 0.055 * recoil - 0.035 * cross,
            ),
            Some(Quat::IDENTITY),
            (body, Vec3::new(if left { 1.0 } else { -1.0 }, -0.2, 0.0)),
        );
    }
}
