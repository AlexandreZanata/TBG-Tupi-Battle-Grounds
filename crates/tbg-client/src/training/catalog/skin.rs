//! Laboratory controls share pointer, keyboard focus and persistent selected styling.

use super::*;

pub(super) fn colors(selected: bool) -> impl Bundle {
    let idle = if selected {
        Color::srgb_u8(23, 69, 91)
    } else {
        Color::srgb_u8(14, 35, 48)
    };
    let edge = if selected { kit::GOLD } else { kit::LINE };
    (
        BackgroundColor(idle),
        BorderColor::all(edge),
        kit::buttons::Chrome {
            enabled: true,
            focused: selected,
        },
        kit::buttons::Palette {
            idle,
            hover: Color::srgb_u8(39, 85, 105),
            pressed: kit::BLUE,
            border: edge,
        },
    )
}
pub(super) fn control(
    text: impl Into<String>,
    action: Action,
    width: f32,
    selected: bool,
) -> impl Bundle {
    (
        Node {
            width: if width == 0. { percent(100) } else { px(width) },
            min_width: px(0),
            min_height: px(38),
            flex_shrink: 0.,
            padding: UiRect::axes(px(10), px(8)),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(4)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        colors(selected),
        crate::ui::UiAction::Disabled,
        shell::Action::Catalog(action),
        children![crate::ui::label(text, 14.)],
    )
}
pub(super) fn primary() -> impl Bundle {
    let gold = Color::srgb_u8(132, 96, 39);
    (
        BackgroundColor(gold),
        kit::buttons::Palette {
            idle: gold,
            hover: Color::srgb_u8(162, 120, 51),
            pressed: Color::srgb_u8(100, 70, 29),
            border: kit::GOLD,
        },
    )
}
pub(super) fn panel(width: f32) -> impl Bundle {
    (
        Node {
            width: percent(width),
            min_width: px(0),
            min_height: px(0),
            padding: UiRect::all(px(15)),
            border: UiRect::all(px(2)),
            border_radius: BorderRadius::all(px(6)),
            flex_direction: FlexDirection::Column,
            row_gap: px(10),
            overflow: Overflow::scroll_y(),
            ..default()
        },
        BackgroundColor(Color::srgb_u8(8, 24, 34)),
        BorderColor::all(kit::GOLD),
        BoxShadow(vec![ShadowStyle {
            color: Color::BLACK.with_alpha(0.4),
            y_offset: px(5),
            blur_radius: px(15),
            ..default()
        }]),
    )
}
