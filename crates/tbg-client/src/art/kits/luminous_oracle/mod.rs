//! Oracle's small planted robe steps, staff alignment, closing offhand and lowering hood/cape fall.
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
use crate::vfx::{impacts::Burst, shots::ShotKind};
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
    gait: Gait {
        stride: 0.8,
        lift: 0.15,
        ..Gait::LIGHT
    },
    attack: Attack::Shot {
        from: Prop(Socket::WeaponR),
        muzzle: model::muzzle,
        shot: |_, _| ShotKind::SunSpark,
        flash: |_, _| None,
        loaded: None,
    },
    evolve: Burst::Valor,
    durations: moves::DURATIONS,
};
fn pose(m: &Moment, p: &mut Pose) {
    let body = model::plan(m.tier);
    match m.clip {
        Clip::Idle => {
            let listen = (m.clock * 1.2).sin();
            tools::set(p, Chest, [1.0 + listen, 0.0, -1.0]);
            tools::set(p, Head, [7.0, -4.0 + 3.0 * listen, 2.0]);
            tools::plant(p, &body, tools::stance(&body, 1.05, 0.01));
        }
        Clip::Walk => {
            moves::walk(m, p, 2.0, 1.0);
            let step = (m.walk.unwrap_or(0.0) * std::f32::consts::TAU).sin();
            tools::set(p, Chest, [1.0, -2.0 * step, step]);
            tools::set(p, Head, [5.0, 2.0 * step, 0.0]);
        }
        Clip::Cast | Clip::AttackRanged | Clip::AttackMelee => {
            let k = tools::hold(m.t, [0.1, 0.22, 0.30, 0.60]);
            let gather = tools::hold(m.t, [0.0, 0.13, 0.16, 0.22]);
            tools::set(p, Chest, [-4.0 * gather + 3.0 * k, -5.0 * gather, 0.0]);
            tools::set(p, Head, [6.0 * gather - 3.0 * k, 0.0, 0.0]);
        }
        Clip::Hit => {
            moves::hit(m, p, 0.7);
            tools::set(p, Chest, [0.0, 12.0, 0.0]);
        }
        Clip::Death => {
            moves::death(m, p, &body, Fall::Kneel);
            p[Prop(Socket::WeaponR).index()].0.x = 35.0 * tools::window(m.t, 0.2, 0.8);
        }
        Clip::Spawn => moves::spawn(m, p, &body, Entry::Rise),
        Clip::Evolve => moves::evolve(m, p, &body, Power::Raise),
        Clip::Victory => {
            let orbit = (m.t / 0.8 * std::f32::consts::TAU).sin();
            p[Root.index()] = (Vec3::ZERO, Vec3::ZERO);
            tools::set(p, Chest, [-2.0, orbit * 5.0, 0.0]);
            tools::set(p, Head, [5.0, -orbit * 8.0, 0.0]);
            tools::plant(p, &body, tools::stance(&body, 1.05, 0.01));
        }
    }
    if m.clip != Clip::Death {
        let cast = matches!(m.clip, Clip::Cast | Clip::AttackRanged | Clip::AttackMelee);
        let gather = if cast {
            tools::hold(m.t, [0.0, 0.13, 0.16, 0.22])
        } else {
            0.0
        };
        let present = if cast {
            tools::hold(m.t, [0.16, 0.22, 0.30, 0.60])
        } else {
            0.0
        };
        let orbit = if m.clip == Clip::Victory {
            (m.t / 0.8 * std::f32::consts::TAU).sin()
        } else {
            0.0
        };
        let open = if m.clip == Clip::Victory {
            0.5 - 0.5 * (m.t / 0.8 * std::f32::consts::TAU).cos()
        } else {
            0.0
        };
        let close = m
            .special
            .map_or(0.0, |t| tools::hold(t, [0.1, 0.30, 0.40, 0.70]));
        tools::reach(
            p,
            false,
            Vec3::new(
                -0.14 - 0.025 * orbit,
                0.10 + 0.07 * gather + 0.08 * open,
                0.15 - 0.035 * gather + 0.055 * present,
            ),
            Some(Quat::from_rotation_z(orbit * 0.10)),
            (&body, Vec3::new(-1.0, -0.2, 0.0)),
        );
        // The higher offhand can close during a cast without exceeding the short arms.
        tools::reach(
            p,
            true,
            Vec3::new(
                0.08 - 0.10 * close + 0.13 * open,
                0.20 + 0.035 * gather,
                0.25 - 0.07 * close + 0.02 * present,
            ),
            Some(Quat::IDENTITY),
            (&body, Vec3::new(1.0, -0.2, 0.0)),
        );
        p[Prop(Socket::OffhandL).index()].0.x = -30.0 * close;
    }
    p[Secondary(Spring::HeadGear).index()].0.y = (m.clock * 0.9).sin() * 12.0;
    p[Secondary(Spring::Back).index()].0.x = (m.clock * 1.0).sin() * 2.0;
    tools::face(p, m.aim, Vec3::new(0.0, 0.9, 0.3));
}
