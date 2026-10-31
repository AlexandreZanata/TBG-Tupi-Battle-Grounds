//! Keyboard navigation for the existing queue, result and battle actions.

use super::{ActionQueue, UiAction};
use crate::GameState;
use bevy::prelude::*;

#[cfg(not(target_os = "linux"))]
type Buttons<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static UiAction,
        &'static ComputedNode,
        &'static UiGlobalTransform,
        &'static InheritedVisibility,
        Option<&'static bevy::ui::CalculatedClip>,
    ),
>;

#[cfg(target_os = "linux")]
type Buttons<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static UiAction,
        &'static ComputedNode,
        &'static UiGlobalTransform,
        &'static InheritedVisibility,
        Option<&'static bevy::ui::CalculatedClip>,
        Option<&'static super::native_battle::CouncilControl>,
    ),
>;

pub(super) fn navigate(
    keys: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    form: NonSend<super::textform::ActiveForm>,
    buttons: Buttons,
    mut focus: Local<usize>,
    mut button_focus: ResMut<super::kit::buttons::Focus>,
    mut queue: ResMut<ActionQueue>,
    #[cfg(target_os = "linux")] view: Res<crate::sync::LatestView>,
) {
    if *state.get() == GameState::Menu || super::textform::is_open(&form) {
        return;
    }
    if keys.just_pressed(KeyCode::Escape) {
        queue.0.push(match state.get() {
            GameState::Queue => UiAction::Cancel,
            GameState::Result => UiAction::ToMenu,
            _ => UiAction::Deselect,
        });
    }
    if !keys.just_pressed(KeyCode::Tab) && !keys.just_pressed(KeyCode::Enter) {
        return;
    }
    let mut visible: Vec<_> = buttons
        .iter()
        .filter(|entry| {
            let (_, a, n, t, v, c, ..) = entry;
            #[cfg(target_os = "linux")]
            if view.0.as_ref().is_some_and(|v| v.augment_round.is_some()) && entry.6.is_none() {
                return false;
            }
            **a != UiAction::Disabled
                && v.get()
                && n.size.x > 0.
                && c.is_none_or(|c| c.contains_point(t.translation))
        })
        .collect();
    visible.sort_by_key(|(_, _, _, t, ..)| {
        (
            (t.translation.y * 10.) as i32,
            (t.translation.x * 10.) as i32,
        )
    });
    if visible.is_empty() {
        return;
    }
    if keys.just_pressed(KeyCode::Tab) {
        *focus = if keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
            (*focus + visible.len() - 1) % visible.len()
        } else {
            (*focus + 1) % visible.len()
        };
    }
    button_focus.0 = Some(visible[*focus % visible.len()].0);
    if keys.just_pressed(KeyCode::Enter) && !keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight])
    {
        queue.0.push(*visible[*focus % visible.len()].1);
    }
}
