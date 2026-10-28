//! In-match HUD: clock, Stronghold HP, Energy, army slots, synergies,
//! rotation timer, and the three shop cards. Reads only `LatestView`.
//! The top line (names, bars, clock) lives in `hud_top`.

use bevy::prelude::*;

use super::{label, screen, UiAction, BUTTON};
use crate::sync::MatchInfo;
use crate::GameState;

#[derive(Component)]
pub(super) enum Hud {
    Clock,
    Hp(usize),
    EnergyFill,
    Info,
    Rotation,
    Card(usize),
    CardText(usize),
    CardStrip(usize),
}

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::InMatch), spawn)
        .add_systems(
            Update,
            super::hud_update::update.run_if(in_state(GameState::InMatch)),
        );
}

pub(super) fn bar(fill: Hud, color: Color, width: f32, height: f32) -> impl Bundle {
    (
        Node {
            width: px(width),
            height: px(height),
            border_radius: BorderRadius::all(px(4.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
        children![(
            Node {
                width: percent(100.0),
                height: percent(100.0),
                border_radius: BorderRadius::all(px(4.0)),
                ..default()
            },
            BackgroundColor(color),
            fill,
        )],
    )
}

fn card(i: usize) -> impl Bundle {
    (
        Node {
            width: px(112.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(px(6.0)),
            row_gap: px(4.0),
            border_radius: BorderRadius::all(px(10.0)),
            ..default()
        },
        BackgroundColor(BUTTON),
        UiAction::Buy(i as u8),
        Hud::Card(i),
        children![
            (
                Node {
                    height: px(6.0),
                    width: percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::WHITE),
                Hud::CardStrip(i)
            ),
            (label("", 15.0), Hud::CardText(i)),
        ],
    )
}

fn row(gap: f32) -> Node {
    Node {
        flex_direction: FlexDirection::Row,
        column_gap: px(gap),
        align_items: AlignItems::Center,
        ..default()
    }
}

fn spawn(mut commands: Commands, info: Res<MatchInfo>, window: Single<&Window>) {
    let narrow = window.width() < 600.0;
    commands
        .spawn((
            screen(JustifyContent::SpaceBetween),
            DespawnOnExit(GameState::InMatch),
        ))
        .with_children(|root| {
            super::hud_top::spawn(root, &info, narrow);
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: px(6.0),
                    ..default()
                },
                children![
                    (label("", 15.0), Hud::Info),
                    bar(
                        Hud::EnergyFill,
                        Color::srgb_u8(0xB4, 0x7C, 0xFF),
                        352.0,
                        14.0
                    ),
                    bar(Hud::Rotation, Color::srgb_u8(0xF4, 0xEE, 0xE0), 352.0, 4.0),
                    (row(8.0), children![card(0), card(1), card(2)]),
                ],
            ));
        });
}
