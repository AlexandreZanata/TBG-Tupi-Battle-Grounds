//! Default Archer motion. The attack is a
//! side-on stance: turn and raise (0–0.10 s), draw to the cheek (to 0.12 s),
//! anchor, Release at 0.18 s (unchanged contract), long follow-through with the
//! bow arm held (to 0.40 s), recovery (to 0.50 s). Idle and walk reuse P15.

use super::{
    anim::{Clip, Easing, Pose},
    archer_aim::{arm, draw},
    archer_v2::{plan, DRAW, STRING_LINE},
    rig::{Joint, Node, Socket},
};
use bevy::prelude::*;

/// Chest-space anchor at the chin; the bow hand sits one full draw ahead.
const ANCHOR: Vec3 = Vec3::new(0.05, 0.40, 0.14);
const LEFT_IDLE: Vec3 = Vec3::new(0.19, 0.15, 0.21);
const RIGHT_IDLE: Vec3 = Vec3::new(-0.20, -0.01, 0.04);
/// Where the string hand ends after the release (past the ear).
const FOLLOW: Vec3 = Vec3::new(-0.06, 0.43, 0.10);

/// Chest-space hand targets at full draw: (string hand at the chin, bow hand).
pub fn full_draw() -> (Vec3, Vec3) {
    (ANCHOR, ANCHOR + Vec3::new(STRING_LINE + DRAW, -0.08, 0.0))
}

pub fn apply(clip: Clip, t: f32, pose: &mut Pose) {
    if clip != Clip::AttackRanged {
        locomotion(clip, t, pose);
        return;
    }
    let turn = Easing::CubicIn.sample((t / 0.10).clamp(0.0, 1.0));
    let back = Easing::CubicInOut.sample(((t - 0.40) / 0.10).clamp(0.0, 1.0));
    let k = turn * (1.0 - back);
    stance(pose, k);
    let (_, bow_hand) = full_draw();
    let left = LEFT_IDLE.lerp(bow_hand, k);
    // The socket is turned 90° about Y, so the string travels along -X.
    let string = left + Vec3::new(-(STRING_LINE + draw(t) * DRAW), 0.08, 0.0);
    let released = Easing::CubicOut.sample(((t - 0.18) / 0.10).clamp(0.0, 1.0));
    let hand = if t < 0.18 {
        string
    } else {
        ANCHOR.lerp(FOLLOW, released)
    };
    let right = RIGHT_IDLE.lerp(hand, k);
    let p = plan();
    arm(pose, true, left, &p);
    arm(pose, false, right, &p);
    // The IK leaves the hand world-aligned, so identity holds the bow upright
    // with its belly forward; the aim turns it 90° to face the target.
    let aim = Quat::from_rotation_y(90f32.to_radians());
    pose[Node::Prop(Socket::OffhandL).index()].0 = degrees(Quat::IDENTITY.slerp(aim, k));
}

/// Body turned side-on to the target, feet apart, head along the arrow. The
/// chest stays square to the hips so the chest-space IK above stays valid.
fn stance(pose: &mut Pose, k: f32) {
    for joint in [Joint::Hips, Joint::Spine, Joint::Chest] {
        pose[Node::Body(joint).index()].0 = Vec3::ZERO;
    }
    pose[Node::Body(Joint::Root).index()].0 = Vec3::new(0.0, -90.0 * k, 0.0);
    pose[Node::Body(Joint::Head).index()].0 = Vec3::new(-6.0 * k, 75.0 * k, 0.0);
    pose[Node::Body(Joint::ThighL).index()].0 = Vec3::new(0.0, 0.0, 13.0 * k);
    pose[Node::Body(Joint::ThighR).index()].0 = Vec3::new(0.0, 0.0, -13.0 * k);
    for joint in [Joint::ShinL, Joint::ShinR, Joint::FootL, Joint::FootR] {
        pose[Node::Body(joint).index()].0 = Vec3::ZERO;
    }
}

fn degrees(q: Quat) -> Vec3 {
    let (x, y, z) = q.to_euler(EulerRot::XYZ);
    Vec3::new(x, y, z) * 180.0 / std::f32::consts::PI
}

use super::anim::Easing::*;
use super::rig::{
    Joint::*,
    Node::{Body, Prop},
};
use super::{clips::Key, curves::sample_track};
type Track = (Node, &'static [Key]);
const Z: [f32; 3] = [0.0; 3];

#[rustfmt::skip]
const IDLE: &[Track] = &[
    (Body(UpperArmL), &[(0.0, [-25.0,0.0,-8.0],Z,CubicInOut),(1.0,[-25.0,0.0,-8.0],Z,Hold)]),
    (Body(ForearmL), &[(0.0, [-65.0,0.0,0.0],Z,Hold)]),
    (Prop(Socket::OffhandL), &[(0.0,[90.0,0.0,8.0],Z,Hold)]),
    (Body(UpperArmR), &[(0.0,[-8.0,0.0,-8.0],Z,CubicInOut),(0.5,[-12.0,0.0,-8.0],Z,CubicInOut),(1.0,[-8.0,0.0,-8.0],Z,Hold)]),
];
#[rustfmt::skip]
const WALK: &[Track] = &[
    (Body(UpperArmL), &[(0.0,[-25.0,0.0,-8.0],Z,CubicInOut),(0.5,[-30.0,0.0,-8.0],Z,CubicInOut),(1.0,[-25.0,0.0,-8.0],Z,Hold)]),
    (Body(ForearmL), &[(0.0,[-65.0,0.0,0.0],Z,Hold)]),
    (Prop(Socket::OffhandL), &[(0.0,[90.0,0.0,8.0],Z,Hold)]),
    (Body(UpperArmR), &[(0.0,[18.0,0.0,-8.0],Z,CubicInOut),(0.5,[-22.0,0.0,-8.0],Z,CubicInOut),(1.0,[18.0,0.0,-8.0],Z,Hold)]),
];
fn locomotion(clip: Clip, seconds: f32, pose: &mut Pose) {
    let tracks = match clip {
        Clip::Idle => IDLE,
        Clip::Walk => WALK,
        _ => return,
    };
    let k = (seconds / clip.duration()).rem_euclid(1.0);
    for (node, keys) in tracks {
        pose[node.index()] = sample_track(keys, k);
    }
}
