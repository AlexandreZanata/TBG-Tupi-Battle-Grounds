//! In-match HUD: clock, Stronghold HP, Energy, army size, synergies,
//! and the five shop cards. Reads only `LatestView`.
//! Linux presentation lives in `native_battle`; other targets keep `hud_top`.

use bevy::prelude::*;

use super::{label, screen, UiAction, BUTTON};
use crate::sync::MatchInfo;
use crate::GameState;

#[derive(Component)]
pub(super) enum Hud {
    #[cfg(not(target_os = "linux"))]
    Clock,
    Hp(usize),
    #[cfg(not(target_os = "linux"))]
    HpLabel(usize),
    Spectators,
    PrivateControls,
    WatchControls,
    Portrait(usize),
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
            (super::hud_update::update, super::hud_portraits::update)
                .run_if(in_state(GameState::InMatch)),
        );
}

#[cfg(not(target_os = "linux"))]
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

fn card(i: usize, narrow: bool) -> impl Bundle {
    (
        Node {
            width: px(if narrow {
                82.0
            } else if cfg!(target_os = "linux") {
                166.0
            } else {
                154.0
            }),
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
            (
                ImageNode::default(),
                Node {
                    width: percent(100),
                    height: px(if narrow {
                        52
                    } else if cfg!(target_os = "linux") {
                        145
                    } else {
                        76
                    }),
                    ..default()
                },
                Hud::Portrait(i)
            ),
            (
                label("", if narrow { 11.0 } else { 15.0 }),
                Hud::CardText(i)
            ),
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

#[cfg(not(target_os = "linux"))]
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
                    Hud::PrivateControls,
                    BackgroundColor(super::kit::PANEL),
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
                        children![
                            card(0, narrow),
                            card(1, narrow),
                            card(2, narrow),
                            card(3, narrow),
                            card(4, narrow)
                        ]
                    ),
                    super::surrender::button(),
                ],
            ));
            root.spawn((
                Node {
                    display: Display::None,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: px(8),
                    ..default()
                },
                Hud::WatchControls,
                children![
                    (label("", 16.0), Hud::Spectators),
                    super::kit::button("Leave battle", UiAction::LeaveWatch, 220.0)
                ],
            ));
        });
}

#[cfg(target_os = "linux")]
fn spawn(mut commands: Commands, info: Res<MatchInfo>, assets: Res<AssetServer>) {
    commands
        .spawn((
            screen(JustifyContent::SpaceBetween),
            HudRoot,
            DespawnOnExit(GameState::InMatch),
        ))
        .with_children(|root| {
            super::native_battle::score(root, &info, &assets);
            if !info.spectating {
                root.spawn(super::surrender::linux_button(&assets));
            }
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: px(36),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: px(6),
                    ..default()
                },
                Hud::PrivateControls,
            ))
            .with_children(|shop| {
                shop.spawn((label("", 11.), Hud::Augments));
                shop.spawn((label("", 11.), Hud::Info));
                shop.spawn((label("", 12.), Hud::CouncilStatus));
                shop.spawn((
                    Node {
                        display: Display::None,
                        flex_wrap: FlexWrap::Wrap,
                        justify_content: JustifyContent::Center,
                        row_gap: px(6),
                        ..row(6.)
                    },
                    Hud::CouncilRow,
                    children![
                        council_card(0),
                        council_card(1),
                        council_card(2),
                        council_card(3),
                        forgo_card()
                    ],
                ));
                shop.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(10),
                    ..default()
                })
                .with_children(|bottom| {
                    super::native_battle::resources(bottom, &assets);
                    bottom.spawn((
                        row(8.),
                        children![
                            card(0, false),
                            card(1, false),
                            card(2, false),
                            card(3, false),
                            card(4, false)
                        ],
                    ));
                });
            });
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: px(36),
                    display: Display::None,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: px(8),
                    ..default()
                },
                Hud::WatchControls,
                children![
                    (label("", 16.), Hud::Spectators),
                    super::kit::button("Leave battle", UiAction::LeaveWatch, 220.)
                ],
            ));
        });
}
