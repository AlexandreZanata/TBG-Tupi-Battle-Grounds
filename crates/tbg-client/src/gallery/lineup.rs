//! `&lineup`: one unit's base and A-branch evolutions side by side, frozen at
//! idle and at the attack anchor (review sheet for the `?archer=v2` candidate).

use super::{figures::spawn_one, layout::Layout};
use crate::art::{anim::Clip, FigureLibrary};
use bevy::prelude::*;
use tbg_core::UnitKind;

/// Poses per row: idle, and the full draw just before the 0.18 s Release.
const POSES: [(Clip, f32); 2] = [(Clip::Idle, 0.4), (Clip::AttackRanged, 0.15)];

pub fn spawn(
    commands: &mut Commands,
    lib: &FigureLibrary,
    unit: UnitKind,
    material: &Handle<StandardMaterial>,
    portrait: bool,
) {
    for (row, (clip, t)) in POSES.into_iter().enumerate() {
        for tier in 0..3u8 {
            let layout = Layout::Lineup(row, usize::from(tier));
            let pos = layout.position(portrait);
            let (root, mut animator) = spawn_one(commands, lib, unit, (tier, 0), pos, material);
            animator.preview = Some((clip, t));
            let stage = commands
                .spawn((
                    Mesh3d(lib.stage.clone()),
                    MeshMaterial3d(lib.material.clone()),
                    Transform::from_xyz(0.0, -0.08, 0.0),
                ))
                .id();
            commands
                .entity(root)
                .add_child(stage)
                .insert((animator, layout));
        }
    }
}
