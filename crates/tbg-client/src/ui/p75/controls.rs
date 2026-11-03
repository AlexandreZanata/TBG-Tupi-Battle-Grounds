//! Modal-only keyboard focus and pointer hit testing; gameplay controls remain captured.
use super::*;
type ButtonRef<'a> = (
    Entity,
    &'a UiAction,
    &'a ComputedNode,
    &'a UiGlobalTransform,
    &'a InheritedVisibility,
    Option<&'a bevy::ui::CalculatedClip>,
);
type Buttons<'w, 's> = Query<'w, 's, ButtonRef<'static>, With<DialogButton>>;
#[allow(clippy::too_many_arguments)]
pub(super) fn input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    #[cfg(target_os = "android")] mut pointer: ResMut<crate::ui::mobile::input::Pointer>,
    window: Single<&Window>,
    form: NonSend<ActiveForm>,
    buttons: Buttons,
    mut focus: Local<usize>,
    mut chrome: ResMut<super::super::kit::buttons::Focus>,
    mut panels: Query<&mut ScrollPosition, With<DialogPanel>>,
) {
    if !modal() || textform::is_open(&form) {
        return;
    }
    if keys.just_pressed(KeyCode::Escape) {
        request(Action::Close);
        return;
    }
    let mut visible: Vec<_> = buttons
        .iter()
        .filter(|(_, _, n, _, v, _)| v.get() && n.size.x > 0.)
        .collect();
    visible.sort_by_key(|(_, _, _, t, _, _)| {
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
    chrome.0 = Some(visible[*focus % visible.len()].0);
    if keys.just_pressed(KeyCode::Enter) {
        activate(visible[*focus % visible.len()].1);
    }
    if keys.just_pressed(KeyCode::PageDown) || keys.just_pressed(KeyCode::PageUp) {
        let delta = if keys.just_pressed(KeyCode::PageDown) {
            400.
        } else {
            -400.
        };
        for mut panel in &mut panels {
            panel.0.y = (panel.0.y + delta).max(0.);
        }
    }
    #[cfg(target_os = "android")]
    {
        for (start, end, _) in pointer.taps.clone() {
            let physical = end * window.scale_factor();
            if let Some((_, action, node, transform, _, clip)) =
                visible.iter().find(|(_, _, node, transform, _, clip)| {
                    node.contains_point(**transform, physical)
                        && node.contains_point(**transform, start * window.scale_factor())
                        && clip.is_none_or(|c| c.contains_point(physical))
                })
            {
                let _ = (node, transform, clip);
                activate(action);
            }
        }
        pointer.consumed = true;
        if pointer.touch_frame {
            return;
        }
    }
    if mouse.just_released(MouseButton::Left) {
        mouse_pointer(&visible, &window);
    }
}
fn activate(action: &UiAction) {
    if let UiAction::P75(a) = action {
        request(*a);
    }
}
fn mouse_pointer(buttons: &[ButtonRef<'_>], window: &Window) {
    let Some(pos) = window.cursor_position().map(|p| p * window.scale_factor()) else {
        return;
    };
    for (_, action, node, transform, _, clip) in buttons {
        if node.contains_point(**transform, pos) && clip.is_none_or(|clip| clip.contains_point(pos))
        {
            activate(action);
            break;
        }
    }
}
