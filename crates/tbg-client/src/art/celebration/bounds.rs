//! Cache-friendly mesh-box sampling through the actual celebration pose pipeline.

use crate::art::{anim::Animator, rig::NODE_COUNT, FigureForm};
use bevy::{mesh::VertexAttributeValues, prelude::*};
use tbg_core::UnitKind;

pub fn envelope(
    form: &FigureForm,
    meshes: &Assets<Mesh>,
    kind: UnitKind,
    tier: u8,
    branch: u8,
) -> (f32, f32) {
    let mut boxes = Vec::new();
    for layer in [&form.meshes, &form.glow] {
        for (i, h) in layer
            .iter()
            .enumerate()
            .filter_map(|(i, h)| h.as_ref().map(|h| (i, h)))
        {
            let Some(VertexAttributeValues::Float32x3(vertices)) = meshes
                .get(h)
                .and_then(|m| m.attribute(Mesh::ATTRIBUTE_POSITION))
            else {
                continue;
            };
            let (mut low, mut high) = (Vec3::splat(f32::INFINITY), Vec3::splat(f32::NEG_INFINITY));
            for v in vertices {
                low = low.min(Vec3::from(*v));
                high = high.max(Vec3::from(*v));
            }
            if low.is_finite() && high.is_finite() {
                boxes.push((i, low, high));
            }
        }
    }
    let mut animator = Animator::new(
        [Entity::PLACEHOLDER; NODE_COUNT],
        form.plan,
        Entity::PLACEHOLDER,
        kind,
        0,
    );
    animator.form = (tier, branch);
    animator.native = form.native;
    let motion = super::celebration_for(kind, tier, branch);
    let (mut radius, mut height) = (0.15f32, 0.3f32);
    for phase in 0..32 {
        let t = phase as f32 * 0.257;
        animator.celebrate(motion, t);
        let pose = animator.pose(motion.clip, t.rem_euclid(motion.duration));
        let world = crate::art::presentation::pose_world(kind, &form.plan, &pose);
        for (index, low, high) in &boxes {
            for x in [low.x, high.x] {
                for y in [low.y, high.y] {
                    for z in [low.z, high.z] {
                        let p =
                            world[*index].transform_point3(Vec3::new(x, y, z)) * form.plan.scale;
                        radius = radius.max(p.x.hypot(p.z));
                        height = height.max(p.y);
                    }
                }
            }
        }
    }
    // Positive padding covers between-sample arcs; cache is bounded by the content/form manifest.
    (radius * 1.2 + 0.12, height * 1.15 + 0.12)
}
