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

#[test]
fn archer_budget_and_release_anchor_obey_p15() {
    use super::{archer_v2_clips, bow::BowAssets, figure, mesh, roster};
    let spec = roster::archer_v2(0);
    let triangles: usize = figure::build_joint_meshes(&spec)
        .iter()
        .flatten()
        .map(mesh::triangles)
        .sum();
    let bow = BowAssets::new(&mut Assets::<Mesh>::default());
    assert!(
        triangles + bow.triangles <= 1500,
        "{} triangles",
        triangles + bow.triangles
    );
    let mut pose = sample(Clip::AttackRanged, 0.18);
    archer_v2_clips::apply(Clip::AttackRanged, 0.18, &mut pose);
    let p = super::archer_v2::plan();
    let wrist = |joints: [Joint; 3]| {
        let mut transform = Mat4::IDENTITY;
        for j in joints {
            let r = pose[j.index()].0 * std::f32::consts::PI / 180.0;
            transform *= Mat4::from_rotation_translation(
                Quat::from_euler(EulerRot::XYZ, r.x, r.y, r.z),
                super::rig::rest(super::rig::Node::Body(j), &p),
            );
        }
        transform.transform_point3(Vec3::ZERO)
    };
    let left = wrist([Joint::UpperArmL, Joint::ForearmL, Joint::HandL]);
    let right = wrist([Joint::UpperArmR, Joint::ForearmR, Joint::HandR]);
    let anchor = left
        + Vec3::new(
            -super::archer_v2::STRING_LINE - super::archer_aim::draw(0.18) * super::archer_v2::DRAW,
            0.08,
            0.0,
        );
    assert!(
        (anchor - right).length() < 0.005,
        "anchor {anchor:?}, right {right:?}"
    );
    assert_eq!(
        events_between(Clip::AttackRanged, 0.17, 0.18),
        [ClipEvent::Release]
    );
    println!(
        "Archer geometry: {triangles} + {} bow/arrow = {} triangles",
        bow.triangles,
        triangles + bow.triangles
    );
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
