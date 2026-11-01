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
        Clip::Idle => moves::idle(m, p, &body, 0.35),
        Clip::Walk => moves::walk(m, p, 3.0, 1.5),
        Clip::AttackMelee | Clip::AttackRanged | Clip::Cast => {
            let k = tools::hold(m.t, [0.15, 0.30, 0.42, 0.65]);
            tools::set(p, Chest, [4.0 * k, -10.0 * k, 0.0]);
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
        Clip::Victory => moves::victory(m, p, &body, Cheer::Raise),
    }
    if m.clip != Clip::Death {
        let attack = matches!(m.clip, Clip::AttackMelee | Clip::AttackRanged | Clip::Cast);
        let thrust = if attack {
            tools::hold(m.t, [0.15, 0.30, 0.42, 0.65])
        } else {
            0.0
        };
        let open = m
            .special
            .map_or(0.0, |t| tools::hold(t, [0.05, 0.30, 0.50, 0.80]));
        tools::reach(
            p,
            false,
            Vec3::new(-0.18, 0.05, 0.08 + 0.12 * thrust),
            Some(Quat::IDENTITY),
            (&body, Vec3::new(-1.0, -0.3, 0.0)),
        );
        tools::reach(
            p,
            true,
            Vec3::new(0.19 + 0.04 * open, 0.12 + 0.10 * open, 0.17 + 0.12 * open),
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
