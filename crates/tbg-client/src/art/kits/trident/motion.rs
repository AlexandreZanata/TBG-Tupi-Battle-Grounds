//! A planted shark guard: both hands follow a single spear line through every life clip.

use super::{model, weapon};
use crate::art::{
    anim::{Clip, Pose},
    ik,
    kits::{aim, tools, Moment},
    rig::{Joint::*, Node::*, Proportions, Socket, Spring},
};
use bevy::prelude::*;
use std::f32::consts::TAU;

const GUARD: Vec3 = Vec3::new(-0.27, 0.32, 0.31);
const SHAFT_YAW: f32 = 50.0;

pub(super) fn pose(m: &Moment, p: &mut Pose) {
    // This kit owns the whole pose: no inherited overhead chop may survive below the grip.
    p.fill((Vec3::ZERO, Vec3::ZERO));
    let body = model::plan(m.tier);
    tools::shift(p, Hips, [0.0, -0.015, 0.0]);
    let (mut slide, pitch) = match m.clip {
        Clip::Idle => idle(m, p),
        Clip::Walk => walk(m, p),
        Clip::AttackMelee | Clip::AttackRanged | Clip::Cast => attack(m.t, p),
        Clip::Hit => hit(m, p),
        Clip::Death => death(m, p, &body),
        Clip::Spawn => spawn(m, p),
        Clip::Evolve => evolve(m, p),
        Clip::Victory => victory(m, p),
    };
    if let Some(t) = m.special.filter(|_| m.clip != Clip::Death) {
        // The independent Bloody Rend channel uses the same braced piercing line.
        let (load, contact) = stroke(t, 0.75);
        if load > 0.0 || contact > 0.0 {
            slide = -0.06 * load + 0.14 * contact;
        }
        tools::add(p, Chest, [2.0 * contact, 0.0, 0.0]);
    }
    if m.clip != Clip::Walk {
        tools::plant(p, &body, tools::stance(&body, 1.25, 0.08));
    }
    grip(m, p, &body, slide.clamp(-0.06, 0.14), pitch);
}

fn idle(m: &Moment, p: &mut Pose) -> (f32, f32) {
    let breath = (m.clock * 1.45).sin();
    tools::set(p, Spine, [1.5 + 1.5 * breath, 0.0, 0.5 * breath]);
    tools::set(p, Head, [-2.0 - breath, 3.0 * (m.clock * 0.55).sin(), 0.0]);
    p[Secondary(Spring::HeadGear).index()].0.x = 2.0 + 2.0 * breath;
    p[Secondary(Spring::Back).index()].0.z = -2.0 * breath;
    (0.0, -20.0 + breath)
}

fn walk(m: &Moment, p: &mut Pose) -> (f32, f32) {
    let step = (m.walk.unwrap_or(m.t / 0.6) * TAU).sin();
    tools::set(p, Chest, [4.0, 2.0 * step, 1.5 * step]);
    tools::set(p, Head, [-3.0, -2.0 * step, -step]);
    p[Secondary(Spring::Back).index()].0.z = -6.0 * step;
    p[Secondary(Spring::HeadGear).index()].0.x = 2.0;
    (-0.01 * step, -20.0 + 2.0 * step)
}

fn stroke(t: f32, recovery: f32) -> (f32, f32) {
    (
        tools::hold(t, [0.0, 0.14, 0.18, 0.30]),
        tools::hold(t, [0.18, 0.30, 0.35, recovery]),
    )
}

fn attack(t: f32, p: &mut Pose) -> (f32, f32) {
    let (load, contact) = stroke(t, 0.62);
    tools::set(p, Chest, [2.0 + 3.0 * contact, 0.0, 0.0]);
    tools::set(p, Head, [-2.0 - 3.0 * contact, 0.0, 0.0]);
    tools::shift(
        p,
        Hips,
        [
            0.0,
            -0.025 * load - 0.035 * contact,
            -0.025 * load + 0.035 * contact,
        ],
    );
    p[Secondary(Spring::HeadGear).index()].0.x = 3.0 + 5.0 * contact;
    p[Secondary(Spring::Back).index()].0.x = -5.0 * load + 3.0 * contact;
    (-0.06 * load + 0.14 * contact, -20.0 + 28.0 * contact)
}

fn hit(m: &Moment, p: &mut Pose) -> (f32, f32) {
    let recoil = tools::hold(m.t, [0.0, 0.05, 0.09, 0.25]);
    tools::set(p, Chest, [-6.0 * recoil, 0.0, -3.0 * recoil]);
    tools::set(p, Head, [-5.0 * recoil, 0.0, 2.0 * recoil]);
    tools::shift(p, Hips, [0.0, -0.025 * recoil, -0.025 * recoil]);
    (-0.025 * recoil, 8.0 + 2.0 * recoil)
}

