//! Opt-in debug render captures for UI parity checks without OS automation.

use bevy::{
    prelude::*,
    render::view::screenshot::{save_to_disk, Screenshot, ScreenshotCaptured},
};

pub fn plugin(app: &mut App) {
    if crate::config::url_param("capture").is_some() {
        app.add_systems(Update, capture);
    }
}

fn capture(
    mut commands: Commands,
    time: Res<Time>,
    state: Res<State<crate::GameState>>,
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
