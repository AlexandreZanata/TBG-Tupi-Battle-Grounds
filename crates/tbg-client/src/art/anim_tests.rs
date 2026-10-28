//! P15 animation contracts: strike pose, walk event boundaries, and easing endpoints.

use super::{
    anim::{sample, Clip, ClipEvent, Easing},
    curves::{events_between, sample_track},
    rig::Joint,
};
use bevy::prelude::*;

#[test]
fn strike_time_is_the_strike_key_pose() {
    // ART-DIRECTION §4: melee lands at 300 ms, never at anticipation.
    let time = Clip::AttackMelee.events()[0].0;
    let pose = sample(Clip::AttackMelee, time);
    assert!((pose[Joint::UpperArmR.index()].0 - Vec3::new(55.0, 0.0, 0.0)).length() < 0.0001);
    assert_eq!(
        events_between(Clip::AttackMelee, 0.29, 0.31),
        [ClipEvent::Strike]
    );
}
#[test]
fn walk_delivers_two_footsteps_without_boundary_duplicates() {
    let cycle = Clip::Walk.duration();
    assert_eq!(
        events_between(Clip::Walk, 0.0, cycle),
        [ClipEvent::Footstep; 2]
    );
    assert_eq!(
        events_between(Clip::Walk, 0.0, cycle * 3.0),
        [ClipEvent::Footstep; 6]
    );
    assert_eq!(events_between(Clip::Walk, cycle, cycle + 0.01), []);
}
#[test]
fn all_easings_preserve_keys_and_hold_preserves_contact() {
    for ease in [
        Easing::Linear,
        Easing::CubicIn,
        Easing::CubicOut,
        Easing::CubicInOut,
        Easing::Hold,
    ] {
        let keys = [
            (0.0, [0.0; 3], [0.0; 3], ease),
            (1.0, [1.0; 3], [1.0; 3], ease),
        ];
        assert_eq!(sample_track(&keys, 0.0).0, Vec3::ZERO);
        assert_eq!(sample_track(&keys, 1.0).0, Vec3::ONE);
    }
    assert_eq!(Easing::Hold.sample(0.99), 0.0);
}

#[test]
fn planted_foot_holds_world_x_while_the_figure_moves() {
    use super::{
        feet::FootLocks,
        rig::{self, BodyPlan, Node},
    };
    let p = BodyPlan::Humanoid.proportions();
    let mut locks = FootLocks::default();
    let mut positions = Vec::new();
    for (phase, x) in [(0.1, 0.0), (0.15, 0.012)] {
        let frame =
            Mat4::from_rotation_translation(Quat::from_rotation_y(65f32.to_radians()), Vec3::X * x);
        let mut pose = sample(Clip::Walk, phase * Clip::Walk.duration());
        locks.apply(&mut pose, phase, frame, &p);
        let mut world = frame;
        for j in [
            Joint::Root,
            Joint::Hips,
            Joint::ThighL,
            Joint::ShinL,
            Joint::FootL,
        ] {
            let (r, offset) = pose[j.index()];
            let r = r * std::f32::consts::PI / 180.0;
            world *= Mat4::from_rotation_translation(
                Quat::from_euler(EulerRot::XYZ, r.x, r.y, r.z),
                rig::rest(Node::Body(j), &p) + offset,
            );
        }
        positions.push(world.transform_point3(Vec3::ZERO).x);
    }
    assert!(
        (positions[0] - positions[1]).abs() < 0.0001,
        "{positions:?}"
    );
}
