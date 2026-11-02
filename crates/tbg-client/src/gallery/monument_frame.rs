//! Fit the monumental lineup from its actual posed vertices, including the forward slam.
use crate::art::{
    anim::{Animator, Clip},
    presentation,
    rig::{self, NODE_COUNT},
    FigureLibrary, FORMS,
};
use bevy::{mesh::VertexAttributeValues, prelude::*};
use tbg_core::UnitKind;

pub(super) fn bounds(kind: UnitKind, lib: &FigureLibrary, meshes: &Assets<Mesh>) -> (Vec3, Vec3) {
    let direction = Vec3::new(0., 1.2, 10.).normalize();
    let right = Vec3::Y.cross(direction).normalize();
    let up = direction.cross(right);
    let basis = Mat3::from_cols(right, up, direction);
    let mut lo = Vec3::splat(f32::INFINITY);
    let mut hi = Vec3::splat(f32::NEG_INFINITY);
    for (col, (tier, branch)) in FORMS.into_iter().enumerate() {
        let form = lib.get(kind, tier, branch);
        let mut actor = Animator::new(
            [Entity::PLACEHOLDER; NODE_COUNT],
            form.plan,
            Entity::PLACEHOLDER,
            kind,
            1,
        );
        actor.native = form.native;
        actor.form = (tier, branch);
        for (row, (clip, time)) in [
            (Clip::Idle, 0.4),
            (Clip::Walk, 0.15),
            super::lineup::attack(kind),
        ]
        .into_iter()
        .enumerate()
        {
            let root = Mat4::from_scale_rotation_translation(
                Vec3::splat(form.plan.scale),
                Quat::from_rotation_y(super::figures::yaw(kind).to_radians()),
                super::layout::Layout::Monument(row, col).position(false),
            );
            let pose = presentation::pose_world(kind, &form.plan, &actor.pose(clip, time));
            for node in rig::nodes() {
                let matrix = root * pose[node.index()];
                for layer in [&form.meshes, &form.glow] {
                    let Some(VertexAttributeValues::Float32x3(points)) = layer[node.index()]
                        .as_ref()
                        .and_then(|h| meshes.get(h))
                        .and_then(|m| m.attribute(Mesh::ATTRIBUTE_POSITION))
                    else {
                        continue;
                    };
                    for point in points {
                        let projected =
                            basis.transpose() * matrix.transform_point3(Vec3::from(*point));
                        lo = lo.min(projected);
                        hi = hi.max(projected);
                    }
                }
            }
        }
    }
    (basis * ((lo + hi) * 0.5), (hi - lo) * 0.5)
}

pub(super) fn target((center, half): (Vec3, Vec3), aspect: f32) -> (Vec3, f32, f32, Vec3) {
    // Add the near-depth term before dividing by the existing conservative FOV.
    let width = half.x * 1.12 + half.z * 0.315 * aspect;
    let height = half.y * 1.18 + half.z * 0.315 + 1.0;
    (center, width, height, Vec3::new(0., 1.2, 10.))
}
