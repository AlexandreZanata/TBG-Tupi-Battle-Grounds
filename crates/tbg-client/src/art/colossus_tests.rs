//! GD-01: native Colossus forms, finite life/action poses and bounded geometry.
use super::*;
use std::collections::BTreeSet;
#[test]
fn gd01_colossus_has_five_distinct_finite_forms_with_one_obelisk() {
    let kit = kits::kit(UnitKind::Colossus).unwrap();
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
        assert!(
            spec.parts
                .iter()
                .filter(|v| matches!(v.node, rig::Node::Prop(rig::Socket::WeaponR)))
                .count()
                > 10
        );
        assert!(!spec
            .parts
            .iter()
            .any(|v| matches!(v.node, rig::Node::Prop(rig::Socket::Back))));
        let mut meshes = Assets::<Mesh>::default();
        let form = native_form(spec, &mut meshes);
        assert!(form.triangles <= 6000);
        let mut actor = anim::Animator::new(
            [Entity::PLACEHOLDER; NODE_COUNT],
            form.plan,
            Entity::PLACEHOLDER,
            UnitKind::Colossus,
            1,
        );
        actor.form = (t, b);
        for clip in [
            anim::Clip::Idle,
            anim::Clip::Walk,
            anim::Clip::AttackMelee,
            anim::Clip::Spawn,
            anim::Clip::Evolve,
            anim::Clip::Death,
            anim::Clip::Victory,
        ] {
            for step in 0..16 {
                let pose = actor.pose(clip, step as f32 * actor.duration(clip) / 16.0);
                assert!(
                    presentation::pose_world(UnitKind::Colossus, &form.plan, &pose)
                        .iter()
                        .all(|m| m.is_finite())
                );
            }
        }
    }
}

