//! Stronghold meshes and their existing three HP-driven damage states.

use super::painted;
use crate::{
    art::{mesh::Shape::*, palette::*},
    config,
    sync::LatestView,
};
use bevy::prelude::*;
use tbg_core::content::STRONGHOLD_HP;

#[derive(Component)]
pub(super) struct TowerPiece {
    pub own: bool,
    pub hide_below: f32,
}

const BASE: &[[f32; 2]] = &[[1.0, 0.0], [1.0, 0.18], [0.82, 0.24], [0.76, 0.95]];
const MID: &[[f32; 2]] = &[[0.76, 0.95], [0.7, 1.65], [0.84, 1.75], [0.84, 1.85]];

/// Base, middle, and crown of a tower; the middle hides at ≤ 33% HP, the
/// crown at ≤ 66%.
pub(super) fn tower(own: bool) -> [(Mesh, f32); 3] {
    let banner = if own { ALLY } else { ENEMY };
    let at = |y: f32| Transform::from_xyz(0.0, y, 0.0);
    let mut base_parts = vec![
        (Lathe(BASE), STONE, at(0.0)),
        (Cyl(0.76, 0.02), STONE, at(0.95)),
        (
            Box(0.32, 0.5, 0.06),
            WOOD,
            Transform::from_xyz(0.78, 0.25, 0.0).with_rotation(Quat::from_rotation_y(1.57)),
        ),
    ];
    for y in [0.24, 0.59, 0.94] {
        base_parts.push((
            Ring(0.016, if y < 0.3 { 0.83 } else { 0.77 }, 360.0),
            ROCK,
            at(y),
        ));
    }
    let base = painted(base_parts);
    let mid = painted(vec![
        (Lathe(MID), STONE, at(0.0)),
        (Cyl(0.84, 0.02), ROCK, at(1.85)),
        (
            Box(0.44, 0.65, 0.035),
            banner,
            Transform::from_xyz(0.0, 1.25, 0.77),
        ),
    ]);
    let mut crown = Vec::new();
    crown.push((
        Box(0.13, 0.13, 0.025),
        LINEN,
        Transform::from_xyz(0.0, 1.30, 0.80).with_rotation(Quat::from_rotation_z(0.785)),
    ));
    for i in 0..8 {
        let a = i as f32 / 8.0 * std::f32::consts::TAU;
        crown.push((
            Box(0.22, 0.24, 0.22),
            STONE,
            Transform::from_xyz(a.cos() * 0.72, 1.97, a.sin() * 0.72),
        ));
    }
    crown.push((Cyl(0.03, 0.9), WOOD, at(2.3)));
    crown.push((
        Box(0.04, 0.32, 0.5),
        banner,
        Transform::from_xyz(0.0, 2.55, 0.26),
    ));
    [(base, -1.0), (mid, 1.0 / 3.0), (painted(crown), 2.0 / 3.0)]
}

/// Towers lose their crown, then their middle, as Stronghold HP drops.
pub(super) fn damage_states(
    view: Res<LatestView>,
    mut pieces: Query<(&TowerPiece, &mut Visibility)>,
) {
    if config::url_param("gallery").is_some() {
        return;
    }
    let hp = view.0.as_ref().map(|v| {
        let me = v.you.index();
        [v.stronghold_hp[me], v.stronghold_hp[1 - me]].map(|h| h as f32 / STRONGHOLD_HP as f32)
    });
    for (piece, mut vis) in &mut pieces {
        let fraction = hp.map_or(1.0, |[own, enemy]| if piece.own { own } else { enemy });
        let want = if fraction > piece.hide_below {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *vis != want {
            *vis = want;
        }
    }
}
