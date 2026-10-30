//! In-match HUD: clock, Stronghold HP, Energy, army size, synergies,
//! and the five shop cards. Reads only `LatestView`.
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
    Card(usize),
    CardText(usize),
    CardStrip(usize),
    Augments,
    CouncilStatus,
    CouncilRow,
    CouncilOption(usize),
    CouncilOptionText(usize),
    Forgo,
    ForgoText,
}

fn council_card(i: usize) -> impl Bundle {
    (
        Node {
            width: px(150.0),
            min_height: px(64.0),
            padding: UiRect::all(px(5.0)),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(BUTTON),
        UiAction::ChooseAugment(i as u8),
        Hud::CouncilOption(i),
        children![(label("", 11.0), Hud::CouncilOptionText(i))],
    )
}

fn forgo_card() -> impl Bundle {
    (
        Node {
            width: px(86.0),
            min_height: px(46.0),
            padding: UiRect::all(px(4.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(BUTTON),
        UiAction::ForgoAugment,
        Hud::Forgo,
        children![(label("Forgo", 12.0), Hud::ForgoText)],
    )
}

#[derive(Component)]
pub(super) struct HudRoot;

pub fn plugin(app: &mut App) {
    super::countdown::plugin(app);
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
            HudRoot,
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
                    (label("", 13.0), Hud::Augments),
                    (label("", 15.0), Hud::Info),
                    (label("", 12.0), Hud::CouncilStatus),
                    (
                        Node {
                            display: Display::None,
                            flex_wrap: FlexWrap::Wrap,
                            justify_content: JustifyContent::Center,
                            row_gap: px(6.0),
                            ..row(6.0)
                        },
                        Hud::CouncilRow,
                        children![
                            council_card(0),
                            council_card(1),
                            council_card(2),
                            council_card(3),
                            forgo_card()
                        ]
                    ),
                    super::fortify::control(),
                    bar(
                        Hud::EnergyFill,
                        Color::srgb_u8(0xB4, 0x7C, 0xFF),
                        352.0,
                        14.0
                    ),
                    (
                        row(8.0),
                        children![card(0), card(1), card(2), card(3), card(4)]
                    ),
                    super::surrender::button(),
                ],
            ));
        });
}
