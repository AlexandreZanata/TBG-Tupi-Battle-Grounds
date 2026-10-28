//! Account widgets on the menu: the profile chip and the save-your-key
//! panel. Rebuilt only when the shown state changes; menu exit despawns
//! them (`DespawnOnExit`).

use bevy::prelude::*;
use tbg_core::protocol::ClientMsg;

use super::account::{download_key, Account, AccountScreen};
use super::textform::{self, ActiveForm, TextKind};
use super::{button, label, screen, Toasts, UiAction};
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
    now: f64,
) {
    match action {
        UiAction::CreateAccount => {
            textform::open(
                form,
                TextKind::Name,
                "Choose a name (3-16 letters, digits, _)",
            );
        }
        UiAction::HaveKey => {
            textform::open(form, TextKind::Key, "Paste your account key");
        }
        UiAction::CopyKey => {
            if let AccountScreen::SaveKey { key, saved, .. } = &mut account.screen {
                config::copy_text(key);
                *saved = true;
                toasts.push("Key copied - store it somewhere safe");
            } else if account.profile.is_some() {
                if let Some(key) = config::load_account_key() {
                    config::copy_text(&key);
                    toasts.push("Key copied - store it somewhere safe");
                }
            }
        }
        UiAction::DownloadKey => {
            if let AccountScreen::SaveKey { key, name, saved } = &mut account.screen {
                download_key(name, key);
                *saved = true;
                toasts.push("Key downloaded");
            }
        }
        UiAction::KeySaved => {
            if let AccountScreen::SaveKey {
                key, saved: true, ..
            } = &account.screen
            {
                config::save_account_key(key);
                account.screen = AccountScreen::None;
                toasts.push("Account ready on this device");
            } else {
                toasts.push("Copy or download your key first");
            }
        }
        UiAction::ClosePanel => {
            account.screen = AccountScreen::None;
        }
        UiAction::Logout => {
            if !account.logout_armed {
                account.logout_armed = true;
                toasts.push("Tap Log out again - did you save your key?");
            } else {
                account.logout_armed = false;
                match link {
                    MatchLink::Net(n) => n.send(ClientMsg::Logout, now),
                    _ => toasts.push("No connection - try again"),
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
                    label("Save your key", 26.0),
                    label(key, 22.0),
                    label("This key is your account.", 14.0),
                    label("Anyone with it can play as you.", 14.0),
                    label("It cannot be recovered.", 14.0),
                    button("Copy", UiAction::CopyKey, 260.0),
                    button("Download", UiAction::DownloadKey, 260.0),
                    button("I saved it", UiAction::KeySaved, 260.0),
                    button("Close", UiAction::ClosePanel, 260.0),
                    label(format!("{name}-tbg-key.txt"), 12.0),
                ],
            ));
        });
}
