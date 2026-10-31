//! Account panel state (BR-P04, BR-P05): profile chip, save-your-key
//! screen, login/logout. Server replies arrive through `AccountInbox`
//! (filled by `sync::messages`); text entry comes from `textform` events.

use bevy::prelude::*;
use tbg_core::protocol::{AccountError, ClientMsg, Profile, ServerMsg};

use super::textform::{self, ActiveForm, TextCancelled, TextEntered, TextKind};
use super::Toasts;
use crate::config;
use crate::link::MatchLink;
use crate::net::{Mode, NetLink};

/// Replies the account panel owns (filled by `sync::messages`).
#[derive(Resource, Default)]
pub struct AccountInbox(pub Vec<ServerMsg>);

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
        .add_systems(Update, (watch_text, text_cancelled, drain_inbox));
}

fn error_text(code: AccountError) -> &'static str {
    match code {
        AccountError::NameTaken => "That name is taken",
        AccountError::NameInvalid => "3-16 letters, digits or _; start with a letter",
        AccountError::RateLimited => "Too many tries, wait a bit",
        AccountError::Unavailable => "Finish your match first",
        AccountError::BadKey => "That key didn't work",
    }
}

/// A Net link that sends account messages: reuse the idle probe socket,
/// else open a listen-only one (it sends nothing by itself).
fn ensure_link(link: &mut MatchLink, now: f64) -> &mut NetLink {
    if let MatchLink::Net(n) = link {
        if n.reuse(Mode::Probe, now) {
            return match link {
                MatchLink::Net(n) => n,
                _ => unreachable!(),
            };
        }
    }
    *link = MatchLink::Net(Box::new(NetLink::new(Mode::Probe, now)));
    match link {
        MatchLink::Net(n) => n,
        _ => unreachable!(),
    }
}

fn watch_text(
    mut events: MessageReader<TextEntered>,
    mut link: NonSendMut<MatchLink>,
    mut account: ResMut<Account>,
    time: Res<Time>,
) {
    for ev in events.read() {
        let now = time.elapsed_secs_f64();
        let req = match ev.kind {
            TextKind::Name => ClientMsg::Register {
                name: ev.value.clone(),
            },
            TextKind::Room => continue,
            TextKind::Key => ClientMsg::Login {
                key: ev.value.clone(),
            },
        };
        ensure_link(&mut link, now).send(req, now);
        account.pending = Some(ev.kind);
    }
}

fn text_cancelled(mut events: MessageReader<TextCancelled>, mut account: ResMut<Account>) {
    for _ in events.read() {
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
) {
    for msg in std::mem::take(&mut inbox.0) {
        match msg {
            ServerMsg::Registered { key, profile } => {
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
                let was_auto = account.auto_login;
                account.profile = Some(profile.clone());
                account.pending = None;
                account.auto_login = false;
                textform::close(&mut form);
                if !was_auto {
                    toasts.push(format!("Logged in as {}", profile.name));
                }
            }
            ServerMsg::LoggedOut => {
                account.profile = None;
                account.screen = AccountScreen::None;
                account.logout_armed = false;
                account.rating_change = None;
                config::delete_account_key();
                toasts.push("Logged out");
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
                    toasts.push("Saved key didn't work - playing as guest");
                } else if code == AccountError::Unavailable
                    && (was_auto || account.profile.is_some())
                {
                    // P14: the auto-login raced a resumed match; the server
                    // already sent `LoggedIn` for the session. Stay quiet.
                } else if textform::is_open(&form) {
                    textform::set_error(&form, error_text(code));
                } else {
                    toasts.push(error_text(code));
                }
            }
            _ => {}
        }
    }
}

/// Download `<Name>-tbg-key.txt` through a `data:` anchor (P11).
#[cfg(target_arch = "wasm32")]
pub(crate) fn download_key(name: &str, key: &str) {
    use wasm_bindgen::JsCast;

    let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    let Ok(el) = doc.create_element("a") else {
        return;
    };
    let Ok(a) = el.dyn_into::<web_sys::HtmlAnchorElement>() else {
        return;
    };
    // The key alphabet needs no URL escaping (alphanumerics and `-`).
    a.set_href(&format!("data:text/plain,{key}"));
    a.set_download(&format!("{name}-tbg-key.txt"));
    a.click();
    a.remove();
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn download_key(name: &str, key: &str) {
    if let Some(dir) = crate::platform::storage::data_dir() {
        // Account names are server-validated; still restrict the filename locally.
        let safe: String = name
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        let path = dir.join(format!("{safe}-tbg-key.txt"));
        let mut options = std::fs::OpenOptions::new();
        options.create(true).truncate(true).write(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        use std::io::Write;
        if let Ok(mut file) = options.open(path) {
            let _ = file.write_all(key.as_bytes());
        }
    }
}
