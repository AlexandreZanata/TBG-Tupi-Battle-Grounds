//! Opt-in debug render captures for UI parity checks without OS automation.

use bevy::{
    prelude::*,
    render::view::screenshot::{save_to_disk, Screenshot, ScreenshotCaptured},
};

pub fn plugin(app: &mut App) {
    if crate::config::url_param("capture").is_some() {
        app.add_systems(Update, capture);
        #[cfg(target_os = "linux")]
        if crate::config::url_param("capture_panel").as_deref() == Some("fortify") {
            app.add_systems(
                OnEnter(crate::GameState::InMatch),
                |mut actions: ResMut<crate::ui::ActionQueue>| {
                    actions.0.push(crate::ui::UiAction::SelectTower);
                },
            );
        }
        #[cfg(target_os = "linux")]
        if crate::config::url_param("capture_panel").as_deref() == Some("unit") {
            app.add_systems(
                Update,
                select_unit.run_if(in_state(crate::GameState::InMatch)),
            );
        }
    }
}

fn capture(
    mut commands: Commands,
    time: Res<Time>,
    state: Res<State<crate::GameState>>,
    #[cfg(target_os = "linux")] view: Res<crate::sync::LatestView>,
    #[cfg(target_os = "linux")] tower: Res<crate::ui::fortify::TowerOpen>,
    #[cfg(target_os = "linux")] inspected: Res<crate::ui::inspect::Inspected>,
    mut started: Local<bool>,
) {
    let delay = crate::config::url_param("capture_after")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(8.0)
        .clamp(1.0, 600.0);
    let result_only = crate::config::url_param("capture_on").as_deref() == Some("result");
    if *started
        || time.elapsed_secs() < delay
        || (result_only && *state.get() != crate::GameState::Result)
    {
        return;
    }
    #[cfg(target_os = "linux")]
    {
        if crate::config::url_param("capture_panel").as_deref() == Some("fortify") && !tower.0 {
            return;
        }
        if crate::config::url_param("capture_panel").as_deref() == Some("unit")
            && inspected.0.is_none()
        {
            return;
        }
        // Capture actual match phases instead of racing startup/loading time.
        let wanted = match crate::config::url_param("capture_on").as_deref() {
            Some("countdown") => Some(tbg_core::Phase::Countdown),
            Some("running") => Some(tbg_core::Phase::Running),
            _ => None,
        };
        if wanted.is_some_and(|phase| view.0.as_ref().is_none_or(|v| v.phase != phase)) {
            return;
        }
    }
    let Some(path) = crate::config::url_param("capture") else {
        return;
    };
    *started = true;
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path))
        .observe(
            |_: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>| {
                exit.write(AppExit::Success);
            },
        );
}

#[cfg(target_os = "linux")]
fn select_unit(
    view: Res<crate::sync::LatestView>,
    mut actions: ResMut<crate::ui::ActionQueue>,
    mut done: Local<bool>,
) {
    if *done {
        return;
    }
    let Some(v) = &view.0 else { return };
    if let Some(unit) = v
        .units
        .iter()
        .find(|unit| unit.seat == v.you && !unit.is_summon())
    {
        actions.0.push(crate::ui::UiAction::SelectUnit(unit.id));
        *done = true;
    }
}
