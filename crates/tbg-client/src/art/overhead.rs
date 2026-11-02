//! Cached body envelopes for overhead UI, excluding long hand-held weapons.
use super::{
    anim::{Animator, Clip},
    rig::{Node, Socket, NODE_COUNT},
    FigureForm,
};
use bevy::{mesh::VertexAttributeValues, prelude::*};
use tbg_core::UnitKind;

/// Keep measured geometry and live crowd participation on the same selection.
pub(crate) fn reserves_equipment(kind: UnitKind) -> bool {
    matches!(
        kind,
        UnitKind::SiegeWalker | UnitKind::Colossus | UnitKind::GustDrummer
    )
}

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
    let equipment = reserves_equipment(kind);
    let clips = [
        Clip::Idle,
        Clip::Walk,
        if kind == UnitKind::GustDrummer {
            Clip::Cast
        } else if kind == UnitKind::Colossus {
            Clip::AttackMelee
        } else if equipment {
            Clip::AttackRanged
        } else {
            Clip::AttackMelee
        },
    ];
    // The turtle strikes with its body; include its head/paw drive in contact clearance.
    let samples = if kind == UnitKind::Shatterback || equipment {
        3
    } else {
        2
    };
    let targets = [
        None,
        Some(Vec3::new(0.0, 0.6, 1.2)),
        Some(Vec3::new(0.0, 0.6, 4.5)),
    ];
    for target in targets.into_iter().take(if equipment { 3 } else { 1 }) {
        for clip in clips.into_iter().take(samples) {
            for step in 0..16 {
                let t = step as f32 * a.duration(clip) / 16.0;
                let pose = if equipment {
                    a.clearance_pose(clip, t, target, step as f32 * 1.6 / 16.0)
                } else {
                    a.pose(clip, t)
                };
                let world = super::presentation::pose_world(kind, &form.plan, &pose);
                for node in super::rig::nodes() {
                    let hand_equipment = matches!(
                        node,
                        Node::Prop(
                            Socket::WeaponR
                                | Socket::OffhandL
                                | Socket::Nock
                                | Socket::StringTop
                                | Socket::StringBottom
                        )
                    );
                    if hand_equipment && (!equipment || kind == UnitKind::Colossus) {
                        continue;
                    }
                    let colossus_arm = matches!(
                        node,
                        Node::Body(
                            super::rig::Joint::UpperArmL
                                | super::rig::Joint::ForearmL
                                | super::rig::Joint::HandL
                                | super::rig::Joint::UpperArmR
                                | super::rig::Joint::ForearmR
                                | super::rig::Joint::HandR
                        )
                    );
                    for layer in [&form.meshes, &form.glow] {
                        let Some(VertexAttributeValues::Float32x3(v)) = layer[node.index()]
                            .as_ref()
                            .and_then(|h| meshes.get(h))
                            .and_then(|m| m.attribute(Mesh::ATTRIBUTE_POSITION))
                        else {
                            continue;
                        };
                        for p in v {
                            let at = world[node.index()].transform_point3(Vec3::from(*p))
                                * form.plan.scale;
                            if !hand_equipment {
                                top = top.max(at.y);
                            }
                            if kind != UnitKind::Colossus || colossus_arm {
                                radius = radius.max(at.x.abs());
                                depth.x = depth.x.min(at.z);
                                depth.y = depth.y.max(at.z);
                            }
                        }
                    }
                }
            }
        }
    }
    if equipment {
        depth += Vec2::new(-0.03, 0.03);
        radius += 0.03;
    }
    // Include breathing/selection lift and the billboard's lower edge at the normal camera pitch.
    (top + 0.32, depth, radius)
}
