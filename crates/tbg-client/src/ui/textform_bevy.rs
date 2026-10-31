//! Native and desktop-web account/room forms using shared Bevy text editing.

use super::{
    kit,
    textform::{self, ActiveForm, TextKind},
};
use bevy::{
    input::keyboard::{Key, KeyboardInput},
    prelude::*,
};

#[derive(Component)]
struct FormRoot;
#[derive(Component, Clone, Copy)]
enum Control {
    Field,
    Submit,
    Cancel,
    Reveal,
}

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (keyboard, render).chain());
}

fn insert(form: &mut ActiveForm, value: &str) {
    let room = form.kind == Some(TextKind::Room);
    let value = if room {
        value.to_ascii_uppercase()
    } else {
        value.into()
    };
    let limit = match form.kind {
        Some(TextKind::Name) => 16,
        Some(TextKind::Room) => 5,
        _ => 128,
    };
    form.edit.insert(&value, limit, |c| {
        if room {
            c.is_ascii_alphanumeric()
        } else {
            true
        }
    });
}

fn keyboard(
    mut form: NonSendMut<ActiveForm>,
    mut events: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if !textform::bevy_form() || !textform::is_open(&form) {
        events.clear();
        return;
    }
    let ctrl = keys.any_pressed([
        KeyCode::ControlLeft,
        KeyCode::ControlRight,
        KeyCode::SuperLeft,
        KeyCode::SuperRight,
    ]);
    let shift = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    for event in events.read().filter(|e| e.state.is_pressed()) {
        match &event.logical_key {
            Key::Enter if !keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight]) => {
                match form.focus {
                    2 => textform::cancel(&form),
                    3 => form.masked = !form.masked,
                    _ => textform::submit(&form),
                }
            }
            Key::Tab => {
                let count = if form.kind == Some(TextKind::Key) {
                    4
                } else {
                    3
                };
                form.focus = if shift {
                    (form.focus + count - 1) % count
                } else {
                    (form.focus + 1) % count
                };
            }
            Key::Escape => textform::cancel(&form),
            Key::Backspace => form.edit.backspace(),
            Key::Delete => form.edit.delete(),
            Key::ArrowLeft => {
                let at = form.edit.cursor.saturating_sub(1);
                form.edit.move_to(at, shift);
            }
            Key::ArrowRight => {
                let at = form.edit.cursor + 1;
                form.edit.move_to(at, shift);
            }
            Key::Home => form.edit.move_to(0, shift),
            Key::End => {
                let at = form.edit.text.chars().count();
                form.edit.move_to(at, shift);
            }
            Key::Character(c) if ctrl && c.eq_ignore_ascii_case("a") => {
                form.edit.anchor = 0;
                form.edit.cursor = form.edit.text.chars().count();
            }
            Key::Character(c) if ctrl && c.eq_ignore_ascii_case("c") => {
                crate::config::copy_text(form.edit.selected())
            }
            Key::Character(c) if ctrl && c.eq_ignore_ascii_case("x") => {
                crate::config::copy_text(form.edit.selected());
                form.edit.delete();
            }
            #[cfg(not(target_arch = "wasm32"))]
            Key::Character(c) if ctrl && c.eq_ignore_ascii_case("v") => {
                if let Some(value) = crate::platform::paste() {
                    insert(&mut form, &value);
                }
            }
            Key::Character(c) if !ctrl => insert(&mut form, c),
            _ => {}
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        let value = take_paste();
        if !value.is_empty() {
            insert(&mut form, &value);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn render(
    mut commands: Commands,
    mut form: NonSendMut<ActiveForm>,
    roots: Query<Entity, With<FormRoot>>,
    controls: Query<(&Control, &ComputedNode, &UiGlobalTransform)>,
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    mut shown: Local<String>,
    lang: Res<crate::i18n::Lang>,
) {
    if !textform::bevy_form() {
        return;
    }
    let open = textform::is_open(&form);
    #[cfg(target_arch = "wasm32")]
    set_paste_active(open);
    if open && mouse.just_released(MouseButton::Left) {
        if let Some(p) = window.cursor_position() {
            for (control, node, transform) in &controls {
                if node.contains_point(*transform, p * window.scale_factor()) {
                    match control {
                        Control::Field => form.focus = 0,
                        Control::Submit => {
                            form.focus = 1;
                            textform::submit(&form);
                        }
                        Control::Cancel => {
                            form.focus = 2;
                            textform::cancel(&form);
                        }
                        Control::Reveal => {
                            form.focus = 3;
                            form.masked = !form.masked;
                        }
                    }
                }
            }
        }
    }
    let error = form.error.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let signature = if open {
        format!(
            "{:?}|{}|{}|{}|{}|{}",
            form.kind,
            form.edit.display(form.masked),
            form.prompt,
            error,
            lang.code(),
            form.focus
        )
    } else {
        String::new()
    };
    if signature == *shown {
        return;
    }
    *shown = signature;
    for e in &roots {
        commands.entity(e).despawn();
    }
    if !open {
        return;
    }
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0., 0., 0., 0.8)),
            GlobalZIndex(100),
            FormRoot,
        ))
        .with_children(|root| {
            root.spawn(kit::panel()).with_children(|p| {
                p.spawn(kit::heading(&form.prompt, 24.));
                p.spawn((
                    Node {
                        min_width: px(400),
                        min_height: px(54),
                        padding: UiRect::all(px(14)),
                        border: UiRect::all(px(1)),
                        ..default()
                    },
                    BorderColor::all(if form.focus == 0 {
                        kit::GOLD
                    } else {
                        kit::LINE
                    }),
                    Control::Field,
                    BackgroundColor(kit::INK),
                    children![super::label(form.edit.display(form.masked), 20.)],
                ));
                p.spawn(kit::muted("Ctrl+A · Ctrl+C · Ctrl+V · Enter · Esc", 13.));
                if !error.is_empty() {
                    p.spawn(super::label(error, 15.))
                        .insert(TextColor(Color::srgb(1., 0.5, 0.4)));
                }
                p.spawn(kit::row()).with_children(|row| {
                    for (key, control) in [
                        ("native.continue", Control::Submit),
                        ("native.cancel", Control::Cancel),
                    ] {
                        let at = if matches!(control, Control::Submit) {
                            1
                        } else {
                            2
                        };
                        row.spawn((kit::control(lang.key(key), true, 140.), control))
                            .insert(kit::buttons::Chrome {
                                enabled: true,
                                focused: form.focus == at,
                            })
                            .insert(BorderColor::all(if form.focus == at {
                                kit::GOLD
                            } else {
                                kit::LINE
                            }));
                    }
                    if form.kind == Some(TextKind::Key) {
                        row.spawn((
                            kit::control(lang.key("native.reveal"), true, 140.),
                            Control::Reveal,
                        ))
                        .insert(kit::buttons::Chrome {
                            enabled: true,
                            focused: form.focus == 3,
                        })
                        .insert(BorderColor::all(if form.focus == 3 {
                            kit::GOLD
                        } else {
                            kit::LINE
                        }));
                    }
                });
            });
        });
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(inline_js = r#"
let pasted = '', active = false;
document.addEventListener('paste', event => {
  if (active) { pasted = event.clipboardData.getData('text/plain'); event.preventDefault(); }
});
export function tbgTakePaste() { const value = pasted; pasted = ''; return value; }
export function tbgSetPasteActive(value) { active = value; }
"#)]
extern "C" {
    #[wasm_bindgen::prelude::wasm_bindgen(js_name = tbgTakePaste)]
    fn take_paste() -> String;
    #[wasm_bindgen::prelude::wasm_bindgen(js_name = tbgSetPasteActive)]
    fn set_paste_active(active: bool);
}
