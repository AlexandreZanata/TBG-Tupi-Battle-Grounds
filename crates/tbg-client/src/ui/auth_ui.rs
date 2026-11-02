//! Shared desktop auth status and explicit recovery controls, without exposing credentials.

use super::{account::Account, kit};
use crate::{
    auth::{self, Command as AuthCommand, Failure, Status},
    i18n::Lang,
    GameState,
};
use bevy::prelude::*;

#[derive(Component)]
struct Root;
#[derive(Component, Clone, Copy)]
enum Action {
    Continue,
    Guest,
    Retry,
    Login,
}

#[cfg(target_arch = "wasm32")]
mod web;
pub fn plugin(app: &mut App) {
    #[cfg(target_arch = "wasm32")]
    web::plugin(app);
    app.add_systems(Update, render.run_if(in_state(GameState::Menu)));
}

fn failure_key(error: Failure) -> &'static str {
    match error {
        Failure::Suspended { .. } => "p75.suspended",
        Failure::EmailInvalid => "p75.email_invalid",
        Failure::EmailBlocked => "p75.email_blocked",
        Failure::EmailLimit => "p75.email_limit",
        Failure::RegistrationClosed => "p75.registration_closed",
        Failure::CodeInvalid => "p75.code_invalid",
        Failure::Incompatible => "auth.incompatible",
        Failure::InvalidKey => "auth.invalid_key",
        Failure::NameTaken => "auth.name_taken",
        Failure::NameInvalid => "auth.name_invalid",
        Failure::RateLimited => "auth.rate_limited",
        Failure::Conflict => "auth.conflict",
        Failure::RecoveryRequired | Failure::NoSession => "auth.recovery",
        Failure::InMatch => "auth.in_match",
        Failure::Capacity => "auth.capacity",
        Failure::Busy => "auth.busy",
        Failure::InvalidOrigin => "auth.invalid_origin",
        Failure::StorageUnavailable => "auth.temporary",
        _ => "auth.unavailable",
    }
}

#[allow(clippy::too_many_arguments)]
fn render(
    mut commands: Commands,
    lang: Res<Lang>,
    mut form: NonSendMut<super::textform::ActiveForm>,
    account: Res<Account>,
    roots: Query<Entity, With<Root>>,
    actions: Query<(&Action, &ComputedNode, &UiGlobalTransform)>,
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    mut shown: Local<String>,
) {
    for (action, node, transform) in &actions {
        if mouse.just_released(MouseButton::Left)
            && window
                .cursor_position()
                .is_some_and(|p| node.contains_point(*transform, p * window.scale_factor()))
        {
            match action {
                Action::Continue => {
                    auth::request(AuthCommand::ContinueHere);
                }
                Action::Guest => {
                    auth::request(AuthCommand::Guest);
                }
                Action::Retry => {
                    auth::request(if auth::snapshot().status == Status::LogoutPending {
                        AuthCommand::Logout
                    } else {
                        AuthCommand::Refresh
                    });
                }
                Action::Login => super::textform::open(
                    &mut form,
                    super::textform::TextKind::Key,
                    lang.t(crate::i18n::Msg::AccountPromptKey),
                ),
            }
        }
    }
    let view = auth::snapshot();
    if let Some(error) = view.error {
        if let Failure::Suspended { until, reason } = error {
            super::p75::suspend(until, reason);
        }
        if super::textform::is_open(&form) {
            super::textform::set_error(&form, &lang.key(failure_key(error)));
        }
    }
    if !crate::config::bevy_ui() {
        return;
    }
    let key = match view.status {
        Status::Restoring => Some("auth.restoring"),
        Status::Refreshing => Some("auth.refreshing"),
        Status::PersistenceUnavailable => Some("auth.temporary"),
        Status::Conflict => Some("auth.conflict"),
        Status::ReauthRequired => Some("auth.reauth"),
        Status::LogoutPending => Some("auth.logout_pending"),
        Status::Guest if view.legacy_available => Some("auth.legacy"),
        Status::Authenticated if !view.remember => Some("auth.session_only"),
        _ => None,
    };
    let signature = format!(
        "{:?}|{:?}|{}|{}|{}",
        view.status,
        view.error,
        view.remember,
        lang.code(),
        account.pending.is_some()
    );
    if *shown == signature {
        return;
    }
    *shown = signature;
    for root in &roots {
        commands.entity(root).despawn();
    }
    let Some(key) = key else {
        return;
    };
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: percent(2),
                top: px(121),
                max_width: percent(42),
                ..default()
            },
            Root,
            DespawnOnExit(GameState::Menu),
            GlobalZIndex(12),
        ))
        .with_children(|root| {
            root.spawn(kit::panel()).with_children(|panel| {
                panel.spawn(kit::muted(lang.key(key), 13.));
                if let Some(error) = view.error {
                    panel.spawn(kit::muted(lang.key(failure_key(error)), 13.));
                }
                panel.spawn(kit::row()).with_children(|row| {
                    let choices: &[(Action, &str)] = match view.status {
                        Status::Conflict => &[(Action::Continue, "auth.continue_here")],
                        Status::ReauthRequired => &[
                            (Action::Login, "lobby.account.key"),
                            (Action::Guest, "auth.guest"),
                        ],
                        Status::LogoutPending => &[(Action::Retry, "auth.retry")],
                        _ => &[],
                    };
                    for (action, key) in choices {
                        row.spawn((kit::control(lang.key(key), true, 190.), *action));
                    }
                });
            });
        });
}
