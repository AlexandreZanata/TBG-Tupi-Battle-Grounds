//! Preserved P6 twelve-joint figure specifications, consumed only by the v2 adapter.
use bevy::prelude::*;

use super::mesh::Shape;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Joint {
    #[expect(
        dead_code,
        reason = "Preserved P6 data contract; the root carries no mesh"
    )]
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
