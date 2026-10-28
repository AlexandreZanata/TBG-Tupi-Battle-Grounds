//! Store replies for account commands (BR-P03, BR-P05). Each reply is
//! validated against the session's current connection and state: a stale
//! reply (reconnect, queue join, disconnect in between) is dropped or
//! answered without changing anything.

use tbg_core::protocol::{AccountError, ErrorCode, Profile, ServerMsg, Token};

use super::accounts::deny;
use super::{send, Lobby, SessionState};
use crate::chan::{ConnEvent, MatchCmd};
use crate::store::AccountId;

impl Lobby {
    pub(super) fn register_done(
        &mut self,
        token: Token,
        conn_id: u64,
        result: Result<(AccountId, Profile, String), AccountError>,
    ) {
        let Some(s) = self.session(&token) else {
            return;
        };
        if s.conn.as_ref().is_none_or(|c| c.id != conn_id) {
            return;
        }
        match result {
            Err(code) => deny(&s.conn, code),
            Ok((id, profile, key)) => {
                // The key belongs to this session even if it queued meanwhile;
                // the account attaches only while still Idle (BR-P04 guard).
                if matches!(s.state, SessionState::Idle) {
                    s.account = Some((id, profile.clone()));
                    s.name = profile.name.clone();
                }
                send(
                    &s.conn,
                    ConnEvent::Send(ServerMsg::Registered { key, profile }),
                );
            }
        }
    }

    pub(super) fn login_done(
        &mut self,
        token: Token,
        conn_id: u64,
        result: Result<(AccountId, Profile), AccountError>,
    ) {
        let Ok((id, profile)) = result else {
            let code = result.unwrap_err();
            if let Some(s) = self.session(&token) {
                if s.conn.as_ref().is_some_and(|c| c.id == conn_id) {
                    deny(&s.conn, code);
                }
            }
            return;
        };
        // Validate the new session before touching the old one (BR-P03).
        let new_conn = match self.session(&token) {
            Some(s)
                if s.conn.as_ref().is_some_and(|c| c.id == conn_id)
                    && matches!(s.state, SessionState::Idle) =>
            {
                s.conn.clone()
            }
            _ => return,
        };
        // BR-P03: the new login takes over; the old socket is closed, and a
        // running match is re-attached to the new connection (its token is
        // updated in the match task so the end report finds this session).
        let stolen = match self.sessions.iter().find_map(|(t, s)| {
            (*t != token && s.account.as_ref().is_some_and(|(a, _)| *a == id)).then_some(*t)
        }) {
            Some(old_token) => {
                let mut stolen = None;
                if let Some(old) = self.sessions.get_mut(&old_token) {
                    send(
                        &old.conn,
                        ConnEvent::Send(ServerMsg::Error {
                            code: ErrorCode::SessionTakenOver,
                        }),
                    );
                    send(&old.conn, ConnEvent::Close);
                    if let SessionState::InMatch { tx, seat } = &old.state {
                        if let Some(nc) = &new_conn {
                            let _ = tx.try_send(MatchCmd::Attach {
                                seat: *seat,
                                conn: nc.clone(),
                                new_token: Some(token),
                            });
                            let _ = nc.tx.try_send(ConnEvent::EnterMatch {
                                seat: *seat,
                                tx: tx.clone(),
                            });
                            stolen = Some((tx.clone(), *seat));
                        }
                    }
                }
                self.leave_waiting(old_token);
                // Drop the old session outright: the closed device
                // auto-resumes with its old token and must get a fresh
                // guest session, not the match it just lost.
                self.sessions.remove(&old_token);
                stolen
            }
            None => None,
        };
        let Some(s) = self.session(&token) else {
            return;
        };
        s.account = Some((id, profile.clone()));
        s.name = profile.name.clone();
        if let Some((tx, seat)) = stolen {
            s.state = SessionState::InMatch { tx, seat };
        }
        send(&s.conn, ConnEvent::Send(ServerMsg::LoggedIn { profile }));
    }
}
