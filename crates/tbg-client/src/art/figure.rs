//! Merged mesh cache and spawning for rigid v2 figures, secondary joints,
//! sockets, and the unlit glow layer (eyes, furnaces, runes, spores).

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
    /// Drawn unlit in the glow layer instead of the lit figure mesh.
    pub glow: bool,
}
pub fn part(node: Node, shape: Shape, color: Color, at: [f32; 3]) -> PartSpec {
    PartSpec {
        node,
        shape,
        color,
        at: at.into(),
        rot: Vec3::ZERO,
        scale: Vec3::ONE,
        glow: false,
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
    pub fn glow(mut self) -> Self {
        self.glow = true;
        self
    }
}
pub struct FigureSpec {
    pub plan: Proportions,
    pub parts: Vec<PartSpec>,
}

/// The lit figure: every part not in the glow layer, merged per joint.
pub fn build_joint_meshes(spec: &FigureSpec) -> [Option<Mesh>; NODE_COUNT] {
    build_layer(spec, false)
}

/// The glow layer: parts marked `.glow()`, merged per joint.
pub fn build_glow_meshes(spec: &FigureSpec) -> [Option<Mesh>; NODE_COUNT] {
    build_layer(spec, true)
}

fn build_layer(spec: &FigureSpec, glow: bool) -> [Option<Mesh>; NODE_COUNT] {
    let mut parts: [Vec<(Mesh, Transform)>; NODE_COUNT] = Default::default();
    for p in spec.parts.iter().filter(|p| p.glow == glow) {
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
    kind: tbg_core::UnitKind,
    plan: &Proportions,
    meshes: &[Option<Handle<Mesh>>; NODE_COUNT],
    material: &Handle<StandardMaterial>,
) -> [Entity; NODE_COUNT] {
    let anatomy = super::kits::kit(kind).and_then(|kit| kit.anatomy);
    let mut ids = [root; NODE_COUNT];
    for node in rig::nodes() {
        let parent = anatomy
            .map_or_else(|| node.parent(), |a| (a.parent)(node))
            .map_or(root, |p| ids[p.index()]);
        let mut e = commands.spawn((
            Transform::from_translation(
                anatomy.map_or_else(|| rig::rest(node, plan), |a| (a.rest)(node, plan)),
            ),
            Visibility::default(),
        ));
        if let Some(m) = &meshes[node.index()] {
            e.insert((Mesh3d(m.clone()), MeshMaterial3d(material.clone())));
        }
        if let Node::Secondary(s) = node {
            let back = Node::Prop(rig::Socket::Back).index();
            let used = meshes[node.index()].is_some()
                || (matches!(s, rig::Spring::Back) && meshes[back].is_some());
            if used && anatomy.is_none_or(|a| a.springs) {
                e.insert(SpringJoint::new(parent));
            }
        }
        let id = e.id();
        commands.entity(parent).add_child(id);
        ids[node.index()] = id;
    }
    ids
}

/// Glow meshes as unlit children of their joints; returns them so an
/// Evolution can replace them.
pub fn spawn_glow(
    commands: &mut Commands,
    joints: &[Entity; NODE_COUNT],
    glow: &[Option<Handle<Mesh>>; NODE_COUNT],
    material: &Handle<StandardMaterial>,
) -> Vec<Entity> {
    glow.iter()
        .zip(joints)
        .filter_map(|(m, joint)| {
            let m = m.as_ref()?;
            let id = commands
                .spawn((
                    Mesh3d(m.clone()),
                    MeshMaterial3d(material.clone()),
                    Transform::IDENTITY,
                    bevy::light::NotShadowCaster,
                ))
                .id();
            commands.entity(*joint).add_child(id);
            Some(id)
        })
        .collect()
}
