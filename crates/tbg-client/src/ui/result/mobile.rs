//! Compact full-width Result actions preserve safe touch sizes and space for the celebration.

use crate::ui::{kit, label as base_label, UiAction};
use bevy::prelude::*;

pub(super) fn button(text: &str, action: UiAction, _desktop_width: f32) -> impl Bundle + use<> {
    (
        Node {
            flex_grow: 1.,
            flex_basis: px(0),
            min_width: px(48),
            min_height: px(48),
            padding: UiRect::all(px(4)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(5)),
            ..default()
        },
        BackgroundColor(kit::BLUE),
        BorderColor::all(kit::LINE),
        kit::buttons::Chrome {
            enabled: true,
            focused: false,
        },
        action,
        children![(
            Node {
                width: percent(100),
                min_width: px(0),
                ..default()
            },
            base_label(text.to_owned(), 11.),
            TextLayout::new(Justify::Center, LineBreak::WordOrCharacter),
        )],
    )
}

pub(super) fn label(text: impl Into<String>, size: f32) -> impl Bundle {
    (
        base_label(text, size),
        Node {
            width: percent(100),
            flex_shrink: 0.,
            ..default()
        },
        TextLayout::justify(Justify::Center),
    )
}
