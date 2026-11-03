//! War Council geometry and image/text layers matching the browser reference.

use super::*;

#[cfg(target_os = "android")]
mod mobile;
#[cfg(target_os = "android")]
use mobile::card;

pub(super) fn spawn(
    mut commands: Commands,
    assets: Res<AssetServer>,
    lang: Res<crate::i18n::Lang>,
) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.04, 0.07, 0.77)),
            Root,
            Visibility::Hidden,
            UiAction::Disabled,
            ZIndex(30),
            DespawnOnExit(GameState::InMatch),
        ))
        .with_children(|backdrop| {
            // Keep the existing two-step surrender available during the pause (BR-M06).
            backdrop.spawn((
                crate::ui::surrender::linux_button(&assets, *lang),
                CouncilControl,
            ));
            backdrop
                .spawn((
                    Node {
                        width: px(1036),
                        max_width: percent(96),
                        max_height: if cfg!(target_os = "android") {
                            percent(96)
                        } else {
                            Val::Auto
                        },
                        padding: UiRect::axes(
                            px(if cfg!(target_os = "android") { 12 } else { 28 }),
                            px(if cfg!(target_os = "android") { 8 } else { 18 }),
                        ),
                        border: UiRect::all(px(3)),
                        border_radius: BorderRadius::all(px(5)),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(if cfg!(target_os = "android") { 6 } else { 14 }),
                        ..default()
                    },
                    chrome::frame(Color::srgb_u8(36, 48, 57)),
                ))
                .with_children(|modal| {
                    heading(modal, *lang);
                    modal
                        .spawn((
                            Node {
                                width: percent(100),
                                height: px(4),
                                border_radius: BorderRadius::all(px(3)),
                                overflow: Overflow::clip(),
                                ..default()
                            },
                            BackgroundColor(Color::srgb_u8(2, 11, 18)),
                        ))
                        .with_children(|track| {
                            track.spawn((
                                Node {
                                    width: percent(100),
                                    height: percent(100),
                                    ..default()
                                },
                                chrome::gradient(
                                    Color::srgb_u8(142, 111, 64),
                                    Color::srgb_u8(244, 209, 139),
                                ),
                                Layout::Timer,
                            ));
                        });
                    modal
                        .spawn(Node {
                            width: percent(100),
                            column_gap: px(if cfg!(target_os = "android") { 8 } else { 16 }),
                            justify_content: JustifyContent::Center,
                            ..default()
                        })
                        .with_children(|row| {
                            for i in 0..4 {
                                card(row, &assets, i);
                            }
                        });
                    #[cfg(target_os = "android")]
                    mobile::details(modal);
                    modal
                        .spawn((
                            Node {
                                width: percent(100),
                                justify_content: JustifyContent::Center,
                                column_gap: px(14),
                                ..default()
                            },
                            Layout::Actions,
                        ))
                        .with_children(|actions| {
                            actions.spawn((
                                control(
                                    &lang.key("council.forgo"),
                                    UiAction::ForgoAugment,
                                    Color::srgb_u8(42, 39, 32),
                                ),
                                Forgo,
                            ));
                            actions.spawn((
                                control(
                                    &lang.key("council.confirm"),
                                    UiAction::Disabled,
                                    Color::srgb_u8(54, 76, 44),
                                ),
                                Confirm,
                            ));
                        });
                    modal.spawn((
                        Node {
                            width: percent(100),
                            min_height: px(42),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        Layout::Waiting,
                        children![(chrome::title("", 18.), Field::Waiting)],
                    ));
                });
        });
}

fn heading(parent: &mut ChildSpawnerCommands, lang: crate::i18n::Lang) {
    parent
        .spawn(Node {
            width: percent(100),
            justify_content: JustifyContent::SpaceBetween,
            column_gap: px(16),
            ..default()
        })
        .with_children(|head| {
            head.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(4),
                ..default()
            })
            .with_children(|title| {
                title.spawn(kit::heading(lang.key("council.battle_paused"), 9.));
                title.spawn(chrome::title(
                    lang.key("council.title"),
                    if cfg!(target_os = "android") {
                        20.
                    } else {
                        32.
                    },
                ));
                if !cfg!(target_os = "android") {
                    title.spawn(kit::muted(lang.key("council.help"), 12.));
                }
            });
            head.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::End,
                ..default()
            })
            .with_children(|clock| {
                clock.spawn(kit::muted(lang.key("council.time_left"), 9.));
                clock.spawn((
                    Text::new("0:20"),
                    TextFont::from_font_size(FontSize::Px(30.)),
                    TextColor(Color::srgb_u8(200, 240, 160)),
                    Field::Time,
                ));
            });
        });
}

