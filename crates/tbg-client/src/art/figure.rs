//! Rigid-part figurines on one shared 12-joint body plan (decision D-07).
//!
//! API used by `visuals.rs`:
//! - `FigureSpec` (data: body plan + parts per joint), built by `roster`;
//! - `build_joint_meshes(spec)` → one merged mesh per joint;
//! - `spawn_joints(commands, plan, meshes, material)` → joint entities under a root;
//! - `rest(joint, plan)` → a joint's rest translation (used by `anim`).
//!
//! Figures face +Z, stand on y = 0, and are about 1.05 units tall at scale 1.

use bevy::prelude::*;

use super::mesh::{self, Shape};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Joint {
    Root,
    Hips,
    Torso,
    Head,
    ArmL,
    ForearmL,
    HandL,
    ArmR,
    ForearmR,
    HandR,
    LegL,
    LegR,
}

pub const JOINTS: [Joint; 12] = [
    Joint::Root,
    Joint::Hips,
    Joint::Torso,
    Joint::Head,
    Joint::ArmL,
    Joint::ForearmL,
    Joint::HandL,
    Joint::ArmR,
    Joint::ForearmR,
    Joint::HandR,
    Joint::LegL,
    Joint::LegR,
];

impl Joint {
    pub fn index(self) -> usize {
        self as usize
    }

    pub fn parent(self) -> Option<Joint> {
        use Joint::*;
        match self {
            Root => None,
            Hips => Some(Root),
            Torso | LegL | LegR => Some(Hips),
            Head | ArmL | ArmR => Some(Torso),
            ForearmL => Some(ArmL),
            HandL => Some(ForearmL),
            ForearmR => Some(ArmR),
            HandR => Some(ForearmR),
        }
    }
}

/// Proportions. Big head, short legs, chunky hands (toy figurine).
#[derive(Clone, Copy, Debug)]
pub struct BodyPlan {
    pub scale: f32,
    pub leg: f32,
    pub torso: Vec3,
    pub head: f32,
    pub arm: f32,
}

impl BodyPlan {
    pub const SMALL: BodyPlan = BodyPlan {
        scale: 1.0,
        leg: 0.3,
        torso: Vec3::new(0.36, 0.36, 0.24),
        head: 0.17,
        arm: 0.17,
    };
    pub const BIG: BodyPlan = BodyPlan {
        scale: 1.45,
        leg: 0.3,
        torso: Vec3::new(0.46, 0.4, 0.3),
        head: 0.15,
        arm: 0.19,
    };
}

/// A joint's rest position relative to its parent.
pub fn rest(joint: Joint, p: &BodyPlan) -> Vec3 {
    use Joint::*;
    let side = |s: f32| s * (p.torso.x / 2.0 + 0.05);
    match joint {
        Root => Vec3::ZERO,
        Hips => Vec3::new(0.0, p.leg + 0.06, 0.0),
        Torso => Vec3::new(0.0, 0.02, 0.0),
        Head => Vec3::new(0.0, p.torso.y + p.head * 0.85, 0.0),
        ArmL => Vec3::new(side(1.0), p.torso.y * 0.9, 0.0),
        ArmR => Vec3::new(side(-1.0), p.torso.y * 0.9, 0.0),
        ForearmL | ForearmR => Vec3::new(0.0, -p.arm, 0.0),
        HandL | HandR => Vec3::new(0.0, -p.arm * 0.9, 0.0),
        LegL => Vec3::new(p.torso.x * 0.25, 0.0, 0.0),
        LegR => Vec3::new(-p.torso.x * 0.25, 0.0, 0.0),
    }
}

/// One mesh piece attached to a joint.
#[derive(Clone, Copy, Debug)]
pub struct PartSpec {
    pub joint: Joint,
    pub shape: Shape,
    pub color: Color,
    pub at: Vec3,
    pub rot: Vec3,
    pub scale: Vec3,
}

/// Shorthand: `part(joint, shape, color, [x, y, z])`.
pub fn part(joint: Joint, shape: Shape, color: Color, at: [f32; 3]) -> PartSpec {
    PartSpec {
        joint,
        shape,
        color,
        at: Vec3::from(at),
        rot: Vec3::ZERO,
        scale: Vec3::ONE,
    }
}

impl PartSpec {
    /// Rotation in degrees (XYZ order).
    pub fn rot(mut self, deg: [f32; 3]) -> Self {
        self.rot = Vec3::from(deg);
        self
    }

    pub fn scale(mut self, s: [f32; 3]) -> Self {
        self.scale = Vec3::from(s);
        self
    }
}

#[derive(Clone, Debug)]
pub struct FigureSpec {
    pub plan: BodyPlan,
    pub parts: Vec<PartSpec>,
}

/// One merged, painted mesh per joint (`None` for joints without parts).
pub fn build_joint_meshes(spec: &FigureSpec) -> [Option<Mesh>; 12] {
    let mut per_joint: [Vec<(Mesh, Transform)>; 12] = Default::default();
    for p in &spec.parts {
        let r = p.rot * std::f32::consts::PI / 180.0;
        let t = Transform::from_translation(p.at)
            .with_rotation(Quat::from_euler(EulerRot::XYZ, r.x, r.y, r.z))
            .with_scale(p.scale);
        per_joint[p.joint.index()].push((mesh::paint(mesh::build(p.shape), p.color), t));
    }
    per_joint.map(mesh::merge)
}

/// Spawn the joint hierarchy under `root`; returns the 12 joint entities.
pub fn spawn_joints(
    commands: &mut Commands,
    root: Entity,
    plan: &BodyPlan,
    meshes: &[Option<Handle<Mesh>>; 12],
    material: &Handle<StandardMaterial>,
) -> [Entity; 12] {
    let mut ids = [root; 12];
    for j in JOINTS {
        let t = Transform::from_translation(rest(j, plan));
        let mut e = commands.spawn((t, Visibility::default()));
        if let Some(m) = &meshes[j.index()] {
            e.insert((Mesh3d(m.clone()), MeshMaterial3d(material.clone())));
        }
        let id = e.id();
        let parent = j.parent().map_or(root, |p| ids[p.index()]);
        commands.entity(parent).add_child(id);
        ids[j.index()] = id;
    }
    ids
}
