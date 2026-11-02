//! OG-01: actual native geometry, clip transforms and equipment clearance regressions.
use super::*;
use bevy::mesh::VertexAttributeValues;
use std::collections::BTreeSet;
#[test]
fn ogre_all_45_forms_are_distinct_finite_and_within_mesh_budget() {
    let mut count = 0;
    for kind in tbg_core::COMPLETION_POOL
        .into_iter()
        .filter(|k| kits::ogres::contains(*k))
    {
        let kit = kits::kit(kind).unwrap();
        let mut signatures = BTreeSet::new();
        for (t, b) in FORMS {
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
            assert!(signatures.insert(signature), "duplicate {kind:?} {t}/{b}");
            let mut meshes = Assets::<Mesh>::default();
            let form = native_form(spec, &mut meshes);
            assert!(form.triangles <= 6000, "{kind:?}: {}", form.triangles);
            let mut anim = anim::Animator::new(
                [Entity::PLACEHOLDER; NODE_COUNT],
                form.plan,
                Entity::PLACEHOLDER,
                kind,
                1,
            );
            anim.form = (t, b);
            for clip in [
                anim::Clip::Idle,
                anim::Clip::Walk,
                anim::Clip::AttackMelee,
                anim::Clip::AttackRanged,
                anim::Clip::Cast,
                anim::Clip::Hit,
                anim::Clip::Death,
                anim::Clip::Spawn,
                anim::Clip::Evolve,
                anim::Clip::Victory,
            ] {
                for sample in 0..12 {
                    let pose = anim.pose(clip, sample as f32 * kit.duration(clip) / 12.0);
                    for matrix in presentation::pose_world(kind, &form.plan, &pose) {
                        assert!(matrix.is_finite(), "{kind:?} {clip:?}");
                    }
                }
            }
            count += 1;
        }
    }
    assert_eq!(count, 45);
}
#[test]
fn ogre_bazooka_and_arms_fit_cached_clearance_in_every_form() {
    let kind = UnitKind::SiegeWalker;
    for (t, b) in FORMS {
        let mut meshes = Assets::<Mesh>::default();
        let form = native_form((kits::kit(kind).unwrap().figure)(t, b), &mut meshes);
        let (top, depth, width) = overhead::height(&form, &meshes, kind, t, b);
        assert!(top > 0.0);
        let mut anim = anim::Animator::new(
            [Entity::PLACEHOLDER; NODE_COUNT],
            form.plan,
            Entity::PLACEHOLDER,
            kind,
            1,
        );
        anim.form = (t, b);
        for clip in [anim::Clip::Idle, anim::Clip::Walk, anim::Clip::AttackRanged] {
            for sample in 0..31 {
                let target = [
                    None,
                    Some(Vec3::new(0.0, 0.6, 1.2)),
                    Some(Vec3::new(0.0, 0.6, 4.5)),
                ][sample % 3];
                let pose = anim.clearance_pose(
                    clip,
                    sample as f32 * anim.duration(clip) / 31.0,
                    target,
                    sample as f32 * 1.6 / 31.0,
                );
                let world = presentation::pose_world(kind, &form.plan, &pose);
                for node in rig::nodes() {
                    for layer in [&form.meshes, &form.glow] {
                        let Some(VertexAttributeValues::Float32x3(v)) = layer[node.index()]
                            .as_ref()
                            .and_then(|h| meshes.get(h))
                            .and_then(|m| m.attribute(Mesh::ATTRIBUTE_POSITION))
                        else {
                            continue;
                        };
                        for point in v {
                            let at = world[node.index()].transform_point3(Vec3::from(*point))
                                * form.plan.scale;
                            assert!(at.x.abs()<=width+0.001&&at.z>=depth.x-0.001&&at.z<=depth.y+0.001,"unreserved {node:?} {t}/{b} {clip:?}: {at:?}, {depth:?} width={width}");
                        }
                    }
                }
            }
        }
        for yaw in [-2.2, -1.1, 0.0, 1.1, 2.2] {
            let mut body = crate::crowd::Body {
                id: 1,
                anchor_x: 0.0,
                radius: 0.35,
                half_length: 0.35,
                facing: 1.0,
                ghost: false,
            };
            body.include_envelope(depth.x, depth.y, width, yaw);
            assert!(body.radius >= width * yaw.sin().abs());
            assert!(body.half_length >= body.radius);
        }
    }
}

#[test]
fn ogre_barrel_cask_and_hands_fit_cached_cast_clearance_in_all_forms() {
    let kind = UnitKind::GustDrummer;
    for (t, b) in FORMS {
        let mut meshes = Assets::<Mesh>::default();
        let form = native_form((kits::kit(kind).unwrap().figure)(t, b), &mut meshes);
        let (_, depth, width) = overhead::height(&form, &meshes, kind, t, b);
        let mut anim = anim::Animator::new(
            [Entity::PLACEHOLDER; NODE_COUNT],
            form.plan,
            Entity::PLACEHOLDER,
            kind,
            1,
        );
        anim.form = (t, b);
        for clip in [anim::Clip::Idle, anim::Clip::Walk, anim::Clip::Cast] {
            for step in 0..31 {
                let pose = anim.clearance_pose(
                    clip,
                    step as f32 * anim.duration(clip) / 31.0,
                    Some(Vec3::new(0.0, 0.6, 3.0)),
                    step as f32 * 1.6 / 31.0,
                );
                let world = presentation::pose_world(kind, &form.plan, &pose);
                for node in rig::nodes() {
                    for layer in [&form.meshes, &form.glow] {
                        let Some(VertexAttributeValues::Float32x3(vertices)) = layer[node.index()]
                            .as_ref()
                            .and_then(|h| meshes.get(h))
                            .and_then(|m| m.attribute(Mesh::ATTRIBUTE_POSITION))
                        else {
                            continue;
                        };
                        for point in vertices {
                            let at = world[node.index()].transform_point3(Vec3::from(*point))
                                * form.plan.scale;
                            assert!(
                                at.x.abs() <= width + 0.001
                                    && at.z >= depth.x - 0.001
                                    && at.z <= depth.y + 0.001,
                                "OG-02 unreserved cask/arm {t}/{b} {clip:?}: {at:?}"
                            );
                        }
                    }
                }
            }
        }
    }
}
