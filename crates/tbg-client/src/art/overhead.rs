//! Cached body envelopes for overhead UI, excluding long hand-held weapons.
use super::{
    anim::{Animator, Clip},
    rig::{Node, Socket, NODE_COUNT},
    FigureForm,
};
use bevy::{mesh::VertexAttributeValues, prelude::*};
use tbg_core::UnitKind;

pub fn height(
    form: &FigureForm,
    meshes: &Assets<Mesh>,
    kind: UnitKind,
    tier: u8,
    branch: u8,
) -> (f32, Vec2, f32) {
    let mut a = Animator::new(
        [Entity::PLACEHOLDER; NODE_COUNT],
        form.plan,
        Entity::PLACEHOLDER,
        kind,
        0,
    );
    a.form = (tier, branch);
    let mut top = 0.0f32;
    let mut radius = 0.0f32;
    let mut depth = Vec2::new(f32::INFINITY, f32::NEG_INFINITY);
    for clip in [Clip::Idle, Clip::Walk] {
        for step in 0..16 {
            let pose = a.pose(clip, step as f32 * clip.duration() / 16.0);
            let world = super::presentation::pose_world(kind, &form.plan, &pose);
            for node in super::rig::nodes() {
                if matches!(
                    node,
                    Node::Prop(
                        Socket::WeaponR | Socket::OffhandL | Socket::Nock | Socket::StringTop
                    )
                ) {
                    continue;
                }
                for layer in [&form.meshes, &form.glow] {
                    let Some(VertexAttributeValues::Float32x3(v)) = layer[node.index()]
                        .as_ref()
                        .and_then(|h| meshes.get(h))
                        .and_then(|m| m.attribute(Mesh::ATTRIBUTE_POSITION))
                    else {
                        continue;
                    };
                    for p in v {
                        let at =
                            world[node.index()].transform_point3(Vec3::from(*p)) * form.plan.scale;
                        top = top.max(at.y);
                        radius = radius.max(at.x.abs());
                        depth.x = depth.x.min(at.z);
                        depth.y = depth.y.max(at.z);
                    }
                }
            }
        }
    }
    // Include breathing/selection lift and the billboard's lower edge at the normal camera pitch.
    (top + 0.32, depth, radius)
}
