//! Shared medieval UI tokens, panels, cards and typography for every platform.

pub mod buttons;
pub mod edit;
use bevy::prelude::*;
pub use buttons::plugin;

pub const INK: Color = Color::srgb_u8(14, 22, 29);
pub const PANEL: Color = Color::srgba(0.055, 0.09, 0.12, 0.97);
pub const GOLD: Color = Color::srgb_u8(211, 177, 105);
pub const MUTED: Color = Color::srgb_u8(155, 169, 175);
pub const LINE: Color = Color::srgb_u8(74, 88, 94);
pub const BLUE: Color = Color::srgb_u8(47, 88, 110);

/// A server-backed field has not received its first value; Linux paints its reserved slot.
#[cfg(tbg_native_ui)]
#[derive(Component)]
pub struct Pending(pub bool);

pub fn panel() -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(px(if cfg!(target_os = "android") { 12 } else { 20 })),
            row_gap: px(if cfg!(target_os = "android") { 8 } else { 12 }),
            #[cfg(target_os = "android")]
            flex_shrink: 0.,
            #[cfg(target_os = "android")]
            min_width: px(0),
            #[cfg(target_os = "android")]
            max_width: percent(100),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(10)),
            ..default()
        },
        BackgroundColor(PANEL),
        BorderColor::all(LINE),
        BackgroundGradient::from(LinearGradient::to_bottom(vec![
            ColorStop::auto(Color::srgba(0.12, 0.17, 0.19, 0.85)),
            ColorStop::auto(PANEL),
        ])),
        BoxShadow(vec![ShadowStyle {
            color: Color::BLACK.with_alpha(0.35),
            x_offset: px(0),
            y_offset: px(5),
            spread_radius: px(0),
            blur_radius: px(12),
        }]),
    )
}
pub fn row() -> Node {
    Node {
        align_items: AlignItems::Center,
        column_gap: px(12),
        ..default()
    }
}
pub fn display_font() -> Handle<Font> {
    Handle::Uuid(
        bevy::asset::uuid::Uuid::from_u128(0xb51e3073bfea43f6a0d07b37e366c774),
        std::marker::PhantomData,
    )
}

pub fn heading(text: impl Into<String>, size: f32) -> impl Bundle {
    (
        Text::new(text),
        TextFont::from_font_size(FontSize::Px(if cfg!(target_os = "android") {
            size.min(26.)
        } else {
            size
        }))
        .with_font(display_font()),
        TextColor(GOLD),
    )
}
pub fn muted(text: impl Into<String>, size: f32) -> impl Bundle {
    text_color(
        text,
        if cfg!(target_os = "android") {
            size.min(14.)
        } else {
            size
        },
        MUTED,
    )
}
pub fn image(
    assets: &AssetServer,
    path: impl Into<String>,
    width: f32,
    height: f32,
) -> impl Bundle {
    (
        ImageNode::new(assets.load(path.into())),
        Node {
            width: px(width),
            height: px(height),
            flex_shrink: 0.0,
            ..default()
        },
    )
}
/// Keep native body readouts inside their scrollable column, including unbroken input.
pub fn paragraph(value: impl Into<String>, size: f32) -> impl Bundle {
    (
        muted(value, size),
        #[cfg(target_os = "android")]
        Node {
            width: percent(100),
            min_width: px(0),
            flex_shrink: 0.,
            ..default()
        },
        #[cfg(target_os = "android")]
        TextLayout::new(Justify::Left, LineBreak::WordOrCharacter),
    )
}

pub fn control(text: impl Into<String>, enabled: bool, width: f32) -> impl Bundle {
    (
        Node {
            min_width: px(if cfg!(target_os = "android") {
                48.
            } else {
                width
            }),
            #[cfg(target_os = "android")]
            width: if width > 0. { px(width) } else { Val::Auto },
            #[cfg(target_os = "android")]
            max_width: percent(100),
            min_height: px(if cfg!(target_os = "android") { 48 } else { 42 }),
            padding: UiRect::axes(px(14), px(10)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(5)),
            ..default()
        },
        BackgroundColor(if enabled { BLUE } else { INK }),
        BorderColor::all(LINE),
        buttons::Chrome {
            enabled,
            focused: false,
        },
        children![(
            #[cfg(target_os = "android")]
            Node {
                width: percent(100),
                min_width: px(0),
                ..default()
            },
            super::label(text, 16.),
            #[cfg(target_os = "android")]
            TextLayout::new(Justify::Center, LineBreak::WordOrCharacter),
        )],
    )
}

pub fn button(text: impl Into<String>, action: super::UiAction, width: f32) -> impl Bundle {
    (
        control(text, action != super::UiAction::Disabled, width),
        action,
    )
}

/// Reference-height scaling keeps the desktop lobby readable at 720p and 4K.
pub fn scale(width: f32, height: f32) -> f32 {
    if width < 900.0 {
        (width / 480.0).clamp(0.65, 1.0)
    } else {
        (height / 900.0).clamp(0.7, 2.4)
    }
}

fn text_color(text: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(text),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(color),
    )
}
