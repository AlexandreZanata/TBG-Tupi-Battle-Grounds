//! Convert preserved twelve-joint specifications without discarding a unit
//! or form; leg parts ride whole thigh, shin or foot segments.

use super::{
    legacy, mesh,
    rig::{Joint, Node, NODE_COUNT},
};
use bevy::prelude::*;

pub fn joint(old: legacy::Joint) -> Joint {
    const MAP: [Joint; 12] = [
        Joint::Root,
        Joint::Hips,
        Joint::Chest,
        Joint::Head,
        Joint::UpperArmL,
        Joint::ForearmL,
        Joint::HandL,
        Joint::UpperArmR,
        Joint::ForearmR,
        Joint::HandR,
        Joint::ThighL,
        Joint::ThighR,
    ];
    MAP[old as usize]
}

pub fn build(spec: &legacy::FigureSpec) -> [Option<Mesh>; NODE_COUNT] {
    let mut parts: [Vec<(Mesh, Transform)>; NODE_COUNT] = Default::default();
    for old in &spec.parts {
        let r = old.rot * std::f32::consts::PI / 180.0;
        let (node, lift) = match old.joint {
            legacy::Joint::LegL | legacy::Joint::LegR => {
                segment(joint(old.joint), old.at.y, spec.plan.leg)
            }
            other => (joint(other), 0.0),
        };
        let transform = Transform::from_translation(old.at + Vec3::Y * lift)
            .with_rotation(Quat::from_euler(EulerRot::XYZ, r.x, r.y, r.z))
            .with_scale(old.scale);
        let m = mesh::paint(mesh::build(old.shape), old.color);
        parts[Node::Body(node).index()].push((m, transform));
    }
    parts.map(mesh::merge)
}

/// The leg segment a legacy leg part's center sits on, and how far up its
/// joint is: whole parts ride one segment, so knees bend without tearing.
fn segment(thigh: Joint, y: f32, length: f32) -> (Joint, f32) {
    let (shin, foot) = if thigh == Joint::ThighL {
        (Joint::ShinL, Joint::FootL)
    } else {
        (Joint::ShinR, Joint::FootR)
    };
    if y < -length + 0.025 {
        (foot, length)
    } else if y < -length * 0.5 {
        (shin, length * 0.5)
    } else {
        (thigh, 0.0)
    }
}
