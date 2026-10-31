//! Shell hit testing, keyboard focus, scrolling and presentation-only actions.

use super::{arcanas, Action, Lang, Panel, Shell, UiAction};
use crate::platform::storage::{self, Scope};
use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    prelude::*,
    window::PresentMode,
};

#[allow(clippy::type_complexity)]
type Buttons<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        Option<&'static Action>,
        &'static UiAction,
        &'static ComputedNode,
        &'static UiGlobalTransform,
        &'static InheritedVisibility,
        Option<&'static bevy::ui::CalculatedClip>,
    ),
>;

#[allow(clippy::too_many_arguments)]
pub fn mouse_and_keys(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut window: Single<&mut Window>,
    buttons: Buttons,
    mut borders: Query<&mut BorderColor>,
    mut shell: ResMut<Shell>,
    mut form: NonSendMut<super::ActiveForm>,
    mut queue: ResMut<super::super::ActionQueue>,
    mut training: ResMut<crate::training::Requests>,
    mut focus: Local<usize>,
    mut button_focus: ResMut<super::kit::buttons::Focus>,
    lang: Res<Lang>,
    mut lights: Query<&mut DirectionalLight>,
    mut volume: ResMut<bevy::audio::GlobalVolume>,
) {
    if !crate::config::bevy_ui() || super::textform::is_open(&form) {
        return;
    }
    let mut ordered: Vec<_> = buttons
        .iter()
        .filter(|(_, a, u, node, t, v, clip)| {
            v.get()
                && (a.is_some() || **u != UiAction::Disabled)
                && clip.is_none_or(|c| c.contains_point(t.translation))
                && node.size.x > 0.0
        })
        .collect();
    ordered.sort_by_key(|(_, _, _, _, t, _, _)| {
        let p = t.translation;
        ((p.y * 10.) as i32, (p.x * 10.) as i32)
    });
    let mut selected = None;
    if mouse.just_released(MouseButton::Left) {
        if let Some(p) = window.cursor_position() {
            selected = ordered.iter().position(|(_, _, _, node, t, _, clip)| {
                node.contains_point(**t, p * window.scale_factor())
                    && clip.is_none_or(|c| c.contains_point(p * window.scale_factor()))
            });
        }
    }
    if keys.just_pressed(KeyCode::Tab) && !ordered.is_empty() {
        *focus = (*focus + 1) % ordered.len();
    }
    if keys.just_pressed(KeyCode::Enter)
        && !keys.any_pressed([KeyCode::AltLeft, KeyCode::AltRight])
        && !ordered.is_empty()
    {
        selected = Some(*focus % ordered.len());
    }
    if keys.just_pressed(KeyCode::Escape) {
        shell.panel = Panel::Lobby;
        shell.dirty = true;
        queue
            .0
            .extend([UiAction::CloseRanking, UiAction::ClosePanel]);
    }
    let pointer = window.cursor_position().map(|p| p * window.scale_factor());
    for (i, (entity, _, _, node, t, _, _)) in ordered.iter().enumerate() {
        if let Ok(mut border) = borders.get_mut(*entity) {
            let hovered = pointer.is_some_and(|p| node.contains_point(**t, p));
            *border = BorderColor::all(if hovered || (keys.pressed(KeyCode::Tab) && *focus == i) {
                super::kit::GOLD
            } else {
                super::kit::LINE
            });
        }
    }
    if keys.just_pressed(KeyCode::Tab) {
        button_focus.0 = ordered.get(*focus % ordered.len().max(1)).map(|row| row.0);
    }
    if let Some(index) = selected {
        *focus = index;
        let (_, action, ui, _, _, _, _) = ordered[index];
        if let Some(Action::Catalog(action)) = action {
            training.0.push(*action);
            return;
        }
        if let Some(action) = action {
            #[cfg(target_os = "linux")]
            if matches!(action, Action::Open(_)) {
                queue.0.push(UiAction::CloseRanking);
            }
            if let Action::Arena(arena) = action {
                queue.0.push(UiAction::PreviewArena(*arena));
            }
            apply(
                *action,
                &mut shell,
                &mut form,
                &mut window,
                &mut lights,
                &mut volume,
                *lang,
            );
        } else if keys.just_pressed(KeyCode::Enter) {
            queue.0.push(*ui);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn apply(
    action: Action,
    shell: &mut Shell,
    form: &mut super::ActiveForm,
    window: &mut Window,
    lights: &mut Query<&mut DirectionalLight>,
    volume: &mut bevy::audio::GlobalVolume,
    lang: Lang,
) {
    shell.dirty = true;
    match action {
        Action::Open(panel) => {
            shell.panel = panel;
            if panel == Panel::Arcanas
                && tbg_core::ArcanaPreset::complete(shell.draft.ranks).is_ok()
            {
                shell.draft = crate::config::load_arcanas();
            }
        }
        Action::RoomInput => super::textform::open(
            form,
            super::TextKind::Room,
            &lang.key("lobby.friend.room_code"),
        ),
        Action::Stronghold(kind) => crate::config::save_stronghold(kind),
        Action::Arena(arena) => {
            crate::config::save_arena(arena);
        }
        Action::News(delta) => {
            shell.news = (shell.news as i32 + i32::from(delta)).rem_euclid(3) as usize
        }
        Action::CopyWatch(id) => crate::config::copy_text(
            &crate::platform::page_url(&format!("/?watch={id:016x}")).unwrap_or_default(),
        ),
        Action::Download => crate::config::open_url(
            "https://github.com/AlexandreZanata/TBG-Tupi-Battle-Grounds/releases",
        ),
        Action::Arcana(_, _) | Action::ResetArcanas | Action::DefaultArcanas => {
            edit_arcanas(action, shell, lang)
        }
        _ => video_audio(action, window, lights, volume),
    }
}
fn edit_arcanas(action: Action, shell: &mut Shell, lang: Lang) {
    match action {
        Action::Arcana(index, delta) => {
            if !arcanas::change(&mut shell.draft, index, delta) {
                shell.notice = lang.key("native.arcana_blocked");
                return;
            }
        }
        Action::ResetArcanas => shell.draft = tbg_core::ArcanaPreset::NEUTRAL,
        Action::DefaultArcanas => shell.draft = tbg_core::ArcanaPreset::DEFAULT,
        _ => return,
    }
    if tbg_core::ArcanaPreset::complete(shell.draft.ranks).is_ok() {
        crate::config::save_arcanas(shell.draft);
        shell.notice = lang.key("native.saved");
    } else {
        shell.notice = lang.key("native.arcana_draft");
    }
}
fn video_audio(
    action: Action,
    window: &mut Window,
    lights: &mut Query<&mut DirectionalLight>,
    volume: &mut bevy::audio::GlobalVolume,
) {
    match action {
        Action::Fullscreen => {
            #[cfg(not(target_arch = "wasm32"))]
            crate::platform::window::toggle_fullscreen(window);
            #[cfg(target_arch = "wasm32")]
            if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                if let Some(el) = doc.document_element() {
                    let _ = el.request_fullscreen();
                }
            }
        }
        Action::Vsync => {
            let off = window.present_mode != PresentMode::AutoNoVsync;
            window.present_mode = if off {
                PresentMode::AutoNoVsync
            } else {
                PresentMode::AutoVsync
            };
            crate::config::save_flag("window.no_vsync", off);
        }
        Action::Quality => {
            let low = !crate::config::load_flag("video.low");
            crate::config::save_flag("video.low", low);
            for mut light in lights.iter_mut() {
                light.shadow_maps_enabled = !low;
            }
        }
        Action::Volume(percent) => {
            volume.volume = bevy::audio::Volume::Linear(f32::from(percent) / 100.);
            storage::set(Scope::Local, "audio.volume", &percent.to_string());
        }
        _ => {}
    }
}

pub fn scroll(
    mut events: MessageReader<MouseWheel>,
    window: Single<&Window>,
    mut lists: Query<(&ComputedNode, &UiGlobalTransform, &mut ScrollPosition)>,
) {
    let Some(p) = window.cursor_position().map(|p| p * window.scale_factor()) else {
        events.clear();
        return;
    };
    for event in events.read() {
        let delta = event.y
            * if event.unit == MouseScrollUnit::Line {
                36.
            } else {
                1.
            };
        for (node, t, mut position) in &mut lists {
            if node.contains_point(*t, p) {
                position.0.y = (position.0.y - delta).max(0.);
            }
        }
    }
}
