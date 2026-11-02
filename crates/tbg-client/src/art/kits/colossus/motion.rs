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
        Clip::Idle => {
            moves::idle(m, p, &plan, 0.85);
            // GD-04: a slow monumental settle of the shoulders between breaths.
            let settle = tools::wave(m.clock, 0.4, 0.7);
            tools::add(p, UpperArmL, [2.0 * settle, 0.0, 3.0 + 1.5 * settle]);
            tools::add(p, UpperArmR, [-2.0 * settle, 0.0, -3.0 - 1.5 * settle]);
        }
        Clip::Walk => {
            moves::walk(m, p, 5.0, 2.0);
            // GD-04: mass shifts over the planted foot; the heavy arms swing against it.
            let phase = m.walk.unwrap_or(0.0) * std::f32::consts::TAU;
            tools::shift(
                p,
                Hips,
                [0.025 * phase.sin(), -0.01 * phase.cos().abs(), 0.0],
            );
            tools::add(p, Chest, [0.0, 0.0, -2.5 * phase.sin()]);
            tools::add(p, UpperArmL, [12.0 * phase.sin(), 0.0, 6.0]);
            tools::add(p, UpperArmR, [-12.0 * phase.sin(), 0.0, -6.0]);
            tools::add(p, ForearmL, [-12.0, 0.0, 0.0]);
            tools::add(p, ForearmR, [-12.0, 0.0, 0.0]);
        }
        Clip::AttackMelee | Clip::AttackRanged | Clip::Cast => {
            let draw = tools::hold(m.t, [0.0, 0.14, 0.18, 0.30]);
            let strike = tools::hold(m.t, [0.18, 0.30, 0.48, 0.85]);
            tools::set(p, Chest, [-8.0 * draw + 18.0 * strike, 0.0, 0.0]);
            tools::shift(p, Hips, [0.0, -0.07 * strike, 0.02 * strike]);
            let chest = tools::chest(p, &plan);
            let default_target = Vec3::new(0.0, 0.15, 1.35);
            let target_root = m.aim.map_or(default_target, |a| {
                let aimed = Vec3::new(0.0, a.target.y.clamp(0.05, 0.45), a.target.xz().length());
                default_target.lerp(aimed, a.weight)
            });
            let target_chest = chest.inverse().transform_point3(target_root);
            let anchor = Vec3::new(0.0, 0.22, 0.18);
            let to_target = Vec3::new(
                0.0,
                target_chest.y - anchor.y,
                (target_chest.z - anchor.z).max(0.35),
            );
            let strike_pitch = to_target
                .z
                .atan2(to_target.y)
                .to_degrees()
                .clamp(75.0, 122.0);
            let strike_dir = Vec3::new(
                0.0,
                strike_pitch.to_radians().cos(),
                strike_pitch.to_radians().sin(),
            );
            let shift_along = (to_target.length() - 1.27).clamp(-0.22, 0.26);
            let strike_at = anchor + strike_dir * shift_along - Vec3::new(0.0, 0.10, -0.27);
            pitch = -8.0 + 8.0 * draw + (strike_pitch + 8.0) * strike;
            at = Vec3::new(-0.23, 0.34, 0.42) * draw + strike_at * strike;
            let strike_grip = anchor + strike_dir * 0.22;
            let base_r = Vec3::new(-0.22, 0.22, 0.24);
            let draw_r = Vec3::new(-0.22, 0.47, 0.24);
            let hit_r = Vec3::new(-0.11, strike_grip.y, strike_grip.z);
            let grip = base_r + (draw_r - base_r) * draw + (hit_r - base_r) * strike;
            let base_l = Vec3::new(0.0, 0.14, 0.26);
            let draw_l = Vec3::new(0.0, 0.39, 0.26);
            let hit_l = Vec3::new(0.11, strike_grip.y - 0.06, strike_grip.z + 0.02);
            let left_grip = base_l + (draw_l - base_l) * draw + (hit_l - base_l) * strike;
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
                left_grip,
                Some(Quat::IDENTITY),
                (&plan, Vec3::new(0.8, -0.4, 0.4)),
            );
            tools::face(p, m.aim, Vec3::ZERO);
            tools::plant(p, &plan, tools::stance(&plan, 1.12, 0.03));
        }
        Clip::Hit => {
            moves::hit(m, p, 0.25);
            tools::face(p, m.aim, Vec3::ZERO);
        }
        Clip::Spawn => moves::spawn(m, p, &plan, Entry::Rise),
        Clip::Evolve => moves::evolve(m, p, &plan, Power::Raise),
        Clip::Death => moves::death(m, p, &plan, Fall::Kneel),
        Clip::Victory => {
            // GD-04: a monument's salute: both fists rise in turn with a planted stomp.
            let beat = (m.clock * 2.0).sin();
            let left = beat.max(0.0);
            let right = (-beat).max(0.0);
            tools::add(p, Chest, [-4.0, 6.0 * beat, 0.0]);
            tools::add(p, Head, [-8.0, 0.0, 0.0]);
            tools::add(p, UpperArmL, [-70.0 - 60.0 * left, 0.0, 20.0]);
            tools::add(p, ForearmL, [-35.0 + 20.0 * left, 0.0, 0.0]);
            tools::add(p, UpperArmR, [-20.0 - 110.0 * right, 0.0, -18.0]);
            tools::add(p, ForearmR, [-30.0 + 15.0 * right, 0.0, 0.0]);
            tools::shift(p, Hips, [0.0, -0.016 * beat.abs(), 0.0]);
        }
    }
    let weapon = &mut p[Node::Prop(Socket::WeaponR).index()];
    weapon.0 = Vec3::new(pitch, 0.0, 0.0);
    weapon.1 = at;
}
