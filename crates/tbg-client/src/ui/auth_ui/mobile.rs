//! Touch recovery buttons reuse the shared explicit auth actions, with modal input ownership.

use super::{activate, Action};
use crate::ui::{mobile::input, textform::ActiveForm};
use bevy::prelude::*;

#[allow(clippy::type_complexity)]
pub(super) fn touch(
    mut pointer: ResMut<input::Pointer>,
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    scale: Res<UiScale>,
    lang: Res<crate::i18n::Lang>,
    mut form: NonSendMut<ActiveForm>,
    buttons: Query<(
        &Action,
        &ComputedNode,
        &UiGlobalTransform,
        &InheritedVisibility,
        &bevy::ui::ComputedStackIndex,
        Option<&bevy::ui::CalculatedClip>,
    )>,
) {
    if pointer.consumed || crate::ui::textform::is_open(&form) || window.height() > window.width() {
        return;
    }
    let mut taps: Vec<_> = pointer.taps.iter().map(|tap| (tap.0, tap.1)).collect();
    if let Some(point) = mouse_release(&pointer, &mouse, &window) {
        taps.push((point, point));
    }
    for (start, end) in taps {
        let physical = end * window.scale_factor();
        let density = window.scale_factor() * scale.0;
        let hit = buttons
            .iter()
            .filter(|(_, node, transform, visible, _, clip)| {
                visible.get()
                    && input::hit(node, **transform, physical, density)
                    && input::hit(node, **transform, start * window.scale_factor(), density)
                    && clip.is_none_or(|clip| clip.contains_point(physical))
            })
            .max_by_key(|(_, _, _, _, stack, _)| stack.0)
            .map(|(action, ..)| *action);
        if let Some(action) = hit {
            pointer.consumed = true;
            activate(action, &mut form, *lang);
        }
    }
}

fn mouse_release(
    pointer: &input::Pointer,
    mouse: &ButtonInput<MouseButton>,
    window: &Window,
) -> Option<Vec2> {
    if !mouse.just_released(MouseButton::Left) {
        return None;
    }
    let point = window.cursor_position()?;
    let duplicate =
        pointer.touch_frame && pointer.taps.iter().any(|tap| tap.1.distance(point) < 2.);
    (!duplicate).then_some(point)
}
