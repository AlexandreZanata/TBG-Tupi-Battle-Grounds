//! Astreon's lance guard and thrust: both wrists stay on one shaft line in every
//! clip, the butt trails behind the right hip and the point lands on the target.

use super::{lance, model};
use crate::art::{
    anim::{Clip, Pose},
    ik,
    kits::{aim, common::Form, tools, Moment},
    rig::{Joint::*, Node::*, Proportions, Socket, Spring},
};
use bevy::prelude::*;
use std::f32::consts::TAU;

/// Rear grip in chest space: right of the sternum, in front of the cuirass.
const GUARD: Vec3 = Vec3::new(-0.19, 0.26, 0.28);
/// Shaft yaw: the point leads to the left-front, the butt trails right-back.
const SHAFT_YAW: f32 = 42.0;

pub(super) fn pose(m: &Moment, p: &mut Pose) {
    p.fill((Vec3::ZERO, Vec3::ZERO));
    let body = model::plan(m.tier);
    let (slide, pitch) = match m.clip {
        Clip::Idle => idle(m, p),
        Clip::Walk => walk(m, p),
        Clip::AttackMelee | Clip::AttackRanged | Clip::Cast => attack(m, p),
        Clip::Hit => hit(m, p),
        Clip::Death => death(m, p, &body),
        Clip::Spawn => spawn(m, p),
        Clip::Evolve => evolve(m, p),
        Clip::Victory => victory(m, p),
    };
    if m.clip != Clip::Walk {
        tools::plant(p, &body, tools::stance(&body, 1.3, 0.12));
    }
    grip(m, p, &body, slide, pitch);
}

fn idle(m: &Moment, p: &mut Pose) -> (f32, f32) {
    let breath = (m.clock * 1.25).sin();
    tools::set(p, Spine, [1.5 + 1.5 * breath, 0.0, 0.0]);
    tools::set(p, Head, [-2.0 - breath, 4.0 * (m.clock * 0.5).sin(), 0.0]);
    p[Secondary(Spring::Back).index()].0.x = 3.0 + 2.0 * breath;
    p[Secondary(Spring::HeadGear).index()].0.x = 1.5 * breath;
    (0.0, -34.0 + 1.5 * breath)
}

fn walk(m: &Moment, p: &mut Pose) -> (f32, f32) {
    let step = (m.walk.unwrap_or(m.t / 0.65) * TAU).sin();
    tools::set(p, Chest, [5.0, 3.0 * step, 1.5 * step]);
    tools::set(p, Head, [-4.0, -2.0 * step, -step]);
    p[Secondary(Spring::Back).index()].0.x = 9.0 + 3.0 * step.abs();
    (-0.01 * step, -30.0 + 3.0 * step)
}

/// Draw back, then one straight thrust; the strike event at 0.30 s never moves.
fn stroke(t: f32) -> (f32, f32) {
    (
        tools::hold(t, [0.0, 0.16, 0.20, 0.30]),
        tools::hold(t, [0.20, 0.30, 0.38, 0.70]),
    )
}

fn attack(m: &Moment, p: &mut Pose) -> (f32, f32) {
    let (load, contact) = stroke(m.t);
    let f = Form::of(m.tier, m.branch);
    // The pierce (counted) and the B executions lunge deeper and heavier.
    let verdict = if m.counter >= 2 { 1.0 } else { 0.0 };
    let weight = 1.0 + 0.35 * verdict + if f.b { 0.25 } else { 0.0 };
    tools::set(p, Chest, [3.0 + 5.0 * contact * weight, -10.0 * load, 0.0]);
    tools::set(p, Head, [-3.0 - 3.0 * contact, 6.0 * load, 0.0]);
    tools::shift(
        p,
        Hips,
        [
            0.0,
            -0.03 * load - 0.04 * contact * weight,
            -0.05 * load + 0.06 * contact * weight,
        ],
    );
    p[Secondary(Spring::Back).index()].0.x = -6.0 * load + 8.0 * contact;
    p[Secondary(Spring::HeadGear).index()].0.x = 6.0 * contact;
    (-0.12 * load + 0.10 * contact, -34.0 + 32.0 * contact)
}

fn hit(m: &Moment, p: &mut Pose) -> (f32, f32) {
    let recoil = tools::hold(m.t, [0.0, 0.05, 0.09, 0.25]);
    tools::set(p, Chest, [-6.0 * recoil, 0.0, -3.0 * recoil]);
    tools::set(p, Head, [-5.0 * recoil, 0.0, 2.0 * recoil]);
    tools::shift(p, Hips, [0.0, -0.02 * recoil, -0.03 * recoil]);
    (-0.04 * recoil, -34.0 + 6.0 * recoil)
}

