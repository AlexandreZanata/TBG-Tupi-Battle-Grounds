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
    // Hide roots, since wear updates can legitimately re-show their child meshes.
    for root in &live {
        commands.entity(root).insert(Visibility::Hidden);
    }
    if !config::url_param("gallery").is_some_and(|m| m.eq_ignore_ascii_case("strongholds")) {
        return;
    }
    let own = config::url_param("enemy").is_none();
    let damage = config::url_param("damage")
        .and_then(|d| d.parse::<u8>().ok())
        .unwrap_or(0)
        .min(20);
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
        commands
            .entity(root)
            .insert((tower, super::damage::Effects::default()));
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

/// Opt-in temporal fixture: damage, destruction, then a fresh healthy rematch look.
pub(super) fn wear_sequence(
    mut commands: Commands,
    time: Res<Time>,
    mut art: ResMut<StrongholdArt>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut towers: Query<(Entity, &mut Tower)>,
    mut previous: Local<Option<u8>>,
    mut start: Local<Option<f32>>,
    loading: Res<crate::ui::native_loading::Status>,
) {
    if config::url_param("damage_sweep").is_none() {
        return;
    }
    if !loading.boot_ready || loading.scene_waiting {
        return;
    }
    let start = *start.get_or_insert(time.elapsed_secs());
    let step = ((time.elapsed_secs() - start).max(0.0) / 0.5) as u8;
    let stage = if step > 20 { 0 } else { step };
    if *previous == Some(stage) {
        return;
    }
    for (root, mut tower) in &mut towers {
        if tower.live {
            continue;
        }
        let Some(mut look) = tower.look else {
            continue;
        };
        look.damage = stage;
        if *previous == Some(20) && stage == 0 {
            dress(&mut commands, &mut art, &mut meshes, root, &mut tower, look);
        } else {
            tower.look = Some(look);
        }
    }
    eprintln!(
        "P71 wear fixture: stage={stage}, time={}",
        time.elapsed_secs()
    );
    *previous = Some(stage);
}
