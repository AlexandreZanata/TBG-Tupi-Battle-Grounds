//! Capture the real Troops preview through its ordinary selection and form actions.

use crate::{
    art::anim::Clip,
    config,
    training::{Action, Catalog, Requests, Tab},
    ui::shell::{Panel, Shell},
    GameState,
};
use bevy::{
    prelude::*,
    render::view::screenshot::{save_to_disk, Screenshot, ScreenshotCaptured},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, capture);
}

#[allow(clippy::too_many_arguments)] // Capture reads the actual shell, model selection and camera.
fn capture(
    mut commands: Commands,
    state: Res<State<GameState>>,
    shell: Res<Shell>,
    catalog: Res<Catalog>,
    time: Res<Time>,
    mut queue: ResMut<Requests>,
    window: Single<&Window>,
    camera: Single<&Transform, With<crate::camera::ArenaCamera>>,
    mut started: Local<Option<f32>>,
    mut done: Local<bool>,
) {
    if *done || *state.get() != GameState::Menu {
        return;
    }
    let name = config::url_param("unit").unwrap_or_default();
    let Some(unit) = tbg_core::FULL_POOL
        .into_iter()
        .find(|kind| format!("{kind:?}") == name)
    else {
        return;
    };
    let number = |key, max| {
        config::url_param(key)
            .and_then(|s| s.parse::<u8>().ok())
            .unwrap_or(0)
            .min(max)
    };
    let tab = match config::url_param("troops_tab").as_deref() {
        Some("practice") => Tab::Practice,
        Some("editor") => Tab::Editor,
        _ => Tab::Catalog,
    };
    let clip = match config::url_param("troops_clip").as_deref() {
        Some("walk") => Clip::Walk,
        Some("attack") => Clip::AttackMelee,
        Some("victory") => Clip::Victory,
        _ => Clip::Idle,
    };
    let form = (number("tier", 2), number("branch", 1));
    let Some(start) = *started else {
        queue.0.extend([
            Action::Open(tab),
            Action::Select(unit),
            Action::Form(form.0, form.1),
            Action::Clip(clip),
        ]);
        *started = Some(time.elapsed_secs());
        return;
    };
    if time.elapsed_secs() < start + 2.0
        || shell.panel != Panel::Troops
        || catalog.selected != unit
        || catalog.form != form
        || catalog.tab != tab
    {
        return;
    }
    let Some(path) = config::url_param("capture") else {
        return;
    };
    *done = true;
    eprintln!(
        "P70 Troops: unit={unit:?}; form={form:?}; clip={:?}; viewport={}x{}; dpi={}; camera={:?}",
        catalog.clip,
        window.physical_width(),
        window.physical_height(),
        window.scale_factor(),
        *camera,
    );
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path))
        .observe(
            |_: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>| {
                exit.write(AppExit::Success);
            },
        );
}
