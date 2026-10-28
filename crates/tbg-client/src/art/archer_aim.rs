//! Archer-only two-bone aiming keeps the drawing hand on its animated bowstring.

use super::{
    anim::{Easing, Pose},
    rig::{self, BodyPlan, Joint, Node, Socket},
};
use bevy::prelude::*;

pub fn draw(t: f32) -> f32 {
    if t < 0.12 {
        (t / 0.12).clamp(0.0, 1.0)
    } else if t < 0.18 {
        1.0
    } else {
        (1.0 - (t - 0.18) / 0.04).clamp(0.0, 1.0)
    }
}
pub fn apply(t: f32, pose: &mut Pose) {
    let raise = Easing::CubicOut.sample((t / 0.12).clamp(0.0, 1.0));
    let recover = Easing::CubicInOut.sample(((t - 0.30) / 0.20).clamp(0.0, 1.0));
    let left_idle = Vec3::new(0.19, 0.15, 0.21);
    let right_idle = Vec3::new(-0.20, -0.01, 0.04);
    let left = left_idle
        .lerp(Vec3::new(0.05, 0.29, 0.255), raise)
        .lerp(left_idle, recover);
    let anchor = left + Vec3::new(0.0, 0.08, -0.23 - draw(t) * 0.16);
    let release = Easing::CubicOut.sample(((t - 0.18) / 0.08).clamp(0.0, 1.0));
    let right = right_idle
        .lerp(anchor, raise)
        .lerp(Vec3::new(-0.10, 0.34, -0.12), release)
        .lerp(right_idle, recover);
    let p = BodyPlan::Humanoid.proportions();
    arm(pose, true, left, &p);
    arm(pose, false, right, &p);
    pose[Node::Prop(Socket::OffhandL).index()].0 = Vec3::ZERO;
}
/// Two-bone IK in chest space for the given anatomy (P15 and `?archer=v2`).
pub fn arm(pose: &mut Pose, left: bool, target: Vec3, p: &rig::Proportions) {
    let [upper, fore, hand] = if left {
        [Joint::UpperArmL, Joint::ForearmL, Joint::HandL]
    } else {
        [Joint::UpperArmR, Joint::ForearmR, Joint::HandR]
    };
    let start = rig::rest(Node::Body(upper), p);
    let delta = target - start;
    let distance = delta.length().clamp(0.01, p.arm * 1.9 - 0.0001);
    let direction = delta.normalize_or_zero();
    let pole = Vec3::new(if left { 1.0 } else { -1.0 }, 0.1, 0.0);
    let perpendicular = (pole - direction * pole.dot(direction)).normalize_or_zero();
    let along = (p.arm * p.arm - (p.arm * 0.9).powi(2) + distance * distance) / (2.0 * distance);
    let elbow =
        start + direction * along + perpendicular * (p.arm * p.arm - along * along).max(0.0).sqrt();
    let u = Quat::from_rotation_arc(Vec3::NEG_Y, (elbow - start).normalize_or_zero());
    let f = Quat::from_rotation_arc(Vec3::NEG_Y, (target - elbow).normalize_or_zero());
    for (joint, q) in [(upper, u), (fore, u.inverse() * f), (hand, f.inverse())] {
        let (x, y, z) = q.to_euler(EulerRot::XYZ);
        pose[joint.index()].0 = Vec3::new(x, y, z) * 180.0 / std::f32::consts::PI;
    }
}
