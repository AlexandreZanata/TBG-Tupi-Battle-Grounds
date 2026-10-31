//! Capybara's belly breath, rolling short steps, palm bump, local flop and sideways settling.
mod model;
use super::{
    common::{moves, moves::*},
    tools, Anatomy, Attack, Gait, Kit, Moment,
};
use crate::art::{
    anim::{Clip, Pose},
    rig::{
        Joint::*,
        Node::{Body, Secondary},
        Spring,
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
        stride: 0.75,
        lift: 0.12,
        ..Gait::HEAVY
    },
    attack: Attack::Melee {
        from: Body(HandR),
        tip: |_, _| Vec3::new(0.0, 0.0, 0.12),
        impact: |_, _| Impact::Crush,
    },
    evolve: Burst::Bloom,
    durations: moves::DURATIONS,
};
fn pose(m: &Moment, p: &mut Pose) {
    let body = model::plan(m.tier);
    match m.clip {
        Clip::Idle => {
            moves::idle(m, p, &body, 0.7);
            tools::set(p, Head, [-4.0, 0.0, (m.clock * 0.6).sin() * 4.0]);
        }
        Clip::Walk => {
            moves::walk(m, p, 5.0, 6.0);
            tools::set(p, Hips, [0.0, 0.0, (m.clock * 3.0).sin() * 5.0]);
        }
        Clip::AttackMelee | Clip::AttackRanged | Clip::Cast => {
            let k = tools::hold(m.t, [0.12, 0.30, 0.38, 0.65]);
            tools::set(p, Chest, [16.0 * k, 0.0, 0.0]);
            tools::set(p, UpperArmR, [-65.0 * k, 0.0, -20.0]);
            tools::set(p, UpperArmL, [-35.0 * k, 0.0, 20.0]);
        }
        Clip::Hit => {
            moves::hit(m, p, 0.6);
            p[Secondary(Spring::Tabard).index()].1.z -= 0.025;
        }
        Clip::Death => {
            let k = tools::window(m.t, 0.05, 0.75);
            tools::set(p, Root, [0.0, 0.0, 82.0 * k]);
            tools::shift(p, Root, [0.0, 0.03 * k, 0.0]);
            tools::set(p, Head, [12.0 * k, 0.0, 0.0]);
        }
        Clip::Spawn => moves::spawn(m, p, &body, Entry::Drop),
        Clip::Evolve => {
            moves::evolve(m, p, &body, Power::Burst);
            tools::set(p, UpperArmR, [-45.0, 0.0, 30.0 * (m.t * 9.0).sin()]);
        }
        Clip::Victory => moves::victory(m, p, &body, Cheer::Both),
    }
    if m.clip != Clip::Death {
        p[Secondary(Spring::Tabard).index()].1.z += (m.clock * 1.8).sin() * 0.012;
    }
    if let Some(t) = m.special {
        let tuck = tools::hold(t, [0.0, 0.12, 0.20, 0.30]);
        let flop = tools::hold(t, [0.14, 0.30, 0.40, 0.70]);
        tools::shift(p, Hips, [0.0, -0.05 * tuck + 0.04 * flop, 0.0]);
        tools::add(p, Chest, [35.0 * flop, 0.0, 0.0]);
        tools::add(p, UpperArmL, [-35.0 * tuck, 0.0, 15.0 * flop]);
        tools::add(p, UpperArmR, [-35.0 * tuck, 0.0, -15.0 * flop]);
    }
    if m.reduction > 0 {
        tools::add(p, Chest, [-6.0, 0.0, 0.0]);
    }
    tools::face(p, m.aim, Vec3::new(0.0, 0.8, 0.3));
}
