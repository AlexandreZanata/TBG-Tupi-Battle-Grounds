//! Manual copy/download and protected key import without changing the active login.

use super::{
    account::Account,
    textform::{self, ActiveForm, TextKind},
    Toasts,
};
use crate::{
    auth,
    i18n::{Lang, Msg},
};
use bevy::prelude::*;

#[derive(Clone, Copy)]
pub(crate) enum ExportAction {
    Copy,
    Download,
}

pub(crate) fn matches_account(key: &str, account: &Account) -> bool {
    account.profile.as_ref().is_some_and(|profile| {
        tbg_core::account::parse_key(key)
            .is_some_and(|(name, _)| name.eq_ignore_ascii_case(&profile.name))
    })
}

pub(crate) fn export(action: ExportAction, name: &str, key: &str) -> bool {
    match action {
        ExportAction::Download => super::account::download_key(name, key),
        ExportAction::Copy => {
            #[cfg(not(target_arch = "wasm32"))]
            {
                crate::platform::copy(key)
            }
            #[cfg(target_arch = "wasm32")]
            {
                crate::config::copy_text(key);
                true
            }
        }
    }
}

pub(crate) fn request(account: &mut Account, action: ExportAction, toasts: &mut Toasts) {
    if let Some(key) = account
        .recovery_key
        .as_ref()
        .filter(|key| matches_account(key, account))
    {
        let Some(profile) = account.profile.as_ref() else {
            return;
        };
        toasts.say(if export(action, &profile.name, key) {
            match action {
                ExportAction::Copy => Msg::AccountKeyCopied,
                ExportAction::Download => Msg::AccountKeyDownloaded,
            }
        } else {
            Msg::AuthBackupFailed
        });
    } else if auth::request(auth::Command::RecoveryKey(None)) {
        account.key_action = Some(action);
    }
}

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Update, finish);
}

fn finish(
    mut account: ResMut<Account>,
    mut form: NonSendMut<ActiveForm>,
    mut toasts: ResMut<Toasts>,
    lang: Res<Lang>,
) {
    let Some(result) = account.key_result.take() else {
        return;
    };
    account.pending = None;
    match result {
        Ok(Some(key)) if matches_account(&key, &account) => {
            account.recovery_key = Some(key);
            if let Some(action) = account.key_action.take() {
                textform::close(&mut form);
                request(&mut account, action, &mut toasts);
            }
        }
        Ok(None) => textform::open(
            &mut form,
            TextKind::BackupKey,
            &lang.key("account.backup.prompt"),
        ),
        Err(error) => {
            let key = if error == auth::Failure::InvalidKey {
                "account.backup.invalid"
            } else {
                "account.backup.failed"
            };
            if textform::is_open(&form) {
                textform::set_error(&form, &lang.key(key));
            } else {
                toasts.push(lang.key(key));
                account.key_action = None;
            }
        }
        _ => {
            account.key_action = None;
            toasts.say(Msg::AuthBackupFailed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn br_as05_backup_never_exports_another_accounts_cached_key() {
        let account = Account {
            profile: Some(tbg_core::protocol::Profile {
                name: "CurrentUser".into(),
                rating: 1000,
                ranked_games: 0,
                wins: 0,
                losses: 0,
                draws: 0,
            }),
            ..Default::default()
        };
        assert!(matches_account("CurrentUser-0000-0000-0000-0000", &account));
        assert!(!matches_account("OtherUser-0000-0000-0000-0000", &account));
        assert!(!matches_account("not-a-key", &account));
    }
}
