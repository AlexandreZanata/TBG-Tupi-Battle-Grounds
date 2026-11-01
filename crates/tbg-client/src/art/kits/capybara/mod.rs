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
            moves::idle(&Moment { aim: None, ..*m }, p, &body, 0.7);
            tools::set(p, Head, [-4.0, 0.0, (m.clock * 0.6).sin() * 4.0]);
            let sniff = tools::hold((m.clock * 0.28).fract(), [0.08, 0.16, 0.22, 0.38]);
            tools::shift(p, Head, [0.0, 0.0, 0.008 * sniff]);
            tools::set(p, UpperArmL, [-8.0, 0.0, 14.0]);
            tools::set(p, UpperArmR, [-8.0, 0.0, -14.0]);
        }
        Clip::Walk => {
            moves::walk(m, p, 5.0, 6.0);
            let step = (m.walk.unwrap_or(0.0) * std::f32::consts::TAU).sin();
            tools::set(p, Hips, [0.0, 0.0, step * 5.0]);
            tools::set(p, UpperArmL, [-10.0 + 7.0 * step, 0.0, 18.0]);
            tools::set(p, UpperArmR, [-10.0 - 7.0 * step, 0.0, -18.0]);
            tools::set(p, ForearmL, [-15.0, 0.0, 0.0]);
            tools::set(p, ForearmR, [-15.0, 0.0, 0.0]);
        }
        Clip::AttackMelee | Clip::AttackRanged | Clip::Cast => {
            let gather = tools::hold(m.t, [0.0, 0.17, 0.23, 0.30]);
            let bump = tools::hold(m.t, [0.23, 0.30, 0.38, 0.65]);
            tools::set(p, Chest, [-3.0 * gather + 10.0 * bump, 0.0, 0.0]);
            tools::reach(
                p,
                false,
                Vec3::new(-0.27 + 0.03 * bump, 0.18 + 0.06 * bump, 0.23 + 0.13 * bump),
                Some(Quat::IDENTITY),
                (&body, Vec3::new(-1.0, -0.7, -0.3)),
            );
            tools::reach(
                p,
                true,
                Vec3::new(0.31, 0.12, 0.28),
                None,
                (&body, Vec3::new(1.0, -0.7, -0.3)),
            );
            tools::plant(p, &body, tools::stance(&body, 1.3, 0.04));
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
        Clip::Victory => {
            let settle = (m.t / 0.8 * std::f32::consts::TAU).sin();
            p[Root.index()] = (Vec3::ZERO, Vec3::ZERO);
            tools::set(p, Chest, [2.0, 0.0, 4.0 * settle]);
            tools::set(p, Head, [-5.0, 5.0 * settle, -3.0 * settle]);
            tools::reach(
                p,
                true,
                Vec3::new(0.31, 0.12 + 0.02 * settle, 0.28),
                None,
                (&body, Vec3::new(1.0, -0.7, -0.3)),
            );
            tools::set(p, UpperArmR, [-12.0, 0.0, -18.0]);
            tools::set(p, ForearmR, [-18.0, 0.0, 0.0]);
            tools::plant(p, &body, tools::stance(&body, 1.25, 0.0));
        }
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
