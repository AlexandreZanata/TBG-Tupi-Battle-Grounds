//! Geometry, motion envelopes and crowd-facing regression evidence for the seventeen new kits.
use super::*;
use crate::art::{
    anim::{Animator, Clip},
    figure, mesh, presentation,
    rig::NODE_COUNT,
    FORMS,
};
use bevy::mesh::VertexAttributeValues;
use std::collections::BTreeSet;
fn kinds() -> impl Iterator<Item = UnitKind> {
    tbg_core::COMPLETION_POOL
        .into_iter()
        .filter(|k| kit(*k).is_some())
}
#[test]
fn completion_all_85_forms_have_distinct_geometry_bounded_cost_and_finite_life_motion() {
    let mut count = 0;
    let mut max_tris = 0;
    for k in kinds() {
        let mut signatures = BTreeSet::new();
        for (t, b) in FORMS {
            let kit = kit(k).expect("new kit");
            let spec = (kit.figure)(t, b);
            let mut signature = spec
                .parts
                .iter()
                .map(|p| {
                    format!(
                        "{:?}/{:?}/{:?}/{:?}",
                        p.node.index(),
                        p.shape,
                        p.at,
                        p.color
                    )
                })
                .collect::<Vec<_>>();
            signature.sort();
            assert!(signatures.insert(signature), "duplicate form {k:?} {t}/{b}");
            let mut vertices = Vec::new();
            let mut tris = 0;
            for (node, m) in figure::build_joint_meshes(&spec)
                .into_iter()
                .enumerate()
                .chain(figure::build_glow_meshes(&spec).into_iter().enumerate())
            {
                if let Some(m) = m {
                    tris += mesh::triangles(&m);
                    if let Some(VertexAttributeValues::Float32x3(v)) =
                        m.attribute(Mesh::ATTRIBUTE_POSITION)
                    {
                        vertices.extend(v.iter().map(|p| (node, Vec3::from(*p))));
                    }
                }
            }
            assert!(tris <= 6000, "{k:?} {t}/{b}: {tris} triangles");
            max_tris = max_tris.max(tris);
            let mut animator = Animator::new(
                [Entity::PLACEHOLDER; NODE_COUNT],
                spec.plan,
                Entity::PLACEHOLDER,
                k,
                1,
            );
            animator.form = (t, b);
            let mut size = Vec3::ZERO;
            let mut lateral = 0.0f32;
            for clip in [
                Clip::Idle,
                Clip::Walk,
                Clip::AttackMelee,
                Clip::AttackRanged,
                Clip::Cast,
                Clip::Spawn,
                Clip::Evolve,
                Clip::Hit,
                Clip::Death,
                Clip::Victory,
            ] {
                for i in 0..8 {
                    let time = animator.duration(clip) * i as f32 / 8.0;
                    animator.present(clip, time);
                    let pose = animator.pose(clip, time);
                    assert!(pose.iter().all(|(r, p)| r.is_finite() && p.is_finite()));
                    let world = presentation::pose_world(k, &spec.plan, &pose);
                    for (node, v) in &vertices {
                        let p = world[*node].transform_point3(*v) * spec.plan.scale;
                        assert!(p.is_finite());
                        size = size.max(p.abs());
                        if matches!(clip, Clip::Idle | Clip::Walk) {
                            lateral = lateral.max(p.x.abs());
                        }
                    }
                }
            }
            assert!(
                size.max_element() < 4.0,
                "{k:?} {t}/{b} escaped its preview envelope: {size:?}"
            );
            println!("completion {k:?} {t}/{b}: {tris} triangles, sampled half-envelope {size:?}");
            assert!(
                lateral <= crowd_radius(k, t).expect("measured profile"),
                "{k:?} {t}/{b}: resting figure escaped its crowd footprint"
            );
            println!("footprint {k:?} {t}/{b}: {lateral}");
            count += 1;
        }
    }
    assert_eq!(count, 85);
    println!("completion: {count} forms; maximum {max_tris} triangles");
}
#[test]
fn completion_additional_insect_feet_hold_world_contact_and_use_only_fixed_rig_nodes() {
    use crate::art::{
        feet::{FootLocks, Stride},
        ik,
        rig::{
            Joint,
            Node::{Prop, Secondary},
            Socket, Spring,
        },
    };
    for k in kinds().filter(|k| is_insect(*k)) {
        let p = plan(k, 2);
        let gait = kit(k).expect("insect kit").gait;
        let mut locks = FootLocks::default();
        let mut previous = None;
        for i in 0..5 {
            let phase = 0.05 + i as f32 * 0.03;
            let mut pose = [(Vec3::ZERO, Vec3::ZERO); NODE_COUNT];
            let stride = Stride {
                phase,
                dir: Vec3::Z,
                lock: true,
            };
            let frame = Mat4::from_translation(Vec3::Z * gait.cycle(p.leg) * phase * p.scale)
                * Mat4::from_scale(Vec3::splat(p.scale));
            locks.apply(&mut pose, &stride, frame, &p, &gait);
            locks.apply_auxiliary(&mut pose, &stride, frame, &p, &gait);
            let world = presentation::pose_world(k, &p, &pose);
            let point = frame
                .transform_point3(world[Prop(Socket::Nock).index()].transform_point3(Vec3::ZERO));
            if let Some(last) = previous {
                assert!(
                    point.distance(last) < 0.0001,
                    "{k:?} auxiliary stance slid: {point:?} / {last:?}"
                );
            }
            previous = Some(point);
            assert!(pose[Secondary(Spring::Tabard).index()].0.is_finite());
            let _ = (Joint::Root, ik::quat(Vec3::ZERO));
        }
    }
}