#[test]
fn gd02_colossus_arm_and_hand_vertices_fit_live_clearance_in_every_form() {
    use bevy::mesh::VertexAttributeValues;
    use rig::{Joint::*, Node};

    let kind = UnitKind::Colossus;
    assert!(overhead::reserves_equipment(kind));
    for (tier, branch) in FORMS {
        let mut meshes = Assets::<Mesh>::default();
        let form = native_form((kits::kit(kind).unwrap().figure)(tier, branch), &mut meshes);
        let (_, depth, width) = overhead::height(&form, &meshes, kind, tier, branch);
        let mut actor = anim::Animator::new(
            [Entity::PLACEHOLDER; NODE_COUNT],
            form.plan,
            Entity::PLACEHOLDER,
            kind,
            1,
        );
        actor.form = (tier, branch);
        for clip in [anim::Clip::Idle, anim::Clip::Walk, anim::Clip::AttackMelee] {
            for target in [
                None,
                Some(Vec3::new(0., 0.6, 1.2)),
                Some(Vec3::new(0., 0.6, 4.5)),
            ] {
                for step in 0..=32 {
                    let pose = actor.clearance_pose(
                        clip,
                        step as f32 * actor.duration(clip) / 32.,
                        target,
                        step as f32 * 1.6 / 32.,
                    );
                    let world = presentation::pose_world(kind, &form.plan, &pose);
                    for yaw in [
                        -2.2,
                        -std::f32::consts::FRAC_PI_2,
                        0.,
                        std::f32::consts::FRAC_PI_2,
                        2.2,
                    ] {
                        let body = colossus_clearance(depth, width, yaw);
                        for joint in [UpperArmL, ForearmL, HandL, UpperArmR, ForearmR, HandR] {
                            let node = Node::Body(joint);
                            for layer in [&form.meshes, &form.glow] {
                                let Some(VertexAttributeValues::Float32x3(vertices)) = layer
                                    [node.index()]
                                .as_ref()
                                .and_then(|h| meshes.get(h))
                                .and_then(|m| m.attribute(Mesh::ATTRIBUTE_POSITION)) else {
                                    continue;
                                };
                                for point in vertices {
                                    let local = world[node.index()]
                                        .transform_point3(Vec3::from(*point))
                                        * form.plan.scale;
                                    let at = Quat::from_rotation_y(yaw) * local;
                                    assert!((at.x - body.anchor_x).abs() <= body.half_length + 0.001 && at.z.abs() <= body.radius + 0.001,
                                        "GD-02 unreserved {joint:?}, {tier}/{branch}, {clip:?}, yaw={yaw}: {at:?}");
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn colossus_clearance(depth: Vec2, width: f32, yaw: f32) -> crate::crowd::Body {
    let mut body = crate::crowd::Body {
        id: 1,
        anchor_x: 0.,
        radius: 0.5,
        half_length: 0.5,
        facing: if yaw < 0. { -1. } else { 1. },
        ghost: false,
    };
    body.include_envelope(depth.x, depth.y, width, yaw);
    body
}

#[test]
fn gd02_colossus_clearance_pushes_a_neighbor_with_the_original_solver() {
    use crate::crowd::{Body, Offset, Params, Solver};
    for (tier, branch) in FORMS {
        let mut meshes = Assets::<Mesh>::default();
        let form = native_form(
            (kits::kit(UnitKind::Colossus).unwrap().figure)(tier, branch),
            &mut meshes,
        );
        let (_, depth, width) = overhead::height(&form, &meshes, UnitKind::Colossus, tier, branch);
        for yaw in [-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2] {
            let giant = colossus_clearance(depth, width, yaw);
            let neighbor = Body {
                id: 2,
                anchor_x: 0.,
                radius: 0.35,
                half_length: 0.35,
                facing: giant.facing,
                ghost: false,
            };
            let bodies = [giant, neighbor];
            let mut offsets = [Offset::default(); 2];
            let mut solver = Solver::default();
            for _ in 0..240 {
                solver.solve(&bodies, &mut offsets, 1. / 60., &Params::default());
            }
            let min = giant.radius + neighbor.radius;
            let x = ((neighbor.anchor_x + offsets[1].dx) - (giant.anchor_x + offsets[0].dx)) * min
                / (giant.half_length + neighbor.half_length).max(min);
            let z = offsets[1].dz - offsets[0].dz;
            assert!(
                x * x + z * z >= (min * 0.9).powi(2),
                "GD-02 neighbor inside Colossus clearance, {tier}/{branch}, yaw={yaw}"
            );
        }
    }
}

#[test]
fn gd03_colossus_turns_body_and_aligns_obelisk_tip_with_target_at_strike() {
    use rig::{Joint::Root, Node::Prop, Socket::WeaponR};
    for (tier, branch) in FORMS {
        let mut meshes = Assets::<Mesh>::default();
        let form = native_form(
            (kits::kit(UnitKind::Colossus).unwrap().figure)(tier, branch),
            &mut meshes,
        );
        let mut actor = anim::Animator::new(
            [Entity::PLACEHOLDER; NODE_COUNT],
            form.plan,
            Entity::PLACEHOLDER,
            UnitKind::Colossus,
            1,
        );
        actor.form = (tier, branch);
        for target in [
            Vec3::new(0.0, 0.18, 1.35),
            Vec3::new(0.85, 0.18, 1.10),
            Vec3::new(-0.90, 0.22, 1.15),
            Vec3::new(1.10, 0.32, 0.75),
            Vec3::new(-0.65, 0.15, -1.10),
        ] {
            let pose = actor.clearance_pose(anim::Clip::AttackMelee, 0.30, Some(target), 0.30);
            let expected_yaw = target.x.atan2(target.z).to_degrees();
            assert!(
                (pose[Root.index()].0.y - expected_yaw).abs() < 1e-3,
                "Colossus body did not turn toward target in form {tier}/{branch}: got {}, expected {expected_yaw}",
                pose[Root.index()].0.y
            );
            let world = presentation::pose_world(UnitKind::Colossus, &form.plan, &pose);
            let weapon = world[Prop(WeaponR).index()];
            let origin = weapon.transform_point3(Vec3::ZERO);
            let tip = weapon.transform_point3(Vec3::new(0.0, 1.27, 0.0));
            let axis_xz = (tip.xz() - origin.xz()).normalize();
            let to_xz = (target.xz() - origin.xz()).normalize();
            assert!(
                axis_xz.perp_dot(to_xz).abs() < 1e-4 && axis_xz.dot(to_xz) > 0.999,
                "Colossus obelisk horizontal line missed target {target:?} in form {tier}/{branch}: origin={origin:?}, tip={tip:?}"
            );
            let ray = (tip - origin).normalize();
            let to_target = (target - origin).normalize();
            assert!(
                ray.angle_between(to_target) < 1e-3,
                "Colossus obelisk 3D ray missed target {target:?} in form {tier}/{branch}: angle={}",
                ray.angle_between(to_target)
            );
            assert!(
                tip.distance(target) < 0.05,
                "Colossus obelisk tip did not reach target {target:?} in form {tier}/{branch}: tip={tip:?}, dist={}",
                tip.distance(target)
            );
        }
    }
}
