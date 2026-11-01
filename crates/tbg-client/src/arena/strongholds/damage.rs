//! Cumulative 5% wear, stable piece visibility and bounded tower damage effects.
use super::{
    parts::{piece, t, PartDef, Piece},
    Tower,
};
use crate::art::{mesh::Shape::*, palette::*};
use bevy::prelude::*;
use tbg_core::StrongholdKind;

#[derive(Component)]
pub(super) struct Wear {
    pub show: u8,
    pub hide: u8,
}

pub(super) fn stage(hp: i32, max_hp: i32) -> u8 {
    let max = i64::from(max_hp.max(1));
    ((max - i64::from(hp).clamp(0, max)) * 20 / max) as u8
}

static CHIP: PartDef = PartDef {
    name: "wear facing chip",
    build: |_| vec![(Bevel(0.12, 0.09, 0.035, 0.01), STONE, Transform::IDENTITY)],
};
static TIMBER_CHIP: PartDef = PartDef {
    name: "wear timber splinter",
    build: |_| vec![(Bevel(0.12, 0.09, 0.035, 0.01), OAK, Transform::IDENTITY)],
};
static CRACK: PartDef = PartDef {
    name: "wear masonry fracture",
    build: |_| {
        vec![
            (
                Box(0.018, 0.14, 0.009),
                CHARCOAL,
                Transform::from_rotation(Quat::from_rotation_z(0.4)),
            ),
            (Box(0.07, 0.012, 0.01), CHARCOAL, t(0.025, 0.02, 0.0)),
        ]
    },
};

/// Kind-specific positions on the camera-facing foundation, not floating smoke in the lane.
fn profile(kind: StrongholdKind) -> (f32, f32, f32) {
    match kind {
        StrongholdKind::Artillery => (0.62, 0.28, 0.62),
        StrongholdKind::Bastion => (0.68, 0.55, 0.74),
        StrongholdKind::Treasury => (0.52, 0.60, 0.52),
        StrongholdKind::Command => (0.47, 0.70, 0.47),
        StrongholdKind::Classic => (0.46, 0.65, 0.46),
    }
}

pub(super) fn regions(kind: StrongholdKind) -> Vec<(Piece, Wear)> {
    let (width, y, z) = profile(kind);
    (1..20)
        .flat_map(|stage| {
            let x = (f32::from(stage % 5) / 4.0 - 0.5) * width * 1.7;
            let y = y + f32::from(stage / 5) * 0.12;
            [
                (
                    piece(
                        if kind == StrongholdKind::Artillery {
                            &TIMBER_CHIP
                        } else {
                            &CHIP
                        },
                        t(x, y, z),
                    ),
                    Wear {
                        show: 0,
                        hide: stage,
                    },
                ),
                (
                    piece(&CRACK, t(x, y, z + 0.025)),
                    Wear {
                        show: stage,
                        hide: 21,
                    },
                ),
            ]
        })
        .collect()
}

pub(super) fn visibility(
    towers: Query<&Tower>,
    mut parts: Query<(&Wear, &ChildOf, &mut Visibility, &mut Transform), Without<Tower>>,
) {
    for (wear, parent, mut visibility, mut transform) in &mut parts {
        let Ok(tower) = towers.get(parent.parent()) else {
            continue;
        };
        let stage = tower.look.map_or(0, |l| l.damage);
        *visibility = if stage >= wear.show && stage < wear.hide {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if stage == 20 {
            transform.scale.y = 0.08;
            transform.translation.y = transform.translation.y.min(0.12);
        }
    }
}

#[derive(Component, Default)]
pub(super) struct Effects {
    previous: Option<u8>,
    timer: f32,
}

pub(super) fn effects(
    mut commands: Commands,
    view: Res<crate::sync::LatestView>,
    scene: Res<State<crate::GameState>>,
    time: Res<Time>,
    assets: Res<crate::vfx::VfxAssets>,
    mut towers: Query<(&Tower, &Transform, &mut Effects)>,
) {
    for (tower, root, mut state) in &mut towers {
        if !matches!(
            *scene.get(),
            crate::GameState::InMatch | crate::GameState::Result | crate::GameState::Gallery
        ) || (tower.live && view.0.is_none())
        {
            state.previous = None;
            continue;
        }
        let stage = tower.look.map_or(0, |l| l.damage);
        if let Some(old) = state.previous {
            if stage > old {
                crate::vfx::wear::debris(
                    &mut commands,
                    &assets,
                    root.translation,
                    stage - old,
                    tower.kind() == StrongholdKind::Artillery,
                );
            }
        }
        state.previous = Some(stage);
        state.timer -= time.delta_secs();
        if stage < 5 || state.timer > 0.0 {
            continue;
        }
        state.timer = if crate::config::low_motion() {
            0.65
        } else {
            0.32
        };
        let (w, y, z) = profile(tower.kind());
        let count = if stage < 9 {
            1
        } else if stage < 14 {
            2
        } else {
            3
        };
        for socket in 0..count {
            let at = root.translation
                + Vec3::new(
                    (socket as f32 - 1.0) * w * 0.6,
                    if stage == 20 { 0.12 } else { y },
                    z - 0.05,
                );
            crate::vfx::wear::burn(&mut commands, &assets, at, stage >= 9, stage);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn p71_damage_tracks_every_completed_five_percent_without_overflow() {
        for max in [1, 4500, 5175, 5796, i32::MAX] {
            assert_eq!(stage(max, max), 0);
            assert_eq!(stage(0, max), 20);
            for k in 1..=20 {
                let edge = (i64::from(max) * (20 - k) / 20) as i32;
                assert!(stage(edge, max) >= k as u8);
                if edge < max {
                    assert!(stage(edge + 1, max) < k as u8);
                }
            }
        }
        assert_eq!(stage(-1, 0), 20);
        assert_eq!(stage(i32::MAX, 0), 0);
    }
}
