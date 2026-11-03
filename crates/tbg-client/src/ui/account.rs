//! Account panel state (BR-P04, BR-P05): profile chip, save-your-key
//! screen, login/logout. Server replies arrive through `AccountInbox`
//! (filled by `sync::messages`); text entry comes from `textform` events.

use bevy::prelude::*;
use tbg_core::protocol::{AccountError, Profile, ServerMsg};

use super::textform::{self, ActiveForm, TextCancelled, TextEntered, TextKind};
use super::Toasts;
use crate::i18n::{Lang, Msg};

/// Replies the account panel owns (filled by `sync::messages`).
#[derive(Resource, Default)]
pub struct AccountInbox(pub Vec<(u64, ServerMsg)>);
impl AccountInbox {
    pub fn push(&mut self, message: ServerMsg) {
        self.0.push((crate::auth::epoch(), message));
    }
}

#[derive(Default)]
pub(crate) enum AccountScreen {
    #[default]
    None,
    SaveKey {
        key: String,
        name: String,
        saved: bool,
    },
}

#[derive(Resource, Default)]
pub struct Account {
    pub registration_name: Option<String>,
    pub(crate) recovery_key: Option<String>,
    pub(crate) key_action: Option<super::account_backup::ExportAction>,
    pub(crate) key_result: Option<Result<Option<String>, crate::auth::Failure>>,
    pub profile: Option<Profile>,
    pub(crate) screen: AccountScreen,
    pub(crate) pending: Option<TextKind>,
    /// A startup auto-login is in flight (its `BadKey` keeps the key).
    pub(crate) auto_login: bool,
    /// First `Logout` tap arms; the second within the same menu visit logs out.
    pub(crate) logout_armed: bool,
    /// P12: last settled rating change `(before, after, arrived_at)` for the
    /// result screen count-up. Cleared on the next `MatchStart`.
    pub(crate) rating_change: Option<(u16, u16, f64)>,
}

pub fn plugin(app: &mut App) {
    app.init_resource::<Account>()
        .init_resource::<AccountInbox>()
        .add_plugins(super::account_backup::plugin)
        .add_systems(Update, (watch_text, text_cancelled, drain_inbox));
}

fn error_msg(code: AccountError) -> Msg {
    match code {
        AccountError::NameTaken => Msg::AccountErrorNameTaken,
        AccountError::NameInvalid => Msg::AccountErrorNameInvalid,
        AccountError::RateLimited => Msg::AccountErrorRateLimited,
        AccountError::Unavailable => Msg::AccountErrorUnavailable,
        AccountError::BadKey => Msg::AccountErrorBadKey,
    }
}

fn watch_text(
    mut events: MessageReader<TextEntered>,
    mut account: ResMut<Account>,
    mut form: NonSendMut<ActiveForm>,
    lang: Res<Lang>,
) {
    for ev in events.read() {
        let command = match ev.kind {
            TextKind::Name => {
                account.registration_name = Some(ev.value.clone());
                textform::open(&mut form, TextKind::Email, &lang.key("p75.email_prompt"));
                continue;
            }
            TextKind::Email => {
                let Some(name) = account.registration_name.clone() else {
                    continue;
                };
                crate::auth::Command::Register(serde_json::json!({"name":name,"email":ev.value,"locale":if *lang==Lang::PtBr{"pt-BR"}else{"en"}}).to_string())
            }
            TextKind::Key => crate::auth::Command::Login(ev.value.clone()),
            TextKind::BackupKey => crate::auth::Command::RecoveryKey(Some(ev.value.clone())),
            _ => continue,
        };
        if crate::auth::request(command) {
            account.pending = Some(ev.kind);
        }
    }
}

fn text_cancelled(mut events: MessageReader<TextCancelled>, mut account: ResMut<Account>) {
    for _ in events.read() {
        account.key_action = None;
        if matches!(account.pending, Some(TextKind::Name | TextKind::Key)) {
            crate::auth::request(crate::auth::Command::Guest);
        }
        account.pending = None;
    }
}

