//! Archer-only two-bone aiming keeps the drawing hand on its animated bowstring.

use super::{
    anim::{Easing, Pose},
    rig::{self, Joint, Node},
};
use bevy::prelude::*;

pub fn draw(t: f32) -> f32 {
    if t < 0.12 {
        Easing::Linear.sample((t / 0.12).clamp(0.0, 1.0))
    } else if t < 0.18 {
        1.0
    } else {
        (1.0 - (t - 0.18) / 0.04).clamp(0.0, 1.0)
    }
}
/// Two-bone IK in chest space for the default Archer anatomy.
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
