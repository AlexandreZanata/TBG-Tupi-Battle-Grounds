//! Minimap frame (the minimap camera renders inside it) plus zoom/auto buttons.

use bevy::prelude::*;
use bevy::ui::UiGlobalTransform;

use super::{panel, text};
use crate::app::Screen;
use crate::camera::MinimapRect;
use crate::input::Action;
use crate::palette;

/// The square the minimap camera draws into.
#[derive(Component)]
struct Slot;
#[derive(Component)]
struct CampTimer(u8);

pub const SIZE: f32 = 262.0;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, timers.run_if(in_state(Screen::Match)));
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

pub fn spawn(root: &mut ChildSpawnerCommands) {
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
                ("H", Action::Center),
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
            frame
                .spawn((
                    Node {
                        width: px(SIZE),
                        height: px(SIZE),
                        ..default()
                    },
                    Slot,
                ))
                .with_children(|map| {
                    for camp in 0..tbg_moba_core::map::CAMP_COUNT {
                        let p = tbg_moba_core::map::camp(camp).0;
                        let x = (p.x as f32 / 1000. + 2.) / 204. * SIZE;
                        let y = (202. - p.y as f32 / 1000.) / 204. * SIZE;
                        map.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                left: px(x - 10.),
                                top: px(y - 6.),
                                ..default()
                            },
                            text("", 10., palette::GOLD),
                            CampTimer(camp as u8),
                        ));
                    }
                });
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

fn timers(feed: Res<crate::link::Feed>, mut q: Query<(&CampTimer, &mut Text)>) {
    for (CampTimer(camp), mut text) in &mut q {
        let seconds = feed
            .curr
            .as_ref()
            .and_then(|w| w.camps.iter().find(|c| c.camp == *camp))
            .map(|c| {
                c.respawn_ticks
                    .div_ceil(tbg_moba_core::content::TICK_HZ as u16)
            });
        let value = seconds.map_or_else(String::new, |s| s.to_string());
        if text.0 != value {
            text.0 = value;
        }
    }
}
