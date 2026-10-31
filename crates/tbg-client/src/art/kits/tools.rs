//! Helpers every kit shares: pose edits by joint, eased time windows,
//! arm IK toward chest-space points, and the aim turn of a whole figure.

use bevy::prelude::*;

use super::super::{
    anim::Pose,
    clips::Key,
    curves::sample_track,
    feet, ik,
    rig::{self, Joint, Node, Proportions},
};
use super::Aim;

/// A kit's own keyframes for one joint (normalized time, like `clips.rs`).
pub type Track = (Joint, &'static [Key]);

/// Write every track of a kit clip at normalized time `k`.
pub fn play(pose: &mut Pose, tracks: &[Track], k: f32) {
    for (joint, keys) in tracks {
        pose[joint.index()] = sample_track(keys, k);
    }
}

pub fn set(pose: &mut Pose, joint: Joint, deg: [f32; 3]) {
    pose[joint.index()].0 = Vec3::from(deg);
}

pub fn add(pose: &mut Pose, joint: Joint, deg: [f32; 3]) {
    pose[joint.index()].0 += Vec3::from(deg);
}

pub fn shift(pose: &mut Pose, joint: Joint, by: [f32; 3]) {
    pose[joint.index()].1 += Vec3::from(by);
}

/// 0 before `a`, 1 after `b`, cubic in-out between.
pub fn window(k: f32, a: f32, b: f32) -> f32 {
    let t = ((k - a) / (b - a).max(1e-4)).clamp(0.0, 1.0);
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) * 0.5
    }
}

/// Rises over `a..b`, holds, falls over `c..d`.
pub fn hold(k: f32, [a, b, c, d]: [f32; 4]) -> f32 {
    window(k, a, b) * (1.0 - window(k, c, d))
}

/// Sharp snap toward 1 over `a..b` (cubic out), for strikes and recoils.
pub fn snap(k: f32, a: f32, b: f32) -> f32 {
    let t = ((k - a) / (b - a).max(1e-4)).clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

/// Put a wrist at `target` (chest space), the elbow bending toward
/// `pole`, the hand turned to `grip` (chest space) when given.
pub fn reach(
    pose: &mut Pose,
    left: bool,
    target: Vec3,
    grip: Option<Quat>,
    (p, pole): (&Proportions, Vec3),
) {
    let [upper, fore, hand] = if left {
        [Joint::UpperArmL, Joint::ForearmL, Joint::HandL]
    } else {
        [Joint::UpperArmR, Joint::ForearmR, Joint::HandR]
    };
    let shoulder = rig::rest(Node::Body(upper), p);
    let (u, l) = ik::two_bone(shoulder, target, p.arm, p.arm * 0.9, pole);
    pose[upper.index()].0 = ik::degrees(u);
    pose[fore.index()].0 = ik::degrees(l);
    if let Some(grip) = grip {
        pose[hand.index()].0 = ik::degrees((u * l).inverse() * grip);
    }
}

/// Bend both legs so the ankles stay at `feet` (root-joint space) after
/// the hips moved: crouches and lunges keep the soles on the ground.
pub fn plant(pose: &mut Pose, p: &Proportions, feet: [Vec3; 2]) {
    for (side, ankle) in feet.into_iter().enumerate() {
        feet::leg(pose, p, side, ankle, Quat::IDENTITY);
    }
}

/// Ankles `wide` × the hip spacing apart, the left one `split` forward.
pub fn stance(p: &Proportions, wide: f32, split: f32) -> [Vec3; 2] {
    let x = rig::rest(Node::Body(Joint::ThighL), p).x * wide;
    [Vec3::new(x, 0.06, split), Vec3::new(-x, 0.06, -split)]
}

/// Root-joint-space transform at the end of a joint chain.
pub fn fk(pose: &Pose, p: &Proportions, chain: &[Joint]) -> Mat4 {
    chain.iter().fold(Mat4::IDENTITY, |m, j| {
        let (r, o) = pose[j.index()];
        m * Mat4::from_rotation_translation(ik::quat(r), rig::rest(Node::Body(*j), p) + o)
    })
}

/// Root-joint-space transform of the chest.
pub fn chest(pose: &Pose, p: &Proportions) -> Mat4 {
    fk(pose, p, &[Joint::Hips, Joint::Spine, Joint::Chest])
}

/// Turn the figure toward its target: the Root takes the yaw (feet pivot
/// in place). Returns the applied yaw in radians.
pub fn face(pose: &mut Pose, aim: Option<Aim>, pivot: Vec3) -> f32 {
    let Some(aim) = aim else {
        return 0.0;
    };
    let yaw = super::aim::yaw(pivot, aim.target) * aim.weight;
    pose[Joint::Root.index()].0.y += yaw.to_degrees();
    yaw
}

/// A deterministic wobble in -1..1 for idle noise.
pub fn wave(clock: f32, speed: f32, seed: f32) -> f32 {
    (clock * speed + seed).sin() * 0.6 + (clock * speed * 2.3 + seed * 1.7).sin() * 0.4
}
