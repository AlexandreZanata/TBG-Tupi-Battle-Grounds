//! Session resume, disconnects, and periodic cleanup.

use std::time::{Duration, Instant};

use tbg_core::protocol::Token;

use super::{send, Lobby, Session, SessionState, ROOM_TTL};
use crate::chan::{ConnEvent, ConnHandle, MatchCmd};
use crate::names::{display_name, random_bytes};

impl Lobby {
    /// Resume a known session (taking over any older socket) or create one.
    pub(super) fn hello(&mut self, resume: Option<Token>, conn: ConnHandle) -> (Token, String) {
        if let Some(s) = resume.and_then(|t| self.sessions.iter_mut().find(|s| s.token == t)) {
            send(&s.conn, ConnEvent::Close);
            if let SessionState::InMatch { tx, seat } = &s.state {
                let _ = tx.try_send(MatchCmd::Attach {
                    seat: *seat,
                    conn: conn.clone(),
                });
                let _ = conn.tx.try_send(ConnEvent::EnterMatch {
                    seat: *seat,
                    tx: tx.clone(),
                });
            }
            s.conn = Some(conn);
            s.detached_at = None;
            return (s.token, s.name.clone());
        }
        let session = Session {
            token: random_bytes::<16>(),
            name: display_name(),
            conn: Some(conn),
            state: SessionState::Idle,
            detached_at: None,
        };
        let pair = (session.token, session.name.clone());
        self.sessions.push(session);
        pair
    }

    pub(super) fn disconnected(&mut self, token: Token, conn_id: u64) {
        let Some(s) = self.session(&token) else {
            return;
        };
        if s.conn.as_ref().is_none_or(|c| c.id != conn_id) {
            return;
        }
        s.conn = None;
        s.detached_at = Some(Instant::now());
        self.leave_waiting(token);
    }

    /// Drop expired rooms and long-gone sessions that are not in a match.
    pub(super) fn cleanup(&mut self) {
        let now = Instant::now();
        self.rooms.retain(|(.., created)| now - *created < ROOM_TTL);
        let window = Duration::from_secs(self.config.reconnect_secs);
        self.sessions.retain(|s| {
            let gone = s.detached_at.is_some_and(|t| now - t > window);
            !gone || matches!(s.state, SessionState::InMatch { .. })
        });
        let tokens: Vec<Token> = self.sessions.iter().map(|s| s.token).collect();
        self.queue.retain(|t| tokens.contains(t));
        self.queued_at.retain(|t, _| self.queue.contains(t));
    }
}
