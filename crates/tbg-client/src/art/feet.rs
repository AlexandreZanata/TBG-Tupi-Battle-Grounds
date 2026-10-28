//! Two-bone walking poses with planted world-X anchors; cadence follows content speed.

use super::{
    curves::Pose,
    rig::{Joint, Proportions},
};
use bevy::prelude::*;

#[derive(Default)]
pub struct FootLocks {
    anchors: [Option<f32>; 2],
    cycles: [i32; 2],
}
impl FootLocks {
    pub fn reset(&mut self) {
        self.anchors = [None; 2];
    }
    pub fn apply(&mut self, pose: &mut Pose, phase: f32, frame: Mat4, p: &Proportions) {
        let inverse = frame.inverse();
        pose[Joint::Hips.index()].1.y =
            -p.leg * 0.05 + (phase * std::f32::consts::TAU * 2.0).cos() * p.leg * 0.015;
        for (side, (thigh, shin, foot)) in [
            (Joint::ThighL, Joint::ShinL, Joint::FootL),
            (Joint::ThighR, Joint::ShinR, Joint::FootR),
        ]
        .into_iter()
        .enumerate()
        {
            let raw = phase + side as f32 * 0.5;
            let k = raw.rem_euclid(1.0);
            let contact = k < 0.5;
            let cycle = raw.floor() as i32;
            let stride = p.leg * 0.8;
            let z = if contact {
                stride * (0.25 - k)
            } else {
                -stride * 0.25 + stride * 0.5 * (k - 0.5) * 2.0
            };
            let lift = if contact {
                0.0
            } else {
                ((k - 0.5) * std::f32::consts::TAU).sin() * p.leg * 0.25
            };
            let x = p.torso.x * 0.25 * if side == 0 { 1.0 } else { -1.0 };
            let mut world = frame.transform_point3(Vec3::new(x, 0.06 + lift, z));
            if contact {
                if self.anchors[side].is_none() || cycle != self.cycles[side] {
                    self.anchors[side] = Some(world.x);
                    self.cycles[side] = cycle;
                }
                if let Some(anchor) = self.anchors[side] {
                    world.x = anchor;
                }
            } else {
                self.anchors[side] = None;
            }
            let target = inverse.transform_point3(world)
                - Vec3::new(x, p.leg + 0.06, 0.0)
                - pose[Joint::Hips.index()].1;
            solve(pose, [thigh, shin, foot], target, p.leg);
        }
    }
}
fn solve(pose: &mut Pose, [thigh, shin, foot]: [Joint; 3], target: Vec3, length: f32) {
    let distance = Vec2::new(target.y, target.z).length().min(length * 0.9999);
    let bend = (distance / length).acos();
    let aim = target.z.atan2(-target.y);
    let upper = (-aim - bend).to_degrees();
    let lower = (2.0 * bend).to_degrees();
    pose[thigh.index()] = (Vec3::new(upper, 0.0, 0.0), Vec3::new(target.x, 0.0, 0.0));
    pose[shin.index()].0 = Vec3::new(lower, 0.0, 0.0);
    pose[foot.index()].0 = Vec3::new(-upper - lower, 0.0, 0.0);
}
