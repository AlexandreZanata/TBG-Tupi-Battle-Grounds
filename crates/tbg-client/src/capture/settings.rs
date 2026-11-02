//! Opt-in settings journeys through normal UI actions, with no injected identities or secrets.

use crate::{
    auth,
    ui::{ActionQueue, UiAction},
    GameState,
};
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, open);
}

fn open(
    time: Res<Time>,
    state: Res<State<GameState>>,
    mut actions: ResMut<ActionQueue>,
    mut done: Local<bool>,
    updater: Res<crate::ui::update::Updater>,
) {
    if *done || *state.get() != GameState::Menu || time.elapsed_secs() < 2. {
        return;
    }
    match crate::config::url_param("capture_panel").as_deref() {
        Some("account-backup") if auth::snapshot().profile.is_some() => {
            actions.0.push(UiAction::CopyKey);
            *done = true;
        }
        Some("updates") if updater.idle => {
            actions.0.push(UiAction::UpdateOpen);
            *done = true;
        }
        _ => {}
    }
}
