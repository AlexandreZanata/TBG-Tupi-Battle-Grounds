//! Merged mesh cache and spawning for rigid v2 figures, secondary joints, and sockets.

use super::{
    mesh::{self, Shape},
    rig::{self, Node, Proportions, NODE_COUNT},
    springs::SpringJoint,
};
use bevy::prelude::*;

#[derive(Clone, Copy)]
pub struct PartSpec {
    pub node: Node,
    pub shape: Shape,
    pub color: Color,
    pub at: Vec3,
    pub rot: Vec3,
    pub scale: Vec3,
}
pub fn part(node: Node, shape: Shape, color: Color, at: [f32; 3]) -> PartSpec {
    PartSpec {
        node,
        shape,
        color,
        at: at.into(),
        rot: Vec3::ZERO,
        scale: Vec3::ONE,
    }
}
impl PartSpec {
    pub fn rot(mut self, degrees: [f32; 3]) -> Self {
        self.rot = degrees.into();
        self
    }
    pub fn scale(mut self, scale: [f32; 3]) -> Self {
        self.scale = scale.into();
        self
    }
}
pub struct FigureSpec {
    pub plan: Proportions,
    pub parts: Vec<PartSpec>,
}

pub fn build_joint_meshes(spec: &FigureSpec) -> [Option<Mesh>; NODE_COUNT] {
    let mut parts: [Vec<(Mesh, Transform)>; NODE_COUNT] = Default::default();
    for p in &spec.parts {
        let r = p.rot * std::f32::consts::PI / 180.0;
        let t = Transform::from_translation(p.at)
            .with_rotation(Quat::from_euler(EulerRot::XYZ, r.x, r.y, r.z))
            .with_scale(p.scale);
        parts[p.node.index()].push((mesh::paint(mesh::build(p.shape), p.color), t));
    }
    parts.map(mesh::merge)
}
pub fn spawn_joints(
    commands: &mut Commands,
    root: Entity,
    plan: &Proportions,
    meshes: &[Option<Handle<Mesh>>; NODE_COUNT],
    material: &Handle<StandardMaterial>,
) -> [Entity; NODE_COUNT] {
    let mut ids = [root; NODE_COUNT];
    for node in rig::nodes() {
        let parent = node.parent().map_or(root, |p| ids[p.index()]);
        let mut e = commands.spawn((
            Transform::from_translation(rig::rest(node, plan)),
            Visibility::default(),
        ));
        if let Some(m) = &meshes[node.index()] {
            e.insert((Mesh3d(m.clone()), MeshMaterial3d(material.clone())));
        }
        if let Node::Secondary(s) = node {
            let back = Node::Prop(rig::Socket::Back).index();
            let used = meshes[node.index()].is_some()
                || (matches!(s, rig::Spring::Back) && meshes[back].is_some());
            if used {
                e.insert(SpringJoint::new(parent));
            }
        }
        let id = e.id();
        commands.entity(parent).add_child(id);
        ids[node.index()] = id;
    }
    ids
}
