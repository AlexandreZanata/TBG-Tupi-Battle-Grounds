//! Metal frames, gradients and heraldic details shared by the Linux battle HUD.

use super::super::kit;
use bevy::prelude::*;

pub const GOLD: Color = Color::srgb_u8(195, 164, 106);
pub const CREAM: Color = Color::srgb_u8(246, 232, 200);
pub const INK: Color = Color::srgb_u8(11, 20, 29);

pub fn gradient(top: Color, bottom: Color) -> BackgroundGradient {
    BackgroundGradient::from(LinearGradient::to_bottom(vec![
        ColorStop::auto(top),
        ColorStop::auto(bottom),
    ]))
}

pub fn frame(edge: Color) -> impl Bundle {
    (
        BackgroundColor(INK),
        BorderColor::all(edge),
        gradient(
            Color::srgba(0.13, 0.20, 0.23, 0.95),
            Color::srgba(0.04, 0.07, 0.09, 0.98),
        ),
        BoxShadow(vec![ShadowStyle {
            color: Color::BLACK.with_alpha(0.55),
            y_offset: px(4),
            blur_radius: px(9),
            ..default()
        }]),
    )
}

pub fn title(text: impl Into<String>, size: f32) -> impl Bundle {
    (
        Text::new(text),
        TextFont::from_font_size(FontSize::Px(size)).with_font(kit::display_font()),
        TextColor(CREAM),
        TextShadow {
            offset: Vec2::new(0., 1.),
            color: Color::BLACK,
        },
    )
}

pub fn icon(assets: &AssetServer, name: &str, size: f32, color: Color) -> impl Bundle {
    (
        Node {
            width: px(size),
            height: px(size),
            flex_shrink: 0.,
            ..default()
        },
        ImageNode {
            color,
            ..ImageNode::new(assets.load(format!("lobby/native-icons/{name}.webp")))
        },
    )
}

/// Shared top-center slot used by arena-sized Evolution purchase cards.
pub(super) fn upgrade_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        top: px(134),
        left: percent(50),
        margin: UiRect::left(px(-195)),
        width: px(390),
        min_height: px(106),
        padding: UiRect::axes(px(12), px(10)),
        border: UiRect::all(px(1)),
        border_radius: BorderRadius::all(px(10)),
        flex_direction: FlexDirection::Column,
        row_gap: px(6),
        ..default()
    }
}

/// Identical Evolution/Fortification heading geometry, including the close affordance.
pub(super) fn upgrade_header() -> Node {
    Node {
        width: percent(100),
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Center,
        ..default()
    }
}
pub(super) fn upgrade_close(action: super::super::UiAction) -> impl Bundle {
    (
        Node {
            width: px(20),
            height: px(20),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        action,
        children![kit::heading("×", 14.)],
    )
}

/// Equal flex shares account for the gap; a lone second-tier choice fills the row.
pub(super) fn upgrade_choice() -> impl Bundle {
    (
        Node {
            flex_grow: 1.,
            flex_basis: px(0),
            min_width: px(0),
            min_height: px(55),
            padding: UiRect::all(px(7)),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(5)),
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(INK),
        BorderColor::all(GOLD),
        super::super::UiAction::Disabled,
        kit::buttons::Chrome {
            enabled: false,
            focused: false,
        },
        kit::buttons::Palette {
            idle: INK,
            hover: Color::srgb_u8(45, 57, 60),
            pressed: Color::srgb_u8(28, 38, 43),
            border: GOLD,
        },
    )
}
