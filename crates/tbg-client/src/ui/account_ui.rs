//! Account widgets on the menu: the profile chip and the save-your-key
//! panel. Rebuilt only when the shown state changes; menu exit despawns
//! them (`DespawnOnExit`).

use bevy::prelude::*;
use tbg_core::protocol::ClientMsg;

use super::account::{download_key, Account, AccountScreen};
use super::textform::{self, ActiveForm, TextKind};
use super::{button, label, screen, Toasts, UiAction};
use crate::i18n::{Lang, Msg};
use crate::link::MatchLink;
use crate::{config, GameState};

#[derive(Component)]
struct SavePanel;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, panel.run_if(in_state(GameState::Menu)));
}

/// Apply the P11 account buttons (called from `actions::apply`).
pub fn apply_action(
    action: UiAction,
    link: &mut MatchLink,
    account: &mut Account,
    form: &mut ActiveForm,
    toasts: &mut Toasts,
    lang: Lang,
    now: f64,
) {
    match action {
        UiAction::CreateAccount => {
            textform::open(form, TextKind::Name, lang.t(Msg::AccountPromptName));
        }
        UiAction::HaveKey => {
            textform::open(form, TextKind::Key, lang.t(Msg::AccountPromptKey));
        }
        UiAction::CopyKey => {
            if let AccountScreen::SaveKey { key, saved, .. } = &mut account.screen {
                config::copy_text(key);
                *saved = true;
                toasts.say(Msg::AccountKeyCopied);
            } else if account.profile.is_some() {
                if let Some(key) = config::load_account_key() {
                    config::copy_text(&key);
                    toasts.say(Msg::AccountKeyCopied);
                } else {
                    toasts.say(Msg::AccountNoKey);
                }
            }
        }
        UiAction::DownloadKey => {
            if let AccountScreen::SaveKey { key, name, saved } = &mut account.screen {
                download_key(name, key);
                *saved = true;
                toasts.say(Msg::AccountKeyDownloaded);
            } else if let Some((profile, key)) =
                account.profile.as_ref().zip(config::load_account_key())
            {
                download_key(&profile.name, &key);
                toasts.say(Msg::AccountKeyDownloaded);
            } else {
                toasts.say(Msg::AccountNoKey);
            }
        }
        UiAction::KeySaved => {
            if let AccountScreen::SaveKey {
                key, saved: true, ..
            } = &account.screen
            {
                config::save_account_key(key);
                account.screen = AccountScreen::None;
                toasts.say(Msg::AccountReady);
            } else {
                toasts.say(Msg::AccountSaveFirst);
            }
        }
        UiAction::ClosePanel => {
            account.screen = AccountScreen::None;
        }
        UiAction::Logout => {
            if !account.logout_armed {
                account.logout_armed = true;
                toasts.say(Msg::AccountLogoutConfirm);
            } else {
                account.logout_armed = false;
                match link {
                    MatchLink::Net(n) => n.send(ClientMsg::Logout, now),
                    _ => toasts.say(Msg::AccountNoConnection),
                }
            }
        }
        _ => {}
    }
}

/// Modal save-your-key panel while the screen is open.
fn panel(
    mut commands: Commands,
    account: Res<Account>,
    lang: Res<Lang>,
    roots: Query<Entity, With<SavePanel>>,
    mut shown: Local<bool>,
) {
    let open = matches!(account.screen, AccountScreen::SaveKey { .. });
    if open == *shown && (open || roots.is_empty()) {
        return;
    }
    *shown = open;
    for e in &roots {
        commands.entity(e).despawn();
    }
    let AccountScreen::SaveKey { key, name, .. } = &account.screen else {
        return;
    };
    let (key, name) = (key.clone(), name.clone());
    commands
        .spawn((
            screen(JustifyContent::Center),
            DespawnOnExit(GameState::Menu),
            SavePanel,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(px(24.0)),
                    row_gap: px(12.0),
                    border_radius: BorderRadius::all(px(16.0)),
                    ..default()
                },
                BackgroundColor(super::PANEL),
                children![
                    label(lang.t(Msg::AccountSaveTitle), 26.0),
                    label(key, 22.0),
                    label(lang.t(Msg::AccountSaveLine1), 14.0),
                    label(lang.t(Msg::AccountSaveLine2), 14.0),
                    label(lang.t(Msg::AccountSaveLine3), 14.0),
                    button(lang.t(Msg::AccountCopy), UiAction::CopyKey, 260.0),
                    button(lang.t(Msg::AccountDownload), UiAction::DownloadKey, 260.0),
                    button(lang.t(Msg::AccountSavedIt), UiAction::KeySaved, 260.0),
                    button(lang.t(Msg::CommonClose), UiAction::ClosePanel, 260.0),
                    label(format!("{name}-tbg-key.txt"), 12.0),
                ],
            ));
        });
}
