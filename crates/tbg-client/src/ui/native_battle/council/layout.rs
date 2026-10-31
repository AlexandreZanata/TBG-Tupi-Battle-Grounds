//! War Council geometry and image/text layers matching the browser reference.

use super::*;

pub(super) fn spawn(mut commands: Commands, assets: Res<AssetServer>) {
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
                super::super::surrender::linux_button(&assets),
                CouncilControl,
            ));
            backdrop
                .spawn((
                    Node {
                        width: px(1036),
                        max_width: percent(96),
                        padding: UiRect::axes(px(28), px(18)),
                        border: UiRect::all(px(3)),
                        border_radius: BorderRadius::all(px(5)),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(14),
                        ..default()
                    },
                    chrome::frame(Color::srgb_u8(36, 48, 57)),
                ))
                .with_children(|modal| {
                    heading(modal);
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
                            column_gap: px(16),
                            justify_content: JustifyContent::Center,
                            ..default()
                        })
                        .with_children(|row| {
                            for i in 0..4 {
                                card(row, &assets, i);
                            }
                        });
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
                                    "Forgo",
                                    UiAction::ForgoAugment,
                                    Color::srgb_u8(42, 39, 32),
                                ),
                                Forgo,
                            ));
                            actions.spawn((
                                control(
                                    "Confirm choice",
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

fn heading(parent: &mut ChildSpawnerCommands) {
    parent.spawn(Node {width:percent(100),justify_content:JustifyContent::SpaceBetween,column_gap:px(16),..default()}).with_children(|head| {
        head.spawn(Node {flex_direction:FlexDirection::Column,row_gap:px(4),..default()}).with_children(|title| {
            title.spawn(kit::heading("BATTLE PAUSED",9.));
            title.spawn(chrome::title("Choose an Augment",32.));
            title.spawn(kit::muted("Both players choose one Augment from the same four. The battle resumes when both confirm or time runs out.",12.));
        });
        head.spawn(Node {flex_direction:FlexDirection::Column,align_items:AlignItems::End,..default()}).with_children(|clock| {
            clock.spawn(kit::muted("TIME LEFT",9.));
            clock.spawn((Text::new("0:20"),TextFont::from_font_size(FontSize::Px(30.)),TextColor(Color::srgb_u8(200,240,160)),Field::Time));
        });
    });
}

fn control(text: &str, action: UiAction, idle: Color) -> impl Bundle {
    (
        Node {
            min_width: px(156),
            min_height: px(41),
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
            BoxShadow::default(),
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
            card.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: percent(59.5),
                    left: percent(12),
                    right: percent(12),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                kit::heading("", 11.),
                Field::Rarity(i),
                ZIndex(3),
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
                    TextLayout::justify(Justify::Center),
                    Field::Description(i),
                ));
            });
            card.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: percent(88.2),
                    height: percent(5),
                    width: percent(100),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    column_gap: px(4),
                    ..default()
                },
                Layout::Price(i),
                ZIndex(3),
            ))
            .with_children(|cost| {
                cost.spawn(kit::image(
                    assets,
                    "battle/native/council-energy.webp",
                    14.,
                    13.,
                ));
                cost.spawn((label("", 14.), Field::Cost(i)));
            });
        });
}
