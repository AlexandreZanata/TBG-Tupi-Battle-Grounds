//! Browser update adapter: discover through the existing version API and reload only while idle.

use super::{UiAction, UpdState, Updater};
use crate::{platform::http::Request, GameState};
use bevy::prelude::*;

#[derive(Default)]
pub(super) struct Inner {
    request: Option<Request>,
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (poll, super::view::overlay, super::view::badge).chain(),
    );
}

pub(super) fn apply(updater: &mut Updater, action: UiAction) {
    match action {
        UiAction::UpdateOpen | UiAction::UpdateCheck => {
            updater.opened = true;
            updater.state = UpdState::Checking;
            updater.inner.request = Some(Request::get("/api/version"));
        }
        UiAction::UpdateLater | UiAction::UpdateCancel => {
            updater.opened = false;
            updater.state = UpdState::Off;
        }
        UiAction::UpdateNow | UiAction::UpdateRestart if updater.idle => {
            if let Some(window) = web_sys::window() {
                let _ = window.location().reload();
            }
        }
        _ => {}
    }
}

fn poll(
    mut updater: ResMut<Updater>,
    state: Res<State<GameState>>,
    form: NonSend<crate::ui::textform::ActiveForm>,
    account: Res<crate::ui::account::Account>,
) {
    updater.idle = *state.get() == GameState::Menu
        && !crate::ui::textform::is_open(&form)
        && matches!(account.screen, crate::ui::account::AccountScreen::None)
        && !matches!(
            crate::auth::snapshot().status,
            crate::auth::Status::Restoring
                | crate::auth::Status::Refreshing
                | crate::auth::Status::LogoutPending
        );
    let Some(result) = updater.inner.request.as_ref().and_then(Request::take) else {
        return;
    };
    updater.inner.request = None;
    updater.state = match result {
        Ok(value)
            if crate::ui::shell::data::newer_client(
                &value,
                env!("CARGO_PKG_VERSION"),
                tbg_core::protocol::PROTOCOL_VERSION,
            )
            .is_some() =>
        {
            UpdState::Available {
                build: value["latest_client"].as_str().unwrap_or_default().into(),
                version: value["latest_client"].as_str().unwrap_or_default().into(),
                size: 0,
                required: false,
            }
        }
        Ok(_) => UpdState::Off,
        Err(_) => UpdState::Error("unavailable".into()),
    };
}