/// He loses balance onto one knee, the lance sinking to the ground in both hands.
fn death(m: &Moment, p: &mut Pose, body: &Proportions) -> (f32, f32) {
    let stagger = tools::hold(m.t, [0.0, 0.10, 0.18, 0.30]);
    let kneel = tools::window(m.t, 0.12, 0.70);
    let bow = tools::window(m.t, 0.35, 0.95);
    tools::shift(p, Hips, [0.0, -body.leg * 0.44 * kneel, -0.05 * stagger]);
    tools::set(p, Spine, [-8.0 * stagger + 16.0 * bow, 0.0, 5.0 * bow]);
    tools::set(p, Head, [12.0 * bow, 0.0, 0.0]);
    p[Secondary(Spring::Back).index()].0.x = -10.0 * bow;
    (-0.02 * bow, -34.0 + 70.0 * kneel)
}

fn spawn(m: &Moment, p: &mut Pose) -> (f32, f32) {
    let rise = tools::window(m.t, 0.0, 0.45);
    tools::shift(p, Root, [0.0, -0.22 * (1.0 - rise), 0.0]);
    tools::set(p, Head, [-8.0 * (1.0 - rise), 0.0, 0.0]);
    (0.0, -60.0 + 26.0 * rise)
}

fn evolve(m: &Moment, p: &mut Pose) -> (f32, f32) {
    let gather = tools::hold(m.t, [0.0, 0.16, 0.26, 0.45]);
    let display = tools::hold(m.t, [0.30, 0.48, 0.70, 0.95]);
    tools::shift(p, Hips, [0.0, -0.05 * gather, 0.0]);
    tools::set(p, Chest, [4.0 * gather - 6.0 * display, 0.0, 0.0]);
    tools::set(p, Head, [-10.0 * display, 0.0, 0.0]);
    p[Secondary(Spring::Back).index()].0.x = -12.0 * display;
    (0.04 * display, -34.0 - 30.0 * display)
}

/// Conquest: the point rises skyward on each beat and settles back to guard.
fn victory(m: &Moment, p: &mut Pose) -> (f32, f32) {
    let phase = (m.t / 1.0).rem_euclid(1.0);
    let raise = tools::hold(phase, [0.0, 0.25, 0.55, 0.95]);
    let beat = (phase * TAU).sin();
    tools::shift(p, Root, [0.0, 0.02 * raise, 0.0]);
    tools::set(p, Chest, [-5.0 * raise, 3.0 * beat, 0.0]);
    tools::set(p, Head, [-12.0 * raise, -3.0 * beat, 0.0]);
    p[Secondary(Spring::Back).index()].0.x = 10.0 * raise;
    (0.06 * raise, -34.0 - 40.0 * raise)
}

fn grip(m: &Moment, p: &mut Pose, body: &Proportions, mut slide: f32, pitch: f32) {
    let chest = tools::chest(p, body);
    let thrust = matches!(m.clip, Clip::AttackMelee | Clip::AttackRanged | Clip::Cast);
    let (_, contact) = stroke(m.t);
    let tip = lance::tip(Form::of(m.tier, m.branch));
    let pitch = match m.aim {
        Some(aim) if thrust => {
            let target = chest.inverse().transform_point3(aim.target);
            let delta = target - GUARD;
            let wanted = -delta.y.atan2(delta.xz().length()).to_degrees();
            // The point lands just inside the target instead of spearing past it.
            let fit = (delta.length() - tip.z + 0.06).clamp(-0.30, 0.12);
            slide += (fit - slide) * aim.weight * contact;
            pitch + (wanted.clamp(-14.0, 26.0) + 2.0 - pitch) * aim.weight * contact
        }
        _ => pitch,
    };
    let rotation =
        Quat::from_rotation_y(SHAFT_YAW.to_radians()) * Quat::from_rotation_x(pitch.to_radians());
    let origin = GUARD + rotation * Vec3::Z * slide;
    p[Prop(Socket::WeaponR).index()] = (ik::degrees(rotation), origin);
    solve_hands(p, body, rotation, origin);
    if let Some(aim) = m.aim {
        let origin = chest.transform_point3(origin);
        let direction = chest.transform_vector3(rotation * Vec3::Z);
        p[Root.index()].0.y +=
            aim::line_yaw(origin, direction, aim.target).to_degrees() * aim.weight;
    }
}

fn solve_hands(p: &mut Pose, body: &Proportions, rotation: Quat, origin: Vec3) {
    for (left, anchor, pole) in [
        (false, Vec3::ZERO, Vec3::new(-1.0, -0.4, 0.0)),
        (true, lance::SUPPORT, Vec3::new(1.0, -0.4, 0.0)),
    ] {
        tools::reach(
            p,
            left,
            origin + rotation * anchor,
            Some(rotation),
            (body, pole),
        );
    }
}

/// Restore both hand constraints after a pose crossfade, on the blended lance frame.
pub(crate) fn finish_blend(pose: &mut Pose, body: &Proportions) {
    let (rotation, origin) = pose[Prop(Socket::WeaponR).index()];
    solve_hands(pose, body, ik::quat(rotation), origin);
}
