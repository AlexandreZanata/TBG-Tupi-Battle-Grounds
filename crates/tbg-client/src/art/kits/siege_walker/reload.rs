//! A readable support-hand transfer, short lever pull and return in front of the ogre's chest.
use super::model::RELOAD_HANDLE;
use crate::art::{
    anim::{Clip, Pose},
    ik,
    kits::{tools, Moment},
    rig::{Joint, Node, Proportions, Socket},
};
use bevy::prelude::*;

pub(super) fn support(
    m: &Moment,
    pose: &mut Pose,
    body: &Proportions,
    origin: Vec3,
    rotation: Quat,
    fore_grip: Vec3,
    weight: f32,
) {
    let cycling = m.clip == Clip::Idle && m.aim.is_some();
    let age = m.since_attack;
    let transfer = if cycling {
        tools::hold(age, [0.56, 0.76, 1.28, 1.55])
    } else {
        0.0
    };
    let pull = if cycling {
        tools::hold(age, [0.80, 1.01, 1.09, 1.25])
    } else {
        0.0
    };
    let slide = Vec3::new(0.0, 0.0, -0.065 * pull);
    pose[Node::Prop(Socket::Nock).index()].1 = slide;
    if weight <= 0.0 {
        return;
    }
    // Bow outward while changing grip, then pause at each end of the mechanical stroke.
    let waypoint = fore_grip.lerp(RELOAD_HANDLE + slide, transfer)
        + Vec3::X * (std::f32::consts::PI * transfer).sin() * 0.025;
    let target = origin + rotation * waypoint;
    let joints = [Joint::UpperArmL, Joint::ForearmL, Joint::HandL];
    let previous = joints.map(|j| pose[j.index()].0);
    tools::reach(
        pose,
        true,
        target,
        Some(rotation),
        (body, Vec3::new(1.0, -0.2, 1.0)),
    );
    for (joint, from) in joints.into_iter().zip(previous) {
        pose[joint.index()].0 =
            ik::degrees(ik::quat(from).slerp(ik::quat(pose[joint.index()].0), weight));
    }
}
