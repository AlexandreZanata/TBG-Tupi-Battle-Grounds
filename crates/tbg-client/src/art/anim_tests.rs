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

/// World position of a foot after forward kinematics through the rig.
fn foot_world(
    pose: &super::anim::Pose,
    frame: Mat4,
    foot: [Joint; 5],
    p: &super::rig::Proportions,
) -> Vec3 {
    let mut world = frame;
    for j in foot {
        let (r, offset) = pose[j.index()];
        world *= Mat4::from_rotation_translation(
            super::ik::quat(r),
            super::rig::rest(super::rig::Node::Body(j), p) + offset,
        );
    }
    world.transform_point3(Vec3::ZERO)
}

#[test]
fn planted_feet_hold_world_x_and_z_at_any_speed_and_heading() {
    use super::{
        feet::{FootLocks, Stride},
        kits::Gait,
        rig::BodyPlan,
    };
    // ART-DIRECTION §4: feet planted while in contact, no foot sliding.
    for (gait, plan) in [
        (Gait::LIGHT, BodyPlan::Humanoid.proportions()),
        (Gait::HEAVY, BodyPlan::Large.proportions()),
    ] {
        for yaw in [65f32, -65.0, 90.0] {
            let rot = Quat::from_rotation_y(yaw.to_radians());
            let dir = (rot.inverse() * Vec3::X).with_y(0.0).normalize();
            let stride = gait.cycle(plan.leg) * plan.scale;
            for (side, legs, start) in [
                (
                    0,
                    [
                        Joint::Root,
                        Joint::Hips,
                        Joint::ThighL,
                        Joint::ShinL,
                        Joint::FootL,
                    ],
                    0.05,
                ),
                (
                    1,
                    [
                        Joint::Root,
                        Joint::Hips,
                        Joint::ThighR,
                        Joint::ShinR,
                        Joint::FootR,
                    ],
                    0.55,
                ),
            ] {
                let _ = side;
                let mut locks = FootLocks::default();
                let mut feet = Vec::new();
                for step in 0..8 {
                    let phase = start + step as f32 * 0.05;
                    let frame = Mat4::from_scale_rotation_translation(
                        Vec3::splat(plan.scale),
                        rot,
                        Vec3::X * phase * stride,
                    );
                    let mut pose = sample(Clip::Walk, phase * Clip::Walk.duration());
                    let s = Stride {
                        phase,
                        dir,
                        lock: true,
                    };
                    locks.apply(&mut pose, &s, frame, &plan, &gait);
                    feet.push(foot_world(&pose, frame, legs, &plan));
                }
                for f in &feet {
                    let drift = (f.xz() - feet[0].xz()).length();
                    assert!(
                        drift < 0.001,
                        "yaw {yaw}: planted foot drifts {drift} ({feet:?})"
                    );
                }
            }
        }
    }
}

#[test]
fn unlocked_feet_follow_the_stride_path_and_stay_reachable() {
    use super::{
        feet::{FootLocks, Stride},
        kits::Gait,
        rig::BodyPlan,
    };
    let plan = BodyPlan::Large.proportions();
    let mut locks = FootLocks::default();
    for step in 0..20 {
        let phase = step as f32 * 0.05;
        let mut pose = sample(Clip::Walk, 0.0);
        let s = Stride {
            phase,
            dir: Vec3::Z,
            lock: false,
        };
        locks.apply(&mut pose, &s, Mat4::IDENTITY, &plan, &Gait::HEAVY);
        let foot = foot_world(
            &pose,
            Mat4::IDENTITY,
            [
                Joint::Root,
                Joint::Hips,
                Joint::ThighL,
                Joint::ShinL,
                Joint::FootL,
            ],
            &plan,
        );
        assert!(
            foot.y > 0.055,
            "foot sinks below the ground at {phase}: {foot:?}"
        );
        if phase < 0.5 {
            let along = Gait::HEAVY.cycle(plan.leg) * (0.25 - phase);
            assert!(
                (foot.z - along).abs() < 0.002,
                "contact path at {phase}: {foot:?}"
            );
        }
    }
}

#[test]
fn ranged_release_lands_at_0_18() {
    // The archer's arrow leaves the string on this event (`archer::RELEASE`).
    assert_eq!(
        events_between(Clip::AttackRanged, 0.17, 0.18),
        [ClipEvent::Release]
    );
}

#[test]
fn adapted_legs_ride_whole_segments_so_knees_bend_cleanly() {
    use super::{adapter, roster};
    use tbg_core::UnitKind;
    // Gait v2 bends every knee; a leg mesh cut by triangles would tear.
    for kind in [
        UnitKind::Shieldbearer,
        UnitKind::Hexer,
        UnitKind::Sawbot,
        UnitKind::Bloomkeeper,
    ] {
        let built = adapter::build(&roster::figure_spec(kind, 0, 0));
        for j in [
            Joint::ThighL,
            Joint::ShinL,
            Joint::FootL,
            Joint::ThighR,
            Joint::ShinR,
            Joint::FootR,
        ] {
            assert!(built[j.index()].is_some(), "{kind:?} has no {j:?} segment");
        }
    }
}

#[test]
fn test_attack_clips_finish_before_unit_intervals_and_events_on_time() {
    use tbg_core::{content, ALL_KINDS};
    for &kind in &ALL_KINDS {
        let stats = content::effective_stats(kind, 0, 0, 0);
        let interval_s = stats.interval as f32 / content::TICK_HZ as f32;
        assert!(
            Clip::AttackMelee.duration() < interval_s,
            "{kind:?} melee {}s >= interval {}s",
            Clip::AttackMelee.duration(),
            interval_s
        );
        assert!(
            Clip::AttackRanged.duration() < interval_s,
            "{kind:?} ranged {}s >= interval {}s",
            Clip::AttackRanged.duration(),
            interval_s
        );
        assert!(
            Clip::Cast.duration() < interval_s,
            "{kind:?} cast {}s >= interval {}s",
            Clip::Cast.duration(),
            interval_s
        );
    }

    // Strike at 0.30s +- 1 frame (1/60s = 0.0167s)
    let strike_events = events_between(Clip::AttackMelee, 0.29, 0.31);
    assert_eq!(strike_events, [ClipEvent::Strike]);

    // Release at 0.18s +- 1 frame
    let release_events = events_between(Clip::AttackRanged, 0.17, 0.19);
    assert_eq!(release_events, [ClipEvent::Release]);
}

#[test]
fn test_corpse_schedule_duration() {
    let corpse_schedule = Clip::Death.duration() + crate::visuals::CORPSE_SECS;
    assert_eq!(corpse_schedule, 1.50);
}
