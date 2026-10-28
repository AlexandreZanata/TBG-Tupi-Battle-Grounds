//! Data-only v2 hierarchy, body proportions, rest transforms, and prop sockets.

pub use super::legacy::BodyPlan as Proportions;
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Joint {
    Root,
    Hips,
    Spine,
    Chest,
    Head,
    UpperArmL,
    ForearmL,
    HandL,
    UpperArmR,
    ForearmR,
    HandR,
    ThighL,
    ShinL,
    FootL,
    ThighR,
    ShinR,
    FootR,
}
#[derive(Clone, Copy, Debug)]
pub enum Spring {
    Back,
    HeadGear,
    Tabard,
}
#[derive(Clone, Copy, Debug)]
pub enum Socket {
    WeaponR,
    OffhandL,
    Back,
}
#[derive(Clone, Copy, Debug)]
pub enum BodyPlan {
    Humanoid,
    Large,
}
#[derive(Clone, Copy, Debug)]
pub enum Node {
    Body(Joint),
    Secondary(Spring),
    Prop(Socket),
}

pub const BODY_COUNT: usize = 17;
pub const NODE_COUNT: usize = BODY_COUNT + 3 + 3;
pub const JOINTS: [Joint; BODY_COUNT] = [
    Joint::Root,
    Joint::Hips,
    Joint::Spine,
    Joint::Chest,
    Joint::Head,
    Joint::UpperArmL,
    Joint::ForearmL,
    Joint::HandL,
    Joint::UpperArmR,
    Joint::ForearmR,
    Joint::HandR,
    Joint::ThighL,
    Joint::ShinL,
    Joint::FootL,
    Joint::ThighR,
    Joint::ShinR,
    Joint::FootR,
];
pub const SPRINGS: [Spring; 3] = [Spring::Back, Spring::HeadGear, Spring::Tabard];
pub const SOCKETS: [Socket; 3] = [Socket::WeaponR, Socket::OffhandL, Socket::Back];

impl Joint {
    pub const fn index(self) -> usize {
        self as usize
    }
}
impl Node {
    pub const fn index(self) -> usize {
        match self {
            Self::Body(j) => j as usize,
            Self::Secondary(s) => BODY_COUNT + s as usize,
            Self::Prop(s) => BODY_COUNT + 3 + s as usize,
        }
    }
    pub fn parent(self) -> Option<Node> {
        use Joint::*;
        Some(match self {
            Self::Body(Root) => return None,
            Self::Body(Hips) => Self::Body(Root),
            Self::Body(Spine | ThighL | ThighR) => Self::Body(Hips),
            Self::Body(Chest) => Self::Body(Spine),
            Self::Body(Head | UpperArmL | UpperArmR) => Self::Body(Chest),
            Self::Body(ForearmL) => Self::Body(UpperArmL),
            Self::Body(HandL) => Self::Body(ForearmL),
            Self::Body(ForearmR) => Self::Body(UpperArmR),
            Self::Body(HandR) => Self::Body(ForearmR),
            Self::Body(ShinL) => Self::Body(ThighL),
            Self::Body(FootL) => Self::Body(ShinL),
            Self::Body(ShinR) => Self::Body(ThighR),
            Self::Body(FootR) => Self::Body(ShinR),
            Self::Secondary(Spring::Back) => Self::Body(Chest),
            Self::Secondary(Spring::HeadGear) => Self::Body(Head),
            Self::Secondary(Spring::Tabard) => Self::Body(Hips),
            Self::Prop(Socket::WeaponR) => Self::Body(HandR),
            Self::Prop(Socket::OffhandL) => Self::Body(HandL),
            Self::Prop(Socket::Back) => Self::Secondary(Spring::Back),
        })
    }
}
impl BodyPlan {
    pub fn proportions(self) -> Proportions {
        Proportions {
            scale: if matches!(self, Self::Large) {
                1.5
            } else {
                1.0
            },
            leg: 0.38,
            torso: Vec3::new(
                if matches!(self, Self::Large) {
                    0.46
                } else {
                    0.36
                },
                0.34,
                0.24,
            ),
            head: 0.1365,
            arm: 0.17,
        }
    }
}

pub fn nodes() -> impl Iterator<Item = Node> {
    JOINTS
        .into_iter()
        .map(Node::Body)
        .chain(SPRINGS.map(Node::Secondary))
        .chain(SOCKETS.map(Node::Prop))
}

pub fn rest(node: Node, p: &Proportions) -> Vec3 {
    use Joint::*;
    match node {
        Node::Body(Root | Chest) => Vec3::ZERO,
        Node::Body(Hips) => Vec3::new(0.0, p.leg + 0.06, 0.0),
        Node::Body(Spine) => Vec3::new(0.0, 0.02, 0.0),
        Node::Body(Head) => Vec3::new(0.0, p.torso.y + p.head * 0.85, 0.0),
        Node::Body(UpperArmL | UpperArmR) => Vec3::new(
            (p.torso.x / 2.0 + 0.05)
                * if matches!(node, Node::Body(UpperArmL)) {
                    1.0
                } else {
                    -1.0
                },
            p.torso.y * 0.9,
            0.0,
        ),
        Node::Body(ForearmL | ForearmR) => Vec3::new(0.0, -p.arm, 0.0),
        Node::Body(HandL | HandR) => Vec3::new(0.0, -p.arm * 0.9, 0.0),
        Node::Body(ThighL | ThighR) => Vec3::new(
            p.torso.x
                * 0.25
                * if matches!(node, Node::Body(ThighL)) {
                    1.0
                } else {
                    -1.0
                },
            0.0,
            0.0,
        ),
        Node::Body(ShinL | FootL | ShinR | FootR) => Vec3::new(0.0, -p.leg * 0.5, 0.0),
        Node::Secondary(Spring::Back) => Vec3::new(0.0, p.torso.y * 0.65, -p.torso.z * 0.65),
        Node::Secondary(Spring::HeadGear) => Vec3::ZERO,
        Node::Secondary(Spring::Tabard) => Vec3::new(0.0, 0.0, p.torso.z * 0.55),
        Node::Prop(_) => Vec3::ZERO,
    }
}
