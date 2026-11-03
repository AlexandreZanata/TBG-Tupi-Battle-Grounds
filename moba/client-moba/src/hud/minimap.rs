//! Minimap frame (the minimap camera renders inside it) plus zoom/auto buttons.

use bevy::prelude::*;
use bevy::ui::UiGlobalTransform;

use super::{panel, text, Ctx};
use crate::app::Screen;
use crate::camera::MinimapRect;
use crate::i18n::Key;
use crate::input::Action;
use crate::palette;

/// The square the minimap camera draws into.
#[derive(Component)]
struct Slot;

pub const SIZE: f32 = 262.0;

pub fn plugin(app: &mut App) {
    app.add_systems(
        PostUpdate,
        track
            .after(bevy::ui::UiSystems::Layout)
            .run_if(in_state(Screen::Match)),
    )
    .add_systems(OnExit(Screen::Match), |mut rect: ResMut<MinimapRect>| {
        *rect = MinimapRect::default()
    });
}

pub fn spawn(root: &mut ChildSpawnerCommands, ctx: &Ctx) {
    root.spawn(Node {
        position_type: PositionType::Absolute,
        right: px(10),
        bottom: px(10),
        align_items: AlignItems::FlexEnd,
        column_gap: px(6),
        ..default()
    })
    .with_children(|row| {
        row.spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(6),
            ..default()
        })
        .with_children(|col| {
            for (label, action) in [
                ("+", Action::ZoomIn),
                ("-", Action::ZoomOut),
                ("A", Action::Auto),
                ("o", Action::Center),
            ] {
                col.spawn((
                    panel(
                        Node {
                            width: px(34),
                            height: px(34),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        palette::GOLD_DARK,
                    ),
                    action,
                ))
                .with_children(|b| {
                    b.spawn(text(label, 20.0, palette::TEXT));
                });
            }
            col.spawn(text(ctx.lang.t(Key::Auto), 10.0, palette::MUTED));
        });
        row.spawn(panel(
            Node {
                width: px(SIZE + 12.0),
                height: px(SIZE + 12.0),
                padding: UiRect::all(px(4)),
                ..default()
            },
            palette::GOLD,
        ))
        .with_children(|frame| {
            frame.spawn((
                Node {
                    width: px(SIZE),
                    height: px(SIZE),
                    ..default()
                },
                Slot,
            ));
        });
    });
}

/// Mirrors the slot's laid-out rectangle (logical px) for the camera and clicks.
fn track(
    slot: Single<(&ComputedNode, &UiGlobalTransform), With<Slot>>,
    window: Single<&Window>,
    mut rect: ResMut<MinimapRect>,
) {
    let (node, transform) = *slot;
    let scale = window.scale_factor();
    let size = node.size() / scale;
    let center = transform.translation / scale;
    let new = MinimapRect {
        min: center - size / 2.0,
        size: size.x.min(size.y),
    };
    if (new.min, new.size) != (rect.min, rect.size) {
        *rect = new;
    }
}
