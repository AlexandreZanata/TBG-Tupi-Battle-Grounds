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
    /// Bow string halves: children of the bow, pivoting on each limb tip.
    StringTop,
    StringBottom,
    /// The nocked arrow: a child of the bow that slides back along the string.
    Nock,
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
pub const NODE_COUNT: usize = BODY_COUNT + 3 + 6;
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
pub const SOCKETS: [Socket; 6] = [
    Socket::WeaponR,
    Socket::OffhandL,
    Socket::Back,
    Socket::StringTop,
    Socket::StringBottom,
    Socket::Nock,
];

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
        const PARENTS: [Option<Joint>; BODY_COUNT] = [
            None,
            Some(Root),
            Some(Hips),
            Some(Spine),
            Some(Chest),
            Some(Chest),
            Some(UpperArmL),
            Some(ForearmL),
            Some(Chest),
            Some(UpperArmR),
            Some(ForearmR),
            Some(Hips),
            Some(ThighL),
            Some(ShinL),
            Some(Hips),
            Some(ThighR),
            Some(ShinR),
        ];
        match self {
            Self::Body(j) => PARENTS[j.index()].map(Self::Body),
            Self::Secondary(s) => Some(Self::Body([Chest, Head, Hips][s as usize])),
            Self::Prop(s) => Some(
                [
                    Self::Body(HandR),
                    Self::Body(HandL),
                    Self::Secondary(Spring::Back),
                    Self::Prop(Socket::OffhandL),
                    Self::Prop(Socket::OffhandL),
                    Self::Prop(Socket::OffhandL),
                ][s as usize],
            ),
        }
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
    let arm = p.torso.x / 2.0 + 0.05;
    let body = [
        Vec3::ZERO,
        Vec3::new(0.0, p.leg + 0.06, 0.0),
        Vec3::Y * 0.02,
        Vec3::ZERO,
        Vec3::new(0.0, p.torso.y + p.head * 0.85, 0.0),
        Vec3::new(arm, p.torso.y * 0.9, 0.0),
        -Vec3::Y * p.arm,
        -Vec3::Y * p.arm * 0.9,
        Vec3::new(-arm, p.torso.y * 0.9, 0.0),
        -Vec3::Y * p.arm,
        -Vec3::Y * p.arm * 0.9,
        Vec3::X * p.torso.x * 0.25,
        -Vec3::Y * p.leg * 0.5,
        -Vec3::Y * p.leg * 0.5,
        -Vec3::X * p.torso.x * 0.25,
        -Vec3::Y * p.leg * 0.5,
        -Vec3::Y * p.leg * 0.5,
    ];
    let secondary = [
        Vec3::new(0.0, p.torso.y * 0.65, -p.torso.z * 0.65),
        Vec3::ZERO,
        Vec3::new(0.0, 0.0, p.torso.z * 0.55),
    ];
    match node {
        Node::Body(j) => body[j.index()],
        Node::Secondary(s) => secondary[s as usize],
        Node::Prop(_) => Vec3::ZERO,
    }
}
