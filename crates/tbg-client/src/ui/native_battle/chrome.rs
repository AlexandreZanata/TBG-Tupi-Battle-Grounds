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
