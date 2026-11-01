//! Shared event-free figure construction and custom-anatomy transforms for catalog and Result.

use super::{
    anim::{Animator, Pose},
    figure, ik, kits,
    rig::{self, Proportions, NODE_COUNT},
    FigureLibrary,
};
use bevy::prelude::*;
use tbg_core::UnitKind;

pub fn spawn(
    commands: &mut Commands,
    lib: &FigureLibrary,
    kind: UnitKind,
    form: (u8, u8),
    transform: Transform,
    id: u32,
    reduced: bool,
) -> Entity {
    let model = lib.get(kind, form.0, form.1);
    let root = commands.spawn((transform, Visibility::Inherited)).id();
    let figure = commands
        .spawn((
            Transform::from_scale(Vec3::splat(model.plan.scale)),
            Visibility::Inherited,
        ))
        .id();
    commands.entity(root).add_child(figure);
    let joints = figure::spawn_joints(
        commands,
        figure,
        kind,
        &model.plan,
        model.meshes(true),
        &lib.material,
    );
    figure::spawn_glow(commands, &joints, &model.glow, &lib.glow_material);
    if reduced {
        for joint in joints {
            commands
                .entity(joint)
                .remove::<super::springs::SpringJoint>();
        }
    }
    let mut animator = Animator::new(joints, model.plan, figure, kind, id);
    animator.native = model.native;
    animator.form = form;
    animator.treadmill = true;
    commands.entity(root).insert(animator);
    root
}

/// Custom kit parents may appear later in the fixed slot order.
pub fn pose_world(kind: UnitKind, plan: &Proportions, pose: &Pose) -> [Mat4; NODE_COUNT] {
    let anatomy = kits::kit(kind).and_then(|k| k.anatomy);
    let mut local = [Mat4::IDENTITY; NODE_COUNT];
    let mut parents = [None; NODE_COUNT];
    for node in rig::nodes() {
        let i = node.index();
        let (rot, offset) = pose[i];
        let rest = anatomy.map_or_else(|| rig::rest(node, plan), |a| (a.rest)(node, plan));
        parents[i] = anatomy
            .map_or_else(|| node.parent(), |a| (a.parent)(node))
            .map(|p| p.index());
        local[i] = Mat4::from_rotation_translation(ik::quat(rot), rest + offset);
    }
    let mut world = [Mat4::IDENTITY; NODE_COUNT];
    let mut seen = [0u8; NODE_COUNT];
    for i in 0..NODE_COUNT {
        resolve(i, &local, &parents, &mut world, &mut seen);
    }
    world
}

fn resolve(
    i: usize,
    local: &[Mat4; NODE_COUNT],
    parents: &[Option<usize>; NODE_COUNT],
    world: &mut [Mat4; NODE_COUNT],
    seen: &mut [u8; NODE_COUNT],
) -> Mat4 {
    if seen[i] == 2 {
        return world[i];
    }
    if seen[i] == 1 {
        return Mat4::IDENTITY;
    }
    seen[i] = 1;
    let parent = parents[i].map_or(Mat4::IDENTITY, |p| resolve(p, local, parents, world, seen));
    world[i] = parent * local[i];
    seen[i] = 2;
    world[i]
}