#[allow(clippy::too_many_arguments)] // one drain for the account replies
fn drain_inbox(
    mut inbox: ResMut<AccountInbox>,
    mut account: ResMut<Account>,
    mut form: NonSendMut<ActiveForm>,
    mut toasts: ResMut<Toasts>,
    time: Res<Time>,
    lang: Res<Lang>,
) {
    for (epoch, msg) in std::mem::take(&mut inbox.0) {
        if epoch != crate::auth::epoch() {
            continue;
        }
        match msg {
            ServerMsg::Registered { key, profile } => {
                super::account_backup::issued_key(&mut account, &key);
                account.profile = Some(profile.clone());
                account.screen = AccountScreen::SaveKey {
                    key,
                    name: profile.name.clone(),
                    saved: false,
                };
                account.pending = None;
                textform::close(&mut form);
            }
            ServerMsg::LoggedIn { profile } => {
                if account
                    .profile
                    .as_ref()
                    .is_some_and(|old| old.name != profile.name)
                {
                    account.recovery_key = None;
                    account.key_action = None;
                    account.key_result = None;
                }
                let was_auto = account.auto_login;
                account.profile = Some(profile.clone());
                account.pending = None;
                account.auto_login = false;
                textform::close(&mut form);
                if !was_auto {
                    toasts.say_with(Msg::AccountLoggedIn, &[("name", profile.name.clone())]);
                }
            }
            ServerMsg::LoggedOut => {
                account.recovery_key = None;
                account.key_action = None;
                account.key_result = None;
                account.profile = None;
                account.screen = AccountScreen::None;
                account.logout_armed = false;
                account.rating_change = None;
                toasts.say(Msg::AccountLoggedOut);
            }
            // P12: settled by the store after `MatchEnd`; the result screen
            // counts it up. The menu chip refreshes from the new profile.
            ServerMsg::RatingChanged {
                before,
                after,
                profile,
            } => {
                account.profile = Some(profile);
                account.rating_change = Some((before, after, time.elapsed_secs_f64()));
            }
            ServerMsg::AccountError { code } => {
                let was_auto = account.auto_login;
                account.auto_login = false;
                account.pending = None;
                if code == AccountError::BadKey && !textform::is_open(&form) {
                    // Post-P14: a stored key failed (or timed out); the
                    // player continues as guest and the UI says so.
                    account.profile = None;
                    toasts.say(Msg::AccountSavedKeyFailed);
                } else if code == AccountError::Unavailable
                    && (was_auto || account.profile.is_some())
                {
                    // P14: the auto-login raced a resumed match; the server
                    // already sent `LoggedIn` for the session. Stay quiet.
                } else if textform::is_open(&form) {
                    textform::set_error(&form, lang.t(error_msg(code)));
                } else {
                    toasts.say(error_msg(code));
                }
            }
            _ => {}
        }
    }
}

/// Export only after the player's explicit recovery-key download action.
#[cfg(target_arch = "wasm32")]
pub(crate) fn download_key(name: &str, key: &str) -> bool {
    #[wasm_bindgen::prelude::wasm_bindgen(inline_js = r#"
    export function downloadAccountKey(name,key) {
      const blob=new Blob([key],{type:'text/plain'}),url=URL.createObjectURL(blob);
      const link=document.createElement('a');link.href=url;link.download=name+'-tbg-key.txt';
      link.click();link.remove();setTimeout(()=>URL.revokeObjectURL(url),0);
    }
    "#)]
    extern "C" {
        #[wasm_bindgen::prelude::wasm_bindgen(catch,js_name=downloadAccountKey)]
        fn export_key(name: &str, key: &str) -> Result<(), wasm_bindgen::JsValue>;
    }
    export_key(name, key).is_ok()
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn download_key(name: &str, key: &str) -> bool {
    crate::platform::export_recovery_key(name, key)
}
