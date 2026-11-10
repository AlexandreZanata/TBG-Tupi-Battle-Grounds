//! The 1v1 battle HUD chrome: metal frames, gradients, display titles and
//! icons, copied from `crates/tbg-client/src/ui/native_battle/chrome.rs` so
//! both modes share one visual language.

use bevy::prelude::*;

use crate::i18n::Fonts;

pub const GOLD: Color = Color::srgb_u8(195, 164, 106);
pub const CREAM: Color = Color::srgb_u8(246, 232, 200);
pub const INK: Color = Color::srgb_u8(11, 20, 29);
pub const MUTED: Color = Color::srgb_u8(155, 169, 175);

pub fn gradient(top: Color, bottom: Color) -> BackgroundGradient {
    BackgroundGradient::from(LinearGradient::to_bottom(vec![
        ColorStop::auto(top),
        ColorStop::auto(bottom),
    ]))
}

/// The dark framed plate of every 1v1 battle panel.
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

pub fn title(text: impl Into<String>, size: f32, fonts: &Fonts) -> impl Bundle {
    (
        Text::new(text),
        TextFont::from_font_size(FontSize::Px(size)).with_font(fonts.display.clone()),
        TextColor(CREAM),
        TextShadow {
            offset: Vec2::new(0., 1.),
            color: Color::BLACK,
        },
    )
}

pub fn muted(text: impl Into<String>, size: f32) -> impl Bundle {
    (
        Text::new(text),
        TextFont::from_font_size(FontSize::Px(size)),
        TextColor(MUTED),
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

/// The round gold coin drawn beside every price.
pub fn coin(size: f32) -> impl Bundle {
    (
        Node {
            width: px(size),
            height: px(size),
            border: UiRect::all(px(2)),
            border_radius: BorderRadius::all(percent(50)),
            flex_shrink: 0.,
            ..default()
        },
        BorderColor::all(Color::srgb_u8(255, 228, 162)),
        gradient(Color::srgb_u8(255, 233, 137), Color::srgb_u8(181, 115, 13)),
    )
}

pub fn divider(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        Node {
            width: px(1),
            height: px(26),
            ..default()
        },
        BackgroundColor(Color::WHITE.with_alpha(0.15)),
    ));
}

/// The gold-edged dark button of the battle HUD (Reroll, locks).
pub fn button(node: Node) -> impl Bundle {
    (
        Node {
            border: UiRect::all(px(2)),
            border_radius: BorderRadius::all(px(6)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..node
        },
        BackgroundColor(INK),
        BorderColor::all(GOLD),
        crate::input::Blocks,
    )
}

/// Portrait of a 1v1 champion, as the 1v1 Shop shows it.
pub fn portrait(assets: &AssetServer, kind: tbg_moba_core::Champion) -> Handle<Image> {
    assets.load(format!(
        "battle/portraits/{}.webp",
        format!("{kind:?}").to_ascii_lowercase()
    ))
}
