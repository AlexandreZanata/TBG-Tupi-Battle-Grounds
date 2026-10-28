//! Account sessions (BR-P01–BR-P05): register, login, logout, and the
//! login-takeover rule (BR-P03). The lobby never awaits the store: each
//! command spawns a tiny task whose reply comes back as a `LobbyCmd`.
//! The plain key exists only in the `Registered` reply, never in logs.

use std::net::IpAddr;
use std::time::Instant;

use sha2::Digest;
use tbg_core::protocol::{AccountError, ServerMsg, Token};

use super::{send, Lobby, SessionState};
use crate::chan::{ConnEvent, LobbyCmd};
use crate::names::{display_name, random_bytes};

fn sha256(secret: &[u8; 16]) -> [u8; 32] {
    sha2::Sha256::digest(secret).into()
}

pub(super) fn deny(conn: &Option<crate::chan::ConnHandle>, code: AccountError) {
    send(conn, ConnEvent::Send(ServerMsg::AccountError { code }));
}

impl Lobby {
    /// `(conn_id, ip)` when the session exists and is `Idle`; otherwise
    /// answers `Unavailable` (or nothing when gone) and returns `None`.
    fn idle_conn(&mut self, token: &Token) -> Option<(u64, IpAddr)> {
        match self.session(token) {
            Some(s) if matches!(s.state, SessionState::Idle) => {
                s.conn.as_ref().map(|c| (c.id, s.ip))
            }
            Some(s) => {
                deny(&s.conn, AccountError::Unavailable);
                None
            }
            None => None,
        }
    }

    fn account_error(&mut self, token: &Token, code: AccountError) {
        if let Some(s) = self.session(token) {
            deny(&s.conn, code);
        }
    }

    pub(super) fn register(&mut self, token: Token, name: String) {
        let Some((conn_id, ip)) = self.idle_conn(&token) else {
            return;
        };
        if tbg_core::account::check_name(&name).is_err() {
            return self.account_error(&token, AccountError::NameInvalid);
        }
        if !self
            .limits
            .check_register(ip, self.config.register_per_ip_hour as u32, Instant::now())
        {
            return self.account_error(&token, AccountError::RateLimited);
        }
        let Some(store) = self.store.clone() else {
            return self.account_error(&token, AccountError::Unavailable);
        };
        let tx = self.self_tx.clone();
        tokio::spawn(async move {
            let secret = tbg_core::account::encode_secret(&random_bytes::<10>());
            let key = tbg_core::account::format_key(&name, &secret);
            let hash = sha256(&secret);
            let result = store
                .register(name, hash)
                .await
                .map(|(id, profile)| (id, profile, key));
            let _ = tx
                .send(LobbyCmd::RegisterDone {
                    token,
                    conn_id,
                    result,
                })
                .await;
        });
    }

    pub(super) fn login(&mut self, token: Token, key: String) {
        let Some((conn_id, ip)) = self.idle_conn(&token) else {
            return;
        };
        let Some((name, secret)) = tbg_core::account::parse_key(&key) else {
            return self.account_error(&token, AccountError::BadKey);
        };
        if !self
            .limits
            .check_login(ip, self.config.login_per_ip_min as u32, Instant::now())
        {
            return self.account_error(&token, AccountError::RateLimited);
        }
        let Some(store) = self.store.clone() else {
            return self.account_error(&token, AccountError::Unavailable);
        };
        let tx = self.self_tx.clone();
        tokio::spawn(async move {
            let name_lower = name.to_ascii_lowercase();
            let result = store.login(name_lower, sha256(&secret)).await;
            let _ = tx
                .send(LobbyCmd::LoginDone {
                    token,
                    conn_id,
                    result,
                })
                .await;
        });
    }

    pub(super) fn logout(&mut self, token: Token) {
        let Some(s) = self.session(&token) else {
            return;
        };
        if !matches!(s.state, SessionState::Idle) {
            deny(&s.conn, AccountError::Unavailable);
            return;
        }
        s.account = None;
        s.name = display_name();
        send(&s.conn, ConnEvent::Send(ServerMsg::LoggedOut));
    }
}
