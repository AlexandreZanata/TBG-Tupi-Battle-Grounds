//! Shared hover, press, disabled and keyboard-focus appearance for Bevy buttons.

use super::{BLUE, GOLD, INK, LINE};
use bevy::prelude::*;

#[derive(Component)]
pub struct Chrome {
    pub enabled: bool,
    pub focused: bool,
}

#[derive(Resource, Default)]
pub struct Focus(pub Option<Entity>);

type Buttons<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Chrome,
        &'static ComputedNode,
        &'static UiGlobalTransform,
        &'static InheritedVisibility,
        Option<&'static bevy::ui::CalculatedClip>,
        &'static mut BackgroundColor,
        &'static mut BorderColor,
    ),
>;

pub fn plugin(app: &mut App) {
    app.init_resource::<Focus>()
        .add_systems(PostUpdate, style.after(bevy::ui::UiSystems::Layout));
}

fn style(
    window: Single<&Window>,
    mouse: Res<ButtonInput<MouseButton>>,
    focus: Res<Focus>,
    mut buttons: Buttons,
) {
    let pointer = window.cursor_position().map(|p| p * window.scale_factor());
    for (entity, chrome, node, t, visible, clip, mut background, mut border) in &mut buttons {
        let hover = chrome.enabled
            && visible.get()
            && pointer.is_some_and(|p| {
                node.contains_point(*t, p) && clip.is_none_or(|c| c.contains_point(p))
            });
        background.0 = if !chrome.enabled {
            INK
        } else if hover && mouse.pressed(MouseButton::Left) {
            Color::srgb_u8(31, 62, 82)
        } else if hover {
            Color::srgb_u8(64, 108, 131)
        } else {
            BLUE
        };
        *border = BorderColor::all(
            if chrome.enabled && (hover || focus.0 == Some(entity) || chrome.focused) {
                GOLD
            } else {
                LINE
            },
        );
    }
}
