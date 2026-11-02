//! Back draw, overhead anticipation, earthward impact and a restrained monumental celebration.
use super::model;
use crate::art::{
    anim::{Clip, Pose},
    kits::{
        common::moves::{self, Entry, Fall, Power},
        tools, Moment,
    },
    rig::{Joint::*, Node, Socket},
};
use bevy::prelude::*;
pub(super) fn pose(m: &Moment, p: &mut Pose) {
    let plan = model::plan(m.tier);
    let mut at = Vec3::ZERO;
    let mut pitch = -8.0;
    match m.clip {
        Clip::Idle => moves::idle(m, p, &plan, 0.85),
        Clip::Walk => moves::walk(m, p, 5.0, 2.0),
        Clip::AttackMelee | Clip::AttackRanged | Clip::Cast => {
            let draw = tools::hold(m.t, [0.0, 0.14, 0.18, 0.30]);
            let strike = tools::hold(m.t, [0.18, 0.30, 0.48, 0.85]);
            pitch = -8.0 + 8.0 * draw + 102.0 * strike;
            at = Vec3::new(
                -0.23 * (draw + strike).min(1.0),
                0.34 * draw - 0.08 * strike,
                0.42 * draw + 0.66 * strike,
            );
            tools::set(p, Chest, [-8.0 * draw + 18.0 * strike, 0.0, 0.0]);
            tools::shift(p, Hips, [0.0, -0.07 * strike, 0.02 * strike]);
            let grip = Vec3::new(
                -0.22,
                0.22 + 0.25 * draw - 0.20 * strike,
                0.24 + 0.25 * strike,
            );
            tools::reach(
                p,
                false,
                grip,
                Some(Quat::IDENTITY),
                (&plan, Vec3::new(-0.8, -0.4, 0.4)),
            );
            tools::reach(
                p,
                true,
                grip + Vec3::new(0.22, -0.08, 0.02),
                Some(Quat::IDENTITY),
                (&plan, Vec3::new(0.8, -0.4, 0.4)),
            );
            tools::plant(p, &plan, tools::stance(&plan, 1.12, 0.03));
        }
        Clip::Hit => moves::hit(m, p, 0.25),
        Clip::Spawn => moves::spawn(m, p, &plan, Entry::Rise),
        Clip::Evolve => moves::evolve(m, p, &plan, Power::Raise),
        Clip::Death => moves::death(m, p, &plan, Fall::Kneel),
        Clip::Victory => {
            let beat = (m.clock * 2.0).sin();
            tools::add(p, Chest, [0.0, 6.0 * beat, 0.0]);
            tools::add(p, Head, [-6.0, 0.0, 0.0]);
            tools::add(p, UpperArmL, [-70.0, 0.0, 20.0]);
            tools::add(p, ForearmL, [-35.0, 0.0, 0.0]);
            tools::shift(p, Hips, [0.0, -0.012 * beat.abs(), 0.0]);
        }
    }
    let weapon = &mut p[Node::Prop(Socket::WeaponR).index()];
    weapon.0 = Vec3::new(pitch, 0.0, 0.0);
    weapon.1 = at;
}
