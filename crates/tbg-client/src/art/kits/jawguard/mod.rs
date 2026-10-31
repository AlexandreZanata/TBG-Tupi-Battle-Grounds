//! Jawguard's clicking mandibles, low broad gait, braced shell and expiry-ready bite stance.
mod model;
use super::{
    common::{moves, moves::*},
    tools, Anatomy, Attack, Gait, Kit, Moment,
};
use crate::art::{
    anim::{Clip, Pose},
    rig::{
        Joint::*,
        Node::{Body, Prop, Secondary},
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
    gait: Gait {
        stride: 0.8,
        lift: 0.12,
        ..Gait::HEAVY
    },
    attack: Attack::Melee {
        from: Body(Head),
        tip: |_, _| Vec3::new(0.0, -0.05, 0.31),
        impact: |_, _| Impact::Bite,
    },
    evolve: Burst::Bloom,
    durations: moves::DURATIONS,
};
fn pose(m: &Moment, p: &mut Pose) {
    let body = model::plan(m.tier);
    let mut jaw = 3.0 + (m.clock * 1.8).sin() * 3.0;
    match m.clip {
        Clip::Idle => moves::idle(m, p, &body, 0.5),
        Clip::Walk => moves::walk(m, p, 3.0, 2.5),
        Clip::AttackMelee | Clip::AttackRanged | Clip::Cast => {
            let load = tools::hold(m.t, [0.0, 0.15, 0.19, 0.30]);
            let bite = tools::hold(m.t, [0.18, 0.30, 0.40, 0.65]);
            jaw += 28.0 * load - 3.0 * bite;
            for (s, upper, fore) in [(-1.0, UpperArmR, ForearmR), (1.0, UpperArmL, ForearmL)] {
                tools::set(p, upper, [-15.0 - 20.0 * bite, 0.0, s * 10.0]);
                tools::set(p, fore, [-20.0 * load - 10.0 * bite, 0.0, 0.0]);
            }
            tools::shift(p, Head, [0.0, 0.0, -0.07 * load + 0.04 * bite]);
        }
        Clip::Hit => {
            moves::hit(m, p, 0.4);
            tools::set(p, Chest, [0.0, 0.0, 6.0]);
        }
        Clip::Death => {
            moves::death(m, p, &body, Fall::Kneel);
            p[Secondary(Spring::Back).index()].0.x = 35.0 * tools::window(m.t, 0.0, 0.8);
            jaw = 0.0;
        }
        Clip::Spawn => moves::spawn(m, p, &body, Entry::Rise),
        Clip::Evolve => {
            moves::evolve(m, p, &body, Power::Raise);
            p[Secondary(Spring::Back).index()].0.x = -15.0 * tools::hold(m.t, [0.0, 0.3, 0.6, 0.9]);
        }
        Clip::Victory => moves::victory(m, p, &body, Cheer::Both),
    }
    if m.reduction > 0 {
        p[Secondary(Spring::Back).index()].1.y -= 0.035;
        tools::add(p, Chest, [5.0, 0.0, 0.0]);
    }
    if m.counter > 0 {
        jaw += 18.0;
    }
    for (s, n) in [(-1.0, Socket::WeaponR), (1.0, Socket::OffhandL)] {
        p[Prop(n).index()].0.y = s * jaw;
    }
    tools::face(p, m.aim, Vec3::new(0.0, 0.8, 0.3));
}
