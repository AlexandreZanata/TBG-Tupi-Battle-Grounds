//! Two-bone walking legs: each foot swings along the real direction of
//! travel and stays planted in world X and Z while in contact. Cadence is
//! the distance travelled (`anim.rs`), so a planted foot never slides.

use super::{
    curves::Pose,
    ik,
    kits::Gait,
    rig::{self, Joint, Node, Proportions},
};
use bevy::prelude::*;
use std::f32::consts::TAU;

/// One frame of a walk: cycle phase (two steps per 1.0), the direction of
/// travel in root space, and whether feet lock to the ground.
pub struct Stride {
    pub phase: f32,
    pub dir: Vec3,
    pub lock: bool,
}

#[derive(Default)]
pub struct FootLocks {
    anchors: [Option<Vec2>; 2],
    cycles: [i32; 2],
}

pub const LEGS: [(Joint, Joint, Joint); 2] = [
    (Joint::ThighL, Joint::ShinL, Joint::FootL),
    (Joint::ThighR, Joint::ShinR, Joint::FootR),
];

impl FootLocks {
    pub fn reset(&mut self) {
        self.anchors = [None; 2];
    }
    /// Pose hips and both legs for this frame. `frame` maps root-joint
    /// space to the world.
    pub fn apply(&mut self, pose: &mut Pose, s: &Stride, frame: Mat4, p: &Proportions, g: &Gait) {
        let inverse = frame.inverse();
        let bob = (s.phase * TAU * 2.0).cos();
        pose[Joint::Hips.index()].1.y = -p.leg * (g.sink + g.bob * 0.5) + bob * p.leg * g.bob;
        let stride = g.cycle(p.leg);
        let scale = frame.transform_vector3(Vec3::X).length();
        for (side, (thigh, _, _)) in LEGS.into_iter().enumerate() {
            let raw = s.phase + side as f32 * 0.5;
            let k = raw.rem_euclid(1.0);
            let contact = k < 0.5;
            let swing = ((k - 0.5) * 2.0).clamp(0.0, 1.0);
            let along = if contact {
                stride * (0.25 - k)
            } else {
                stride * (-0.25 + 0.5 * (3.0 - 2.0 * swing) * swing * swing)
            };
            let lift = if contact {
                0.0
            } else {
                (swing * TAU * 0.5).sin() * p.leg * g.lift
            };
            let hip_rest = rig::rest(Node::Body(thigh), p);
            let ground = Vec3::new(hip_rest.x, 0.06 + lift, 0.0) + s.dir * along;
            let mut world = frame.transform_point3(ground);
            if contact && s.lock {
                let anchor = match self.anchors[side] {
                    // Re-plant after a hitch that would overstretch the leg.
                    Some(a)
                        if self.cycles[side] == raw.floor() as i32
                            && a.distance(world.xz()) < stride * 0.6 * scale =>
                    {
                        a
                    }
                    _ => world.xz(),
                };
                self.anchors[side] = Some(anchor);
                self.cycles[side] = raw.floor() as i32;
                world.x = anchor.x;
                world.z = anchor.y;
            } else {
                self.anchors[side] = None;
            }
            // Toes dip at lift-off and come level for the heel strike.
            let toe = Quat::from_rotation_x((swing * TAU * 0.5).sin() * 0.35);
            leg(pose, p, side, inverse.transform_point3(world), toe);
        }
    }
}

/// Bend one leg so its ankle reaches `ankle` (root-joint space), knee
/// forward, the foot level with the root (then turned by `toe`).
pub fn leg(pose: &mut Pose, p: &Proportions, side: usize, ankle: Vec3, toe: Quat) {
    let (thigh, shin, foot) = LEGS[side];
    let hips_rot = ik::quat(pose[Joint::Hips.index()].0);
    let hips_at = rig::rest(Node::Body(Joint::Hips), p) + pose[Joint::Hips.index()].1;
    let hip = hips_at + hips_rot * rig::rest(Node::Body(thigh), p);
    let target = hips_rot.inverse() * (ankle - hip);
    let (u, l) = ik::two_bone(Vec3::ZERO, target, p.leg * 0.5, p.leg * 0.5, Vec3::Z);
    pose[thigh.index()] = (ik::degrees(u), Vec3::ZERO);
    pose[shin.index()].0 = ik::degrees(l);
    pose[foot.index()].0 = ik::degrees((hips_rot * u * l).inverse() * toe);
}
