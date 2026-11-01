//! Siege Walker motion. The bombard rides in the right hand and both arms
//! follow it by IK: it rests on the shoulder at ease, is shouldered and
//! aimed along its own shell's launch tangent when engaged (the body turns
//! so the off-center barrel lines up), kicks back on Release, and is
//! cycled with a forward side lever between shots. Heavy walk, drop-in spawn.

use std::f32::consts::TAU;

use bevy::prelude::*;

use super::{
    model::{bombard, plan, Bombard, Form},
    reload,
};
use crate::art::{
    anim::{Clip, Pose},
    kits::{
        aim,
        tools::{self, hold, snap, window},
        Moment,
    },
    rig::{Joint, Proportions},
};
use crate::vfx::shots;

/// Where the gun sits in chest space: grip position and orientation.
#[derive(Clone, Copy)]
struct Carry {
    at: Vec3,
    turn: Quat,
}

/// Resting on the shoulder, muzzle tipped up.
fn at_ease() -> Carry {
    Carry {
        at: Vec3::new(-0.27, 0.17, 0.08),
        turn: Quat::from_rotation_y(6f32.to_radians()) * Quat::from_rotation_x(-18f32.to_radians()),
    }
}

/// Shouldered and level; the chest twists the left shoulder forward and
/// the gun turns back the same amount, so it still points ahead.
const TWIST: f32 = 20.0;
fn braced() -> Carry {
    Carry {
        at: Vec3::new(-0.25, 0.16, 0.14),
        turn: Quat::from_rotation_y(TWIST.to_radians()),
    }
}

fn blend(a: Carry, b: Carry, w: f32) -> Carry {
    Carry {
        at: a.at.lerp(b.at, w),
        turn: a.turn.slerp(b.turn, w),
    }
}

pub fn pose(m: &Moment, pose: &mut Pose) {
    let f = Form::of(m.tier, m.branch);
    let p = plan(m.tier);
    let gun = bombard(f);
    let engaged = m.aim.map_or(0.0, |a| a.weight);
    let (mut carry, brace) = match m.clip {
        Clip::Walk => (walk(m, pose), 0.0),
        Clip::AttackRanged => fire(m, pose),
        Clip::Spawn => (spawn(m, pose, &p), 0.0),
        Clip::Evolve => (salute(m, pose, &p), 0.0),
        Clip::Idle => (idle(m, pose, engaged), engaged),
        Clip::Victory => (victory(m, pose, &p), 0.0),
        _ => (at_ease(), 0.0),
    };
    if brace > 0.0 {
        stance(pose, &p, brace);
        aim_gun(m, pose, (&p, &gun), &mut carry, brace);
    }
    let support = match m.clip {
        Clip::Idle => brace,
        Clip::AttackRanged | Clip::Evolve | Clip::Victory => 1.0,
        _ => 0.0,
    };
    hands(m, pose, (&p, &gun), carry, support);
}

fn idle(m: &Moment, pose: &mut Pose, engaged: f32) -> Carry {
    let breath = (m.clock * 1.7).sin();
    tools::add(pose, Joint::Chest, [2.5 * breath, 0.0, 0.0]);
    tools::shift(pose, Joint::Chest, [0.0, 0.006 * breath, 0.0]);
    tools::add(
        pose,
        Joint::Head,
        [0.0, 10.0 * tools::wave(m.clock, 0.4, 1.0), 0.0],
    );
    let settle = hold((m.clock * 0.22).fract(), [0.08, 0.16, 0.22, 0.38]) * (1.0 - engaged);
    tools::add(pose, Joint::Chest, [-3.0 * settle, 0.0, -2.0 * settle]);
    let mut carry = blend(at_ease(), braced(), engaged);
    carry.at.y += 0.024 * settle;
    carry
}

fn walk(m: &Moment, pose: &mut Pose) -> Carry {
    let phase = m.walk.unwrap_or(0.0) * TAU;
    // Lumbering roll from foot to foot, chest leaning into the stride.
    tools::set(pose, Joint::Spine, [5.0, 0.0, 4.0 * phase.sin()]);
    tools::set(pose, Joint::Chest, [3.0, -5.0 * phase.sin(), 0.0]);
    tools::add(
        pose,
        Joint::Head,
        [-4.0, 4.0 * phase.sin(), -3.0 * phase.sin()],
    );
    let mut carry = at_ease();
    carry.at.y += 0.012 * (phase * 2.0).cos();
    carry.turn *= Quat::from_rotation_z((2.0 * phase.sin()).to_radians());
    tools::set(
        pose,
        Joint::ForearmL,
        [-24.0 - 10.0 * phase.sin(), 0.0, 0.0],
    );
    carry
}

/// Brace (0–0.12 s), Release and recoil at 0.18 s, recover by 0.62 s.
fn fire(m: &Moment, pose: &mut Pose) -> (Carry, f32) {
    let t = m.t;
    let brace = window(t, 0.0, 0.12).max(m.aim.map_or(0.0, |a| a.weight));
    let kick = snap(t, 0.18, 0.21) * (1.0 - window(t, 0.24, 0.55));
    let mut carry = braced();
    carry.at -= carry.turn * Vec3::Z * 0.11 * kick;
    carry.turn *= Quat::from_rotation_x(-16f32.to_radians() * kick);
    tools::add(pose, Joint::Chest, [-8.0 * kick, 0.0, 0.0]);
    tools::shift(pose, Joint::Root, [0.0, 0.0, -0.07 * kick]);
    tools::shift(pose, Joint::Hips, [0.0, -0.025 * kick, 0.0]);
    tools::add(pose, Joint::Head, [6.0 * kick, 0.0, 0.0]);
    tools::add(pose, Joint::Spine, [2.0 * brace - 3.0 * kick, 0.0, 0.0]);
    (carry, brace)
}

