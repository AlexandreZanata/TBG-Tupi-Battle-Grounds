//! In-match HUD: clock, Stronghold HP, Energy, army size, synergies,
//! and the five shop cards. Reads only `LatestView`.
//! Linux presentation lives in `native_battle`; other targets keep `hud_top`.

use bevy::prelude::*;

#[cfg(not(tbg_native_ui))]
use super::BUTTON;
use super::{label, screen, UiAction};
use crate::sync::MatchInfo;
use crate::GameState;

#[derive(Component)]
pub(super) enum Hud {
    #[cfg(not(tbg_native_ui))]
    Clock,
    Hp(usize),
    #[cfg(not(tbg_native_ui))]
    HpLabel(usize),
    Spectators,
    PrivateControls,
    WatchControls,
    Portrait(usize),
    EnergyFill,
    Info,
    #[cfg(not(tbg_native_ui))]
    Card(usize),
    #[cfg(not(tbg_native_ui))]
    CardText(usize),
    #[cfg(not(tbg_native_ui))]
    CardStrip(usize),
    Augments,
    #[cfg(not(tbg_native_ui))]
    CouncilStatus,
    #[cfg(not(tbg_native_ui))]
    CouncilRow,
    #[cfg(not(tbg_native_ui))]
    CouncilOption(usize),
    #[cfg(not(tbg_native_ui))]
    CouncilOptionText(usize),
    #[cfg(not(tbg_native_ui))]
    Forgo,
    #[cfg(not(tbg_native_ui))]
    ForgoText,
}

#[cfg(not(tbg_native_ui))]
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

#[cfg(not(tbg_native_ui))]
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
        children![(label("", 12.0), Hud::ForgoText)],
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

#[cfg(not(tbg_native_ui))]
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

#[cfg(not(tbg_native_ui))]
fn card(i: usize, narrow: bool) -> impl Bundle {
    (
        Node {
            width: px(if narrow {
                82.0
            } else if cfg!(tbg_native_ui) {
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
                    } else if cfg!(tbg_native_ui) {
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

#[cfg(not(tbg_native_ui))]
fn row(gap: f32) -> Node {
    Node {
        flex_direction: FlexDirection::Row,
        column_gap: px(gap),
        align_items: AlignItems::Center,
        ..default()
    }
}

#[cfg(not(tbg_native_ui))]
fn spawn(
    mut commands: Commands,
    info: Res<MatchInfo>,
    lang: Res<crate::i18n::Lang>,
    window: Single<&Window>,
) {
    let narrow = window.width() < 600.0;
    commands
        .spawn((
            screen(JustifyContent::SpaceBetween),
            HudRoot,
            DespawnOnExit(GameState::InMatch),
        ))
        .with_children(|root| {
            super::hud_top::spawn(root, &info, *lang, narrow);
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
                    super::kit::button(
                        lang.key("training.leave_battle"),
                        UiAction::LeaveWatch,
                        220.0
                    )
                ],
            ));
        });
}

#[cfg(tbg_native_ui)]
fn spawn(
    mut commands: Commands,
    info: Res<MatchInfo>,
    assets: Res<AssetServer>,
    lang: Res<crate::i18n::Lang>,
) {
    commands
        .spawn((
            screen(JustifyContent::SpaceBetween),
            HudRoot,
            DespawnOnExit(GameState::InMatch),
        ))
        .with_children(|root| {
            super::native_battle::score(root, &info, &assets, *lang);
            if !info.spectating {
                root.spawn(super::surrender::linux_button(&assets, *lang));
            }
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: px(super::native_battle::SHOP_BOTTOM),
                    left: if cfg!(target_os = "android") {
                        px(12)
                    } else {
                        Val::Auto
                    },
                    right: if cfg!(target_os = "android") {
                        px(12)
                    } else {
                        Val::Auto
                    },
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
                shop.spawn((
                    Node {
                        width: if cfg!(target_os = "android") {
                            percent(100)
                        } else {
                            px(862)
                        },
                        flex_direction: FlexDirection::Column,
                        row_gap: px(10),
                        ..default()
                    },
                    UiTransform {
                        scale: Vec2::splat(super::native_battle::SHOP_SCALE),
                        translation: Val2::percent(
                            0.,
                            (1. - super::native_battle::SHOP_SCALE) * 50.,
                        ),
                        ..default()
                    },
                ))
                .with_children(|bottom| {
                    super::native_battle::resources(bottom, &assets);
                    super::native_battle::cards(bottom, &assets);
                    // Desktop auxiliaries share the Shop scale; the mobile drawer covers the safe frame.
                    #[cfg(not(target_os = "android"))]
                    super::native_battle::auxiliaries(bottom, &assets);
                });
            });
            #[cfg(target_os = "android")]
            super::native_battle::auxiliaries(root, &assets);
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
                    super::kit::button(
                        lang.key("training.leave_battle"),
                        UiAction::LeaveWatch,
                        220.
                    )
                ],
            ));
        });
}
