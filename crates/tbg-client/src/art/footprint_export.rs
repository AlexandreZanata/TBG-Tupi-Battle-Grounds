//! Debug authoring export of real native mesh/pose envelopes for PK-01 integer calibration.
use super::*;
use bevy::mesh::VertexAttributeValues;

pub fn write(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut rows = Vec::new();
    let selected = crate::config::url_param("footprint-kind");
    for kind in tbg_core::ALL_KINDS {
        if selected
            .as_ref()
            .is_some_and(|name| !format!("{kind:?}").eq_ignore_ascii_case(name))
        {
            continue;
        }
        let mut measured = [0.0f32; 4];
        for (tier, branch) in FORMS {
            let spec = kits::kit(kind).map_or_else(
                || roster::blockout::spec(kind),
                |k| (k.figure)(tier, branch),
            );
            let mut meshes = Assets::<Mesh>::default();
            let form = native_form(spec, &mut meshes);
            let mut animator = anim::Animator::new(
                [Entity::PLACEHOLDER; NODE_COUNT],
                form.plan,
                Entity::PLACEHOLDER,
                kind,
                1,
            );
            animator.form = (tier, branch);
            let action = match tbg_core::content::def(kind).archetype {
                tbg_core::Archetype::Ranged => anim::Clip::AttackRanged,
                tbg_core::Archetype::Support | tbg_core::Archetype::Control => anim::Clip::Cast,
                _ => anim::Clip::AttackMelee,
            };
            let targets = [
                None,
                Some(Vec3::new(0.0, 0.6, 1.2)),
                Some(Vec3::new(0.0, 0.6, 4.5)),
                Some(Vec3::new(-1.5, 0.6, 1.2)),
                Some(Vec3::new(1.5, 0.6, 1.2)),
                Some(Vec3::new(0.0, 0.6, -1.2)),
            ];
            for target in targets {
                for clip in [anim::Clip::Idle, anim::Clip::Walk, action] {
                    for step in 0..16 {
                        let pose = animator.clearance_pose(
                            clip,
                            step as f32 * animator.duration(clip) / 16.0,
                            target,
                            step as f32 * 1.6 / 16.0,
                        );
                        let world = presentation::pose_world(kind, &form.plan, &pose);
                        let normalize = form.plan.scale / (1.0 + 0.08 * f32::from(tier));
                        for node in rig::nodes() {
                            for layer in [&form.meshes, &form.glow] {
                                let Some(VertexAttributeValues::Float32x3(points)) = layer
                                    [node.index()]
                                .as_ref()
                                .and_then(|h| meshes.get(h))
                                .and_then(|m| m.attribute(Mesh::ATTRIBUTE_POSITION)) else {
                                    continue;
                                };
                                for point in points {
                                    let at = world[node.index()]
                                        .transform_point3(Vec3::from(*point))
                                        * normalize;
                                    if !at.is_finite() {
                                        return Err(format!("nonfinite envelope: {kind:?}").into());
                                    }
                                    measured[0] = measured[0].max(at.x.abs() * 2.0);
                                    measured[1] = measured[1].max(at.z);
                                    measured[2] = measured[2].max(-at.z);
                                    if matches!(node, rig::Node::Body(_)) {
                                        measured[3] = measured[3].max(at.z.abs());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        let milli = |v: f32| (v * 1000.0).ceil() as i32 + 100;
        rows.push(serde_json::json!({"kind":format!("{kind:?}"),"width":milli(measured[0]),"front":milli(measured[1]),"back":milli(measured[2]),"body":milli(measured[3]),"forms":5,"samples_per_clip":16,"aim_cases":6}));
    }
    std::fs::write(path, serde_json::to_string_pretty(&rows)? + "\n")?;
    println!("Exported {} native profiles to {path}", rows.len());
    Ok(())
}
