//! Lobby and shared menu-header skins using the common input/focus pipeline.

use super::kit;
use bevy::prelude::*;

pub const EDGE: Color = Color::srgb_u8(151, 118, 65);
pub const METAL: Color = Color::srgba(0.035, 0.075, 0.09, 0.94);
pub const WOOD: Color = Color::srgb_u8(48, 36, 26);

pub fn region(x: f32, y: f32, w: f32, h: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: percent(x),
        top: percent(y),
        width: percent(w),
        height: percent(h),
        min_width: px(0),
        min_height: px(0),
        ..default()
    }
}

pub fn panel_node(rect: [f32; 4]) -> Node {
    let [x, y, w, h] = rect;
    Node {
        flex_direction: FlexDirection::Column,
        padding: UiRect::all(px(12)),
        border: UiRect::all(px(2)),
        border_radius: BorderRadius::all(px(4)),
        overflow: Overflow::clip(),
        ..region(x, y, w, h)
    }
}

pub fn panel(rect: [f32; 4]) -> impl Bundle {
    (
        panel_node(rect),
        BackgroundColor(METAL),
        BorderColor::all(EDGE),
        BoxShadow(vec![ShadowStyle {
            color: Color::BLACK.with_alpha(0.5),
            y_offset: px(5),
            blur_radius: px(12),
            ..default()
        }]),
    )
}

pub fn skin(color: Color, enabled: bool, active: bool) -> impl Bundle {
    (
        BackgroundColor(color),
        BorderColor::all(EDGE),
        kit::buttons::Chrome {
            enabled,
            focused: active,
        },
        kit::buttons::Palette {
            idle: color,
            hover: Color::srgb_u8(67, 65, 48),
            pressed: Color::srgb_u8(33, 43, 45),
            border: EDGE,
        },
    )
}

pub fn icon(assets: &AssetServer, name: &str, size: f32, color: Color) -> impl Bundle {
    (
        ImageNode {
            color,
            ..ImageNode::new(assets.load(format!("lobby/native-icons/{name}.webp")))
        },
        Node {
            width: px(size),
            height: px(size),
            flex_shrink: 0.,
            ..default()
        },
    )
}

pub fn column() -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        min_width: px(0),
        ..default()
    }
}

pub fn image(assets: &AssetServer, path: &str, height: f32) -> impl Bundle {
    (
        ImageNode::new(assets.load(path.to_string())),
        Node {
            width: percent(100),
            height: percent(height),
            min_height: px(0),
            flex_shrink: 0.,
            ..default()
        },
    )
}
