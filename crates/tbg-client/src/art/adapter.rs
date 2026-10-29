//! Convert preserved twelve-joint specifications without discarding a unit or form.

use super::{
    figure, legacy, mesh,
    rig::{Joint, Node, NODE_COUNT},
};
use bevy::{mesh::VertexAttributeValues, prelude::*};

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
        let mut p = figure::part(
            Node::Body(joint(old.joint)),
            old.shape,
            old.color,
            old.at.into(),
        );
        p.rot = old.rot;
        p.scale = old.scale;
        let r = p.rot * std::f32::consts::PI / 180.0;
        let transform = Transform::from_translation(p.at)
            .with_rotation(Quat::from_euler(EulerRot::XYZ, r.x, r.y, r.z))
            .with_scale(p.scale);
        let m = mesh::paint(mesh::build(p.shape), p.color).transformed_by(transform);
        if matches!(old.joint, legacy::Joint::LegL | legacy::Joint::LegR) {
            split_leg(m, joint(old.joint), spec.plan.leg, &mut parts);
        } else {
            parts[p.node.index()].push((m, Transform::IDENTITY));
        }
    }
    parts.map(mesh::merge)
}
fn split_leg(m: Mesh, thigh: Joint, length: f32, parts: &mut [Vec<(Mesh, Transform)>; NODE_COUNT]) {
    let (shin, foot) = if thigh == Joint::ThighL {
        (Joint::ShinL, Joint::FootL)
    } else {
        (Joint::ShinR, Joint::FootR)
    };
    let Some(VertexAttributeValues::Float32x3(positions)) = m.attribute(Mesh::ATTRIBUTE_POSITION)
    else {
        return;
    };
    let mut groups = [Vec::new(), Vec::new(), Vec::new()];
    for (i, triangle) in positions.as_chunks::<3>().0.iter().enumerate() {
        let y = triangle.iter().map(|p| p[1]).sum::<f32>() / 3.0;
        let group = if y < -length + 0.025 {
            2
        } else if y < -length * 0.5 {
            1
        } else {
            0
        };
        groups[group].extend(i * 3..i * 3 + 3);
    }
    for (group, node) in groups.into_iter().zip([thigh, shin, foot]) {
        if group.is_empty() {
            continue;
        }
        let mut piece = m.clone();
        select_vertices(&mut piece, &m, &group);
        let offset = if node == thigh {
            0.0
        } else if node == shin {
            length * 0.5
        } else {
            length
        };
        parts[node.index()].push((piece, Transform::from_xyz(0.0, offset, 0.0)));
    }
}

fn select_vertices(piece: &mut Mesh, m: &Mesh, group: &[usize]) {
    for attribute in [
        Mesh::ATTRIBUTE_POSITION,
        Mesh::ATTRIBUTE_NORMAL,
        Mesh::ATTRIBUTE_COLOR,
    ] {
        let Some(values) = m.attribute(attribute) else {
            continue;
        };
        let selected = match values {
            VertexAttributeValues::Float32x3(v) => {
                VertexAttributeValues::Float32x3(group.iter().map(|&i| v[i]).collect())
            }
            VertexAttributeValues::Float32x4(v) => {
                VertexAttributeValues::Float32x4(group.iter().map(|&i| v[i]).collect())
            }
            _ => continue,
        };
        piece.insert_attribute(attribute, selected);
    }
}
