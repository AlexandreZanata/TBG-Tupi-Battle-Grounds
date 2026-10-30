//! `&lineup`: one unit's five forms (base, A1, A2, B1, B2) side by side,
//! frozen at idle, mid-stride, and at its archetype's attack key pose.

use super::{figures::spawn_one, layout::Layout};
use crate::art::{anim::Clip, FigureLibrary, FORMS};
use bevy::prelude::*;
use tbg_core::{Archetype, UnitKind};

/// The attack row's clip and moment: the full draw just before a ranged
/// Release, the windup peak of a swing, the raised hands of a cast.
fn attack(unit: UnitKind) -> (Clip, f32) {
    match tbg_core::content::def(unit).archetype {
        Archetype::Ranged => (Clip::AttackRanged, 0.15),
        Archetype::Tank | Archetype::Dps => (Clip::AttackMelee, 0.22),
        Archetype::Control | Archetype::Support => (Clip::Cast, 0.18),
    }
}

pub fn spawn(
    commands: &mut Commands,
    lib: &FigureLibrary,
    unit: UnitKind,
    material: &Handle<StandardMaterial>,
    portrait: bool,
) {
    let poses = [(Clip::Idle, 0.4), (Clip::Walk, 0.15), attack(unit)];
    for (row, (clip, t)) in poses.into_iter().enumerate() {
        for (col, form) in FORMS.into_iter().enumerate() {
            let layout = Layout::Lineup(row, col);
            let pos = layout.position(portrait);
            let (root, mut animator) = spawn_one(commands, lib, unit, form, pos, material);
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
