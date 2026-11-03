//! Compact landscape choices retain the original art and a full selected-effect readout.

use super::*;

pub(super) fn card(parent: &mut ChildSpawnerCommands, assets: &AssetServer, i: usize) {
    parent
        .spawn((
            Node {
                flex_grow: 1.,
                flex_basis: px(0),
                min_width: px(0),
                height: px(112),
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
            // Preserve the original frame/illustration proportions rather than stretching art.
            card.spawn(Node {
                position_type: PositionType::Absolute,
                left: px(5),
                top: px(48),
                width: px(42),
                height: px(42. * 1180. / 820.),
                ..default()
            })
            .with_children(|art| {
                art.spawn((
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
                art.spawn((
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
            });
            card.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(4),
                    right: px(4),
                    top: px(6),
                    min_width: px(0),
                    ..default()
                },
                chrome::title("", 11.),
                TextLayout::new(Justify::Left, LineBreak::WordOrCharacter),
                Field::Name(i),
            ));
            card.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(54),
                    right: px(4),
                    bottom: px(8),
                    height: px(24),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    column_gap: px(4),
                    ..default()
                },
                ImageNode {
                    image: assets.load("battle/native/council-price.webp"),
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
            ))
            .with_children(|cost| {
                cost.spawn(kit::image(
                    assets,
                    "battle/native/council-energy.webp",
                    14.,
                    12.,
                ));
                cost.spawn((chrome::title("", 14.), Field::Cost(i)));
            });
        });
}

pub(super) fn details(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn(Node {
            width: percent(100),
            height: px(48),
            flex_shrink: 0.,
            overflow: Overflow::scroll_y(),
            padding: UiRect::all(px(4)),
            ..default()
        })
        .with_children(|detail| {
            detail.spawn((
                Node {
                    width: percent(100),
                    min_width: px(0),
                    flex_shrink: 0.,
                    ..default()
                },
                label("", 13.),
                Field::Selected,
            ));
        });
}