fn control(text: &str, action: UiAction, idle: Color) -> impl Bundle {
    (
        Node {
            min_width: px(156),
            min_height: px(if cfg!(target_os = "android") { 48 } else { 41 }),
            padding: UiRect::axes(px(18), px(9)),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(4)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(idle),
        BorderColor::all(chrome::GOLD),
        action,
        CouncilControl,
        kit::buttons::Chrome {
            enabled: action != UiAction::Disabled,
            focused: false,
        },
        kit::buttons::Palette {
            idle,
            hover: Color::srgb_u8(74, 84, 60),
            pressed: Color::srgb_u8(32, 43, 32),
            border: chrome::GOLD,
        },
        children![label(text.to_owned(), 13.)],
    )
}

#[cfg(not(target_os = "android"))]
fn card(parent: &mut ChildSpawnerCommands, assets: &AssetServer, i: usize) {
    parent
        .spawn((
            Node {
                width: px(CARD_W),
                height: px(CARD_H),
                flex_shrink: 0.,
                border: UiRect::all(px(2)),
                border_radius: BorderRadius::all(px(7)),
                ..default()
            },
            BackgroundColor(chrome::INK),
            BorderColor::all(Color::NONE),
            OptionCard(i),
            UiAction::Disabled,
            CouncilControl,
            kit::buttons::Chrome {
                enabled: false,
                focused: false,
            },
            kit::buttons::Palette {
                idle: chrome::INK,
                hover: chrome::INK,
                pressed: chrome::INK,
                border: Color::NONE,
            },
        ))
        .with_children(|card| {
            card.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: percent(10),
                    top: percent(5),
                    width: percent(80),
                    height: percent(45),
                    ..default()
                },
                ImageNode {
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
                Picture::Illustration(i),
            ));
            card.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: percent(100),
                    height: percent(100),
                    ..default()
                },
                ImageNode {
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
                Picture::Frame(i),
                ZIndex(2),
            ));
            card.spawn(Node {
                position_type: PositionType::Absolute,
                top: percent(64),
                left: percent(12),
                right: percent(12),
                height: percent(8),
                justify_content: JustifyContent::Center,
                overflow: Overflow::clip(),
                ..default()
            })
            .insert(ZIndex(3))
            .with_children(|title| {
                title.spawn((
                    chrome::title("", 18.),
                    TextLayout::justify(Justify::Center),
                    Field::Name(i),
                ));
            });
            card.spawn(Node {
                position_type: PositionType::Absolute,
                top: percent(71),
                bottom: percent(14),
                left: percent(12),
                right: percent(12),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                overflow: Overflow::clip(),
                ..default()
            })
            .insert(ZIndex(3))
            .with_children(|body| {
                body.spawn((
                    label("", 12.),
                    Node {
                        width: percent(100),
                        min_width: px(0),
                        ..default()
                    },
                    TextLayout::justify(Justify::Center),
                    Field::Description(i),
                ));
            });
            card.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: percent(85.5),
                    left: percent(50),
                    margin: UiRect::left(px(-68)),
                    height: px(36),
                    width: px(136),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ImageNode {
                    image: assets.load("battle/native/council-price.webp"),
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
                ZIndex(4),
            ))
            .with_children(|cost| {
                cost.spawn(Node {
                    column_gap: px(6),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|price| {
                    price.spawn(kit::image(
                        assets,
                        "battle/native/council-energy.webp",
                        18.,
                        15.,
                    ));
                    price.spawn((
                        Node {
                            width: px(26),
                            ..default()
                        },
                        chrome::title("", 17.),
                        TextLayout::justify(Justify::Center),
                        Field::Cost(i),
                    ));
                });
            });
        });
}
