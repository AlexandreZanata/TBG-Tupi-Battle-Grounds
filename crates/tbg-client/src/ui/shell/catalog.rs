//! Linux catalog geometry shared by Arena, Stronghold and Arcana banner presentation.

use super::{kit, lobby::chrome};
use bevy::prelude::*;

pub(super) fn grid() -> Node {
    Node {
        display: Display::Grid,
        width: percent(100),
        min_width: px(0),
        flex_shrink: 0.,
        grid_template_columns: RepeatedGridTrack::flex(4, 1.),
        column_gap: px(16),
        row_gap: px(16),
        ..default()
    }
}

pub(super) fn card() -> impl Bundle {
    (
        Node {
            min_width: px(0),
            flex_direction: FlexDirection::Column,
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(7)),
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(kit::PANEL),
        BorderColor::all(chrome::EDGE),
        BackgroundGradient::from(LinearGradient::to_bottom(vec![
            ColorStop::auto(Color::srgb_u8(31, 47, 53)),
            ColorStop::auto(kit::PANEL),
        ])),
    )
}

pub(super) fn body() -> Node {
    Node {
        width: percent(100),
        min_width: px(0),
        flex_grow: 1.,
        flex_direction: FlexDirection::Column,
        padding: UiRect::all(px(16)),
        row_gap: px(10),
        ..default()
    }
}

pub(super) fn banner(assets: &AssetServer, path: String, ratio: f32) -> impl Bundle {
    (
        ImageNode {
            image_mode: NodeImageMode::Stretch,
            ..ImageNode::new(assets.load(path))
        },
        Node {
            width: percent(100),
            aspect_ratio: Some(ratio),
            min_width: px(0),
            flex_shrink: 0.,
            ..default()
        },
    )
}

pub(super) fn button_node() -> Node {
    Node {
        width: percent(100),
        min_width: px(0),
        min_height: px(42),
        flex_shrink: 0.,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        padding: UiRect::axes(px(10), px(9)),
        border: UiRect::all(px(1)),
        border_radius: BorderRadius::all(px(4)),
        ..default()
    }
}
