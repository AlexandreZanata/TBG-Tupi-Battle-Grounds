//! Trident's two-handed short thrust, jaw breath, fin counterbalance and independent low rend sweep.
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
        tip: |_, _| Vec3::new(0.0, 0.0, 0.88),
        impact: |_, _| Impact::Bite,
    },
    evolve: Burst::Bloom,
    durations: moves::DURATIONS,
};
fn pose(m: &Moment, p: &mut Pose) {
    let body = model::plan(m.tier);
    match m.clip {
        Clip::Idle => {
            moves::idle(m, p, &body, 0.55);
            p[Secondary(Spring::HeadGear).index()].0.x = 3.0 + (m.clock * 1.5).sin() * 3.0;
        }
        Clip::Walk => {
            moves::walk(m, p, 4.0, 3.0);
            p[Secondary(Spring::Back).index()].0.z = (m.clock * 2.5).sin() * 5.0;
        }
        Clip::AttackMelee | Clip::AttackRanged | Clip::Cast => {
            let k = tools::hold(m.t, [0.15, 0.30, 0.40, 0.65]);
            tools::set(p, Chest, [8.0 * k, -12.0 * k, 0.0]);
        }
        Clip::Hit => {
            moves::hit(m, p, 0.8);
            tools::set(p, Head, [0.0, 15.0, 0.0]);
        }
        Clip::Death => {
            moves::death(m, p, &body, Fall::Kneel);
            p[Root.index()].0.z = 75.0 * tools::window(m.t, 0.3, 0.9);
        }
        Clip::Spawn => moves::spawn(m, p, &body, Entry::Rise),
        Clip::Evolve => {
            moves::evolve(m, p, &body, Power::Raise);
            p[Secondary(Spring::Back).index()].0.x = -15.0 * tools::hold(m.t, [0.0, 0.2, 0.6, 0.9]);
        }
        Clip::Victory => moves::victory(m, p, &body, Cheer::Raise),
    }
    if m.clip != Clip::Death {
        grip(m, p, &body);
    }
    tools::face(p, m.aim, Vec3::new(0.0, 1.0, 0.3));
}
fn grip(m: &Moment, p: &mut Pose, body: &crate::art::rig::Proportions) {
    let attack = matches!(m.clip, Clip::AttackMelee | Clip::AttackRanged | Clip::Cast);
    let load = if attack {
        tools::hold(m.t, [0.0, 0.14, 0.18, 0.30])
    } else {
        0.0
    };
    let hit = if attack {
        tools::hold(m.t, [0.18, 0.30, 0.40, 0.65])
    } else {
        0.0
    };
    let sweep = m
        .special
        .map_or(0.0, |t| tools::hold(t, [0.14, 0.30, 0.45, 0.75]));
    tools::add(p, Chest, [0.0, -55.0 * sweep, 0.0]);
    let z = -0.10 * load + 0.08 * hit;
    tools::reach(
        p,
        false,
        Vec3::new(-0.14, 0.13 - 0.08 * sweep, 0.12 + z),
        Some(Quat::IDENTITY),
        (body, Vec3::new(-1.0, -0.2, 0.0)),
    );
    tools::reach(
        p,
        true,
        Vec3::new(-0.04, 0.12 - 0.08 * sweep, 0.37 + z),
        Some(Quat::IDENTITY),
        (body, Vec3::new(1.0, -0.2, 0.0)),
    );
}
