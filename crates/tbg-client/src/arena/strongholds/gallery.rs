//! `?gallery=strongholds`: every Stronghold kind in every form for review
//! screenshots. Rows are Treasury, Bastion, Artillery, Command; columns are
//! tier 0, A1, A2, B1, B2, with Classic closing the first row. `&enemy`
//! shows the enemy color and facing, `&damage=1|2` a damage state.
//! Artillery fires at its authoritative interval so the recoil can be timed.

use super::{dress, Look, StrongholdArt, Tower};
use crate::art::palette::INK;
use crate::{config, GameState};
use bevy::prelude::*;
use tbg_core::StrongholdKind;

const COLUMN: f32 = 3.2;
const ROW: f32 = 3.6;
const FORMS: [(u8, u8); 5] = [(0, 0), (1, 0), (2, 0), (1, 1), (2, 1)];

pub(super) fn wanted(state: Res<State<GameState>>) -> bool {
    *state.get() == GameState::Gallery
}

pub(super) fn spawn(
    mut commands: Commands,
    mut done: Local<bool>,
    mut art: ResMut<StrongholdArt>,
    mut meshes: ResMut<Assets<Mesh>>,
    live: Query<Entity, With<Tower>>,
) {
    if *done {
        return;
    }
    *done = true;
    if !config::url_param("gallery").is_some_and(|m| m.eq_ignore_ascii_case("strongholds")) {
        return;
    }
    for root in &live {
        commands.entity(root).insert(Visibility::Hidden);
    }
    let own = config::url_param("enemy").is_none();
    let damage = config::url_param("damage")
        .and_then(|d| d.parse::<u8>().ok())
        .unwrap_or(0)
        .min(2);
    let mut place = |kind, (tier, branch), x: f32, z: f32| {
        let look = Look {
            kind,
            tier,
            branch,
            damage,
        };
        let mut tower = Tower::new(own, false);
        let root = commands
            .spawn((
                Name::new(format!("gallery {kind:?} {tier}/{branch}")),
                Transform::from_xyz(x, 0.0, z),
                Visibility::default(),
            ))
            .id();
        dress(&mut commands, &mut art, &mut meshes, root, &mut tower, look);
        commands.entity(root).insert(tower);
    };
    for (row, kind) in StrongholdKind::SELECTABLE.into_iter().enumerate() {
        for (col, form) in FORMS.into_iter().enumerate() {
            place(kind, form, col as f32 * COLUMN, -(row as f32) * ROW);
        }
    }
    place(StrongholdKind::Classic, (0, 0), 5.0 * COLUMN, 0.0);
    commands.spawn((
        Text::new(format!(
            "STRONGHOLDS | damage {damage} | {}\n\
             rows (back to front): Command / Artillery / Bastion / Treasury\n\
             columns: tier 0 / A1 / A2 / B1 / B2 (Classic at the far right)",
            if own { "own" } else { "enemy" }
        )),
        TextFont {
            font_size: FontSize::Px(16.0),
            ..default()
        },
        TextColor(INK),
        Node {
            position_type: PositionType::Absolute,
            left: px(12),
            top: px(12),
            max_width: percent(94),
            ..default()
        },
    ));
}
