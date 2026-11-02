//! GD-04: Volthar and Astreon forms, grips held in every clip and a lance clear of the torso.
use super::*;
use std::collections::BTreeSet;

const CLIPS: [anim::Clip; 8] = [
    anim::Clip::Idle,
    anim::Clip::Walk,
    anim::Clip::AttackMelee,
    anim::Clip::AttackRanged,
    anim::Clip::Hit,
    anim::Clip::Spawn,
    anim::Clip::Evolve,
    anim::Clip::Death,
];

fn actor(kind: UnitKind, t: u8, b: u8) -> (anim::Animator, FigureForm) {
    let mut meshes = Assets::<Mesh>::default();
    let form = native_form((kits::kit(kind).unwrap().figure)(t, b), &mut meshes);
    let mut a = anim::Animator::new(
        [Entity::PLACEHOLDER; NODE_COUNT],
        form.plan,
        Entity::PLACEHOLDER,
        kind,
        1,
    );
    a.form = (t, b);
    (a, form)
}

/// Every sampled frame of every clip, including Victory, with and without an aim.
fn frames(kind: UnitKind, t: u8, b: u8, mut check: impl FnMut(anim::Clip, &[Mat4; NODE_COUNT])) {
    let (mut actor, form) = actor(kind, t, b);
    for clip in CLIPS.into_iter().chain([anim::Clip::Victory]) {
        for step in 0..24 {
            let time = step as f32 * actor.duration(clip) / 24.0;
            let pose = actor.pose(clip, time);
            check(clip, &presentation::pose_world(kind, &form.plan, &pose));
            for target in [Vec3::new(0.0, 0.3, 2.2), Vec3::new(0.9, 0.2, 1.6)] {
                let aimed = actor.clearance_pose(clip, time, Some(target), 0.30);
                check(clip, &presentation::pose_world(kind, &form.plan, &aimed));
            }
        }
    }
}

#[test]
fn gd04_new_gods_have_five_distinct_bounded_finite_forms() {
    for kind in [UnitKind::Volthar, UnitKind::Astreon] {
        let kit = kits::kit(kind).unwrap();
        let mut seen = BTreeSet::new();
        for (t, b) in FORMS {
            let spec = (kit.figure)(t, b);
            assert!(seen.insert(format!(
                "{:?}",
                spec.parts
                    .iter()
                    .map(|v| (v.node.index(), v.at, format!("{:?}", v.color)))
                    .collect::<Vec<_>>()
            )));
            let mut meshes = Assets::<Mesh>::default();
            let form = native_form(spec, &mut meshes);
            assert!(
                form.triangles <= 6000,
                "{kind:?} {t}/{b}: {}",
                form.triangles
            );
            frames(kind, t, b, |clip, world| {
                assert!(world.iter().all(|m| m.is_finite()), "{kind:?} {clip:?}");
            });
        }
    }
}

#[test]
fn gd04_volthar_right_hand_never_leaves_the_hammer_grip() {
    use rig::{Joint::HandR, Node::Body, Node::Prop, Socket::WeaponR};
    for (t, b) in FORMS {
        frames(UnitKind::Volthar, t, b, |clip, world| {
            let grip = world[Prop(WeaponR).index()].transform_point3(Vec3::ZERO);
            let wrist = world[Body(HandR).index()].transform_point3(Vec3::ZERO);
            assert!(
                grip.distance(wrist) < 0.04,
                "Volthar {t}/{b} {clip:?}: wrist {wrist:?} grip {grip:?}"
            );
        });
    }
}

#[test]
fn gd04_astreon_holds_the_lance_in_both_hands_and_its_shaft_stays_out_of_the_torso() {
    use rig::{
        Joint::{Chest, HandL, HandR},
        Node::Body,
        Node::Prop,
        Socket::WeaponR,
    };
    for (t, b) in FORMS {
        let plan = kits::astreon::plan(t);
        frames(UnitKind::Astreon, t, b, |clip, world| {
            let lance = world[Prop(WeaponR).index()];
            for (hand, anchor) in [(HandR, 0.0f32), (HandL, 0.20)] {
                let at = lance.transform_point3(Vec3::new(0.0, 0.0, anchor));
                let wrist = world[Body(hand).index()].transform_point3(Vec3::ZERO);
                assert!(
                    at.distance(wrist) < 0.05,
                    "Astreon {t}/{b} {clip:?} {hand:?}: wrist {wrist:?} anchor {at:?}"
                );
            }
            // Sample the shaft from butt to point inside the chest's own frame.
            let chest = world[Body(Chest).index()].inverse();
            for i in 0..=20 {
                let z = -0.78 + i as f32 * (1.30 / 20.0);
                let s = chest.transform_point3(lance.transform_point3(Vec3::new(0.0, 0.0, z)));
                let inside = s.x.abs() < plan.torso.x * 0.45
                    && (-0.02..plan.torso.y * 0.95).contains(&s.y)
                    && s.z.abs() < plan.torso.z * 0.45;
                assert!(
                    !inside,
                    "Astreon {t}/{b} {clip:?}: shaft point {z} at {s:?} is inside the torso"
                );
            }
        });
    }
}