#[test]
fn completion_two_hand_grips_stay_on_real_weapon_shafts_and_lantern_rims() {
    use crate::art::rig::{Joint, Node, Socket};
    for kind in [
        UnitKind::Sunlancer,
        UnitKind::Starbreaker,
        UnitKind::Dawnmender,
    ] {
        for (t, b) in FORMS {
            let kit = kit(kind).expect("kit");
            let spec = (kit.figure)(t, b);
            let mut animator = Animator::new(
                [Entity::PLACEHOLDER; NODE_COUNT],
                spec.plan,
                Entity::PLACEHOLDER,
                kind,
                1,
            );
            animator.form = (t, b);
            for clip in [
                Clip::Idle,
                Clip::Walk,
                Clip::AttackMelee,
                Clip::Cast,
                Clip::Spawn,
                Clip::Evolve,
                Clip::Victory,
            ] {
                for i in 0..16 {
                    let pose = animator.pose(clip, animator.duration(clip) * i as f32 / 16.0);
                    let world = presentation::pose_world(kind, &spec.plan, &pose);
                    let socket = world[Node::Prop(Socket::WeaponR).index()];
                    let left = socket.inverse().transform_point3(
                        world[Node::Body(Joint::HandL).index()].transform_point3(Vec3::ZERO),
                    );
                    if kind == UnitKind::Dawnmender {
                        let (center, r) = spec
                            .parts
                            .iter()
                            .find_map(|part| match part.shape {
                                crate::art::mesh::Shape::Ring(_, r, 360.0)
                                    if part.node.index() == Node::Prop(Socket::WeaponR).index() =>
                                {
                                    Some((part.at, r))
                                }
                                _ => None,
                            })
                            .expect("lantern rim");
                        assert!(
                            (left.distance(center) - r).abs() < 0.04,
                            "{kind:?} {t}/{b} {clip:?}: {left:?} escaped rim"
                        );
                    } else {
                        assert!(
                            left.x.abs() < 0.025
                                && left.y.abs() < 0.025
                                && (0.07..=0.30).contains(&left.z),
                            "{kind:?} {t}/{b} {clip:?}: offhand detached from shaft {left:?}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn completion_victory_loops_close_without_pose_jumps() {
    for kind in kinds() {
        for (t, b) in FORMS {
            let kit = kit(kind).expect("kit");
            let spec = (kit.figure)(t, b);
            let mut actor = Animator::new(
                [Entity::PLACEHOLDER; NODE_COUNT],
                spec.plan,
                Entity::PLACEHOLDER,
                kind,
                1,
            );
            actor.form = (t, b);
            assert_eq!(actor.duration(Clip::Victory), 3.0);
            let first = actor.pose(Clip::Victory, 0.00001);
            let last = actor.pose(Clip::Victory, 2.99999);
            for (a, b) in first.into_iter().zip(last) {
                assert!(
                    a.0.distance(b.0) < 0.05 && a.1.distance(b.1) < 0.001,
                    "{kind:?}: celebration seam jumps {a:?}/{b:?}"
                );
            }
        }
    }
}
