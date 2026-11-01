//! Solar Sentinel's shield-led gait, controlled spear thrust and independent sun-opening grant pose.
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
use crate::vfx::impacts::{Burst, Impact};
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
    gait: Gait::HEAVY,
    attack: Attack::Melee {
        from: Prop(Socket::WeaponR),
        tip: |_, _| Vec3::new(0.0, 0.0, 0.73),
        impact: |_, _| Impact::Sparks,
    },
    evolve: Burst::Valor,
    durations: moves::DURATIONS,
};
fn pose(m: &Moment, p: &mut Pose) {
    let body = model::plan(m.tier);
    match m.clip {
        Clip::Idle => {
            let watch = tools::wave(m.clock, 0.65, 0.4);
            tools::set(p, Chest, [2.0, -3.0, (m.clock * 1.3).sin()]);
            tools::set(p, Head, [-3.0, watch * 8.0, 0.0]);
            tools::plant(p, &body, tools::stance(&body, 1.2, 0.04));
        }
        Clip::Walk => {
            moves::walk(m, p, 3.0, 1.5);
            let step = (m.walk.unwrap_or(0.0) * std::f32::consts::TAU).sin();
            tools::add(p, Chest, [0.0, -3.0 * step, 0.0]);
            tools::set(p, Head, [-2.0, 2.0 * step, 0.0]);
        }
        Clip::AttackMelee | Clip::AttackRanged | Clip::Cast => {
            let k = tools::hold(m.t, [0.15, 0.30, 0.42, 0.65]);
            let brace = tools::hold(m.t, [0.0, 0.14, 0.16, 0.30]);
            tools::set(
                p,
                Chest,
                [-4.0 * brace + 4.0 * k, 7.0 * brace - 10.0 * k, 0.0],
            );
            tools::set(p, Head, [3.0 * brace - 2.0 * k, 0.0, 0.0]);
        }
        Clip::Hit => {
            moves::hit(m, p, 0.7);
            tools::set(p, UpperArmL, [-85.0, 0.0, 15.0]);
        }
        Clip::Death => {
            moves::death(m, p, &body, Fall::Kneel);
            tools::set(p, UpperArmL, [-65.0, 0.0, 10.0]);
        }
        Clip::Spawn => moves::spawn(m, p, &body, Entry::Rise),
        Clip::Evolve => moves::evolve(m, p, &body, Power::Raise),
        Clip::Victory => {
            let salute = 0.5 - 0.5 * (m.t / 0.8 * std::f32::consts::TAU).cos();
            p[Root.index()] = (Vec3::ZERO, Vec3::ZERO);
            tools::set(p, Chest, [-5.0 * salute, -4.0 * salute, 0.0]);
            tools::set(p, Head, [-7.0 * salute, 0.0, 0.0]);
            tools::plant(p, &body, tools::stance(&body, 1.25, 0.04));
        }
    }
    if m.clip != Clip::Death {
        let attack = matches!(m.clip, Clip::AttackMelee | Clip::AttackRanged | Clip::Cast);
        let thrust = if attack {
            tools::hold(m.t, [0.15, 0.30, 0.42, 0.65])
        } else {
            0.0
        };
        let load = if attack {
            tools::hold(m.t, [0.0, 0.14, 0.16, 0.30])
        } else {
            0.0
        };
        let salute = if m.clip == Clip::Victory {
            0.5 - 0.5 * (m.t / 0.8 * std::f32::consts::TAU).cos()
        } else {
            0.0
        };
        let carry = m.walk.map_or(0.0, |v| (v * std::f32::consts::TAU).sin());
        let open = m
            .special
            .map_or(0.0, |t| tools::hold(t, [0.05, 0.30, 0.50, 0.80]));
        // Lifting the thrust wrist keeps its full envelope inside the 0.323 arm span.
        tools::reach(
            p,
            false,
            Vec3::new(
                -0.18,
                0.05 + 0.05 * thrust + 0.16 * salute,
                0.08 - 0.06 * load + 0.14 * thrust + 0.012 * carry,
            ),
            Some(Quat::from_rotation_x(-0.65 * salute)),
            (&body, Vec3::new(-1.0, -0.3, 0.0)),
        );
        tools::reach(
            p,
            true,
            Vec3::new(
                0.19 + 0.04 * open + 0.04 * salute,
                0.12 + 0.10 * open + 0.10 * salute,
                0.17 + 0.12 * open - 0.012 * carry,
            ),
            Some(Quat::IDENTITY),
            (&body, Vec3::new(1.0, -0.3, 0.0)),
        );
        for (s, n) in [(-1.0, Socket::StringTop), (1.0, Socket::StringBottom)] {
            p[Prop(n).index()].0.y = s * 45.0 * open;
        }
    }
    p[Secondary(Spring::HeadGear).index()].0.y = (m.clock * 0.7).sin() * 5.0;
    tools::face(p, m.aim, Vec3::new(0.0, 1.0, 0.3));
}