/// A planted artillery salute keeps both hands on the existing gun grips.
fn victory(m: &Moment, pose: &mut Pose, p: &Proportions) -> Carry {
    let lift = 0.5 - 0.5 * (m.t / 0.8 * TAU).cos();
    tools::set(pose, Joint::Spine, [-4.0 * lift, 0.0, 0.0]);
    tools::set(pose, Joint::Chest, [0.0, -8.0, 0.0]);
    tools::set(pose, Joint::Head, [-8.0 - 12.0 * lift, 8.0, 0.0]);
    tools::plant(pose, p, tools::stance(p, 1.3, 0.04));
    let mut carry = braced();
    carry.turn *= Quat::from_rotation_x((-15.0 - 30.0 * lift).to_radians());
    carry.at.y += 0.025 * lift;
    carry
}

fn spawn(m: &Moment, pose: &mut Pose, p: &Proportions) -> Carry {
    // Dropped onto the field: falls, lands at 60 %, squats, stands.
    let k = (m.t / 0.5).clamp(0.0, 1.0);
    let fall = (1.0 - k / 0.6).max(0.0);
    pose[Joint::Root.index()].1 = Vec3::Y * 0.9 * fall * fall;
    let squat = hold(k, [0.55, 0.65, 0.7, 1.0]);
    tools::shift(pose, Joint::Hips, [0.0, -0.08 * squat, 0.0]);
    tools::add(pose, Joint::Spine, [14.0 * squat, 0.0, 0.0]);
    tools::plant(pose, p, tools::stance(p, 1.25, 0.04));
    at_ease()
}

/// Evolve (0.9 s): crouch and raise the bombard to the sky, fire a salute
/// at 0.35 s with a hop, settle back onto the shoulder.
fn salute(m: &Moment, pose: &mut Pose, p: &Proportions) -> Carry {
    let k = (m.t / 0.9).clamp(0.0, 1.0);
    let rise = window(k, 0.0, 0.3) * (1.0 - window(k, 0.75, 1.0));
    let kick = snap(k, 0.38, 0.42) * (1.0 - window(k, 0.45, 0.75));
    pose[Joint::Root.index()] = (Vec3::ZERO, Vec3::Y * 0.08 * hold(k, [0.3, 0.38, 0.42, 0.6]));
    tools::add(pose, Joint::Spine, [-10.0 * rise, 0.0, 0.0]);
    tools::add(pose, Joint::Head, [-25.0 * rise, 0.0, 0.0]);
    tools::shift(pose, Joint::Hips, [0.0, -0.05 * rise, 0.0]);
    tools::plant(pose, p, tools::stance(p, 1.4, 0.05));
    let mut carry = braced();
    carry.turn *= Quat::from_rotation_x((-65.0 * rise - 12.0 * kick).to_radians());
    carry.at += Vec3::new(0.02, 0.06 * rise, -0.04 * rise) - carry.turn * Vec3::Z * 0.08 * kick;
    carry
}

/// Wide, knee-bent firing stance, left foot forward.
fn stance(pose: &mut Pose, p: &Proportions, w: f32) {
    tools::shift(pose, Joint::Hips, [0.0, -0.035 * w, 0.0]);
    tools::add(pose, Joint::Chest, [0.0, -TWIST * w, 0.0]);
    tools::add(pose, Joint::Head, [0.0, TWIST * 0.8 * w, 0.0]);
    tools::plant(pose, p, tools::stance(p, 1.0 + 0.45 * w, 0.07 * w));
}

/// Turn the body so the barrel line passes over the target, then raise the
/// gun to the launch angle of its own shell (30 % chest, 70 % arms).
fn aim_gun(
    m: &Moment,
    pose: &mut Pose,
    (p, gun): (&Proportions, &Bombard),
    carry: &mut Carry,
    w: f32,
) {
    let Some(target) = m.aim.map(|a| a.target - pose[Joint::Root.index()].1) else {
        return;
    };
    let chest = tools::chest(pose, p);
    let muzzle = chest.transform_point3(carry.at + carry.turn * gun.muzzle);
    let dir = chest
        .transform_vector3(carry.turn * Vec3::Z)
        .normalize_or(Vec3::Z);
    let yaw = aim::line_yaw(muzzle, dir, target);
    let turn = Quat::from_rotation_y(yaw);
    let arc = shots::arc(super::shot(m.tier, m.branch));
    let need = aim::tangent(turn * muzzle, target, arc, 0.0);
    let up = (aim::pitch(need) - aim::pitch(dir)).clamp(-0.6, 0.9) * w;
    tools::add(pose, Joint::Root, [0.0, yaw.to_degrees() * w, 0.0]);
    tools::add(pose, Joint::Chest, [-(up * 0.3).to_degrees(), 0.0, 0.0]);
    carry.turn *= Quat::from_rotation_x(-up * 0.7);
}

/// Keep the carrying hand steady while the support hand cycles the external lever.
fn hands(
    m: &Moment,
    pose: &mut Pose,
    (p, gun): (&Proportions, &Bombard),
    carry: Carry,
    support: f32,
) {
    let right_pole = Vec3::new(-1.0, -0.7, -0.5);
    tools::reach(pose, false, carry.at, Some(carry.turn), (p, right_pole));
    reload::support(m, pose, p, carry.at, carry.turn, gun.fore_grip, support);
}