fn death(m: &Moment, p: &mut Pose, body: &Proportions) -> (f32, f32) {
    let kneel = tools::window(m.t, 0.08, 0.70);
    let bow = tools::window(m.t, 0.30, 0.90);
    tools::shift(p, Hips, [0.0, -body.leg * 0.43 * kneel, 0.0]);
    tools::set(p, Spine, [12.0 * bow, 0.0, 3.0 * bow]);
    tools::set(p, Head, [10.0 * bow, 0.0, 0.0]);
    p[Secondary(Spring::Back).index()].0.x = -12.0 * bow;
    // Keep both hands on the lowered spear instead of reverting to a loose shared fall.
    (-0.015 * bow, 8.0 - 15.0 * bow)
}

fn spawn(m: &Moment, p: &mut Pose) -> (f32, f32) {
    let rise = tools::window(m.t, 0.0, 0.42);
    tools::shift(p, Root, [0.0, -0.25 * (1.0 - rise), 0.0]);
    tools::shift(p, Hips, [0.0, -0.04 * (1.0 - rise), 0.0]);
    tools::set(p, Head, [-8.0 * (1.0 - rise), 0.0, 0.0]);
    p[Secondary(Spring::Back).index()].0.x = -8.0 * (1.0 - rise);
    (0.0, 8.0 - 8.0 * (1.0 - rise))
}

fn evolve(m: &Moment, p: &mut Pose) -> (f32, f32) {
    let gather = tools::hold(m.t, [0.0, 0.14, 0.24, 0.45]);
    let display = tools::hold(m.t, [0.30, 0.45, 0.65, 0.90]);
    tools::shift(p, Hips, [0.0, -0.045 * gather, 0.0]);
    tools::set(p, Chest, [2.0 - 3.0 * display, 0.0, 0.0]);
    tools::set(p, Head, [-6.0 * display, 0.0, 0.0]);
    p[Secondary(Spring::Back).index()].0.x = -12.0 * display;
    p[Secondary(Spring::HeadGear).index()].0.x = 5.0 * display;
    (0.02 * display, 8.0 - 12.0 * display)
}

fn victory(m: &Moment, p: &mut Pose) -> (f32, f32) {
    let phase = (m.t / 0.8).rem_euclid(1.0);
    let salute = tools::hold(phase, [0.0, 0.20, 0.40, 0.80]);
    let beat = (phase * TAU).sin();
    tools::shift(p, Root, [0.0, 0.025 * salute, 0.0]);
    tools::set(p, Chest, [-2.0 * salute, 0.0, 2.0 * beat]);
    tools::set(p, Head, [-5.0 * salute, -4.0 * beat, 0.0]);
    p[Secondary(Spring::Back).index()].0.z = -4.0 * beat;
    p[Secondary(Spring::HeadGear).index()].0.x = 2.0 + 3.0 * salute;
    (0.03 * salute, 8.0 - 20.0 * salute)
}

fn grip(m: &Moment, p: &mut Pose, body: &Proportions, slide: f32, pitch: f32) {
    let chest = tools::chest(p, body);
    let pitch = m.aim.map_or(pitch, |aim| {
        let target = chest.inverse().transform_point3(aim.target);
        let delta = target - GUARD;
        let wanted = -delta.y.atan2(delta.xz().length()).to_degrees();
        // Carry/load above the opposing body; lower onto it only during the thrust.
        let contact = if matches!(m.clip, Clip::AttackMelee | Clip::AttackRanged | Clip::Cast) {
            stroke(m.t, 0.62).1
        } else {
            0.0
        };
        pitch + (wanted.clamp(-12.0, 28.0) - 8.0) * aim.weight * contact
    });
    let rotation =
        Quat::from_rotation_y(SHAFT_YAW.to_radians()) * Quat::from_rotation_x(pitch.to_radians());
    let origin = GUARD + rotation * Vec3::Z * slide;
    p[Prop(Socket::WeaponR).index()] = (ik::degrees(rotation), origin);
    for (left, anchor, pole) in [
        (false, Vec3::ZERO, Vec3::new(-1.0, -0.4, 0.0)),
        (true, weapon::SUPPORT, Vec3::new(1.0, -0.4, 0.0)),
    ] {
        tools::reach(
            p,
            left,
            origin + rotation * anchor,
            Some(rotation),
            (body, pole),
        );
    }
    if let Some(aim) = m.aim {
        let origin = chest.transform_point3(origin);
        let direction = chest.transform_vector3(rotation * Vec3::Z);
        p[Root.index()].0.y +=
            aim::line_yaw(origin, direction, aim.target).to_degrees() * aim.weight;
    }
}

/// Restore the two hand constraints after a pose crossfade, using the actual spear frame.
pub(crate) fn finish_blend(pose: &mut Pose, body: &Proportions) {
    let (rotation, origin) = pose[Prop(Socket::WeaponR).index()];
    let rotation = ik::quat(rotation);
    for (left, anchor, pole) in [
        (false, Vec3::ZERO, Vec3::new(-1.0, -0.4, 0.0)),
        (true, weapon::SUPPORT, Vec3::new(1.0, -0.4, 0.0)),
    ] {
        tools::reach(
            pose,
            left,
            origin + rotation * anchor,
            Some(rotation),
            (body, pole),
        );
    }
}
