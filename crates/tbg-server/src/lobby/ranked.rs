//! Ranked queue and rating delivery (BR-R01, BR-R02, BR-R06). Queues never
//! mix; pairing stays FIFO and ignores rating. Rating math lives only in
//! the store transaction; the lobby only routes its outcome.

use tbg_core::protocol::{AccountError, ErrorCode, RatingResults, ServerMsg, Token};

use super::{send, Lobby, SessionState};
use crate::chan::ConnEvent;

impl Lobby {
    /// Enqueue an account for ranked play, or pair with the longest waiter.
    pub(super) fn ranked(&mut self, token: Token) {
        match self.session(&token) {
            Some(s) if matches!(s.state, SessionState::Idle) => {
                if s.account.is_none() {
                    send(
                        &s.conn,
                        ConnEvent::Send(ServerMsg::Error {
                            code: ErrorCode::LoginRequired,
                        }),
                    );
                    return;
                }
            }
            Some(s) => {
                send(
                    &s.conn,
                    ConnEvent::Send(ServerMsg::AccountError {
                        code: AccountError::Unavailable,
                    }),
                );
                return;
            }
            None => return,
        }
        self.leave_waiting(token);
        match self.ranked_queue.pop_front() {
            Some(other) => self.pair([other, token], tbg_core::protocol::MatchMode::Ranked),
            None => {
                self.ranked_queue.push_back(token);
                if let Some(s) = self.session(&token) {
                    s.state = SessionState::Queued;
                    send(&s.conn, ConnEvent::Send(ServerMsg::Queued));
                }
            }
        }
    }

    /// Deliver settled ratings: refresh the cached profile and tell the
    /// session's current connection (if any) what changed (BR-R06).
    pub(super) fn ranked_done(&mut self, tokens: [Token; 2], results: RatingResults) {
        for (token, (before, after, profile)) in tokens.into_iter().zip(results) {
            if let Some(s) = self.session(&token) {
                if let Some((id, _)) = s.account {
                    s.account = Some((id, profile.clone()));
                    send(
                        &s.conn,
                        ConnEvent::Send(ServerMsg::RatingChanged {
                            before,
                            after,
                            profile,
                        }),
                    );
                }
            }
        }
    }
}
