//! Ranked queue and rating delivery (BR-R01, BR-R02, BR-R06). Queues never
//! mix; pairing stays FIFO and ignores rating. Rating math lives only in
//! the store transaction; the lobby only routes its outcome.

use tbg_core::protocol::{AccountError, ErrorCode, ServerMsg, Token};

use super::live::Listing;
use super::{send, Lobby, SessionState};
use crate::chan::ConnEvent;

impl Lobby {
    /// Enqueue an account for ranked play, or pair with the longest waiter.
    pub(super) fn ranked(&mut self, token: Token) {
        // P60 §7: while a finished ranked result cannot settle, new ranked
        // admission is explicitly unavailable (Casual is not).
        let stuck = self
            .ranked
            .as_ref()
            .is_some_and(|r| r.unavailable.load(std::sync::atomic::Ordering::Relaxed));
        match self.session(&token) {
            Some(s) if matches!(s.state, SessionState::Idle) => {
                if stuck {
                    send(
                        &s.conn,
                        ConnEvent::Send(ServerMsg::AccountError {
                            code: AccountError::Unavailable,
                        }),
                    );
                    return;
                }
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
            Some(other) => self.pair(
                [other, token],
                tbg_core::protocol::MatchMode::Ranked,
                false,
                Listing::Public,
            ),
            None => {
                self.ranked_queue.push_back(token);
                if let Some(s) = self.session(&token) {
                    s.state = SessionState::Queued;
                    send(&s.conn, ConnEvent::Send(ServerMsg::Queued));
                }
            }
        }
    }

    /// Deliver a settlement (BR-R06, P60 §7): find each seat's **frozen
    /// account id** among live sessions (after logout, re-login or takeover
    /// alike) and refresh it only when this sequence is newer than what its
    /// cached profile already contains. A token alone never selects an
    /// account; at most one session holds an account (BR-P03).
    pub(super) fn ranked_done(&mut self, receipt: crate::ranked::RankedReceipt) {
        for (account, (before, after, profile)) in receipt.accounts.into_iter().zip(receipt.results)
        {
            let seen = self.ranked_seen.get(&account).copied().unwrap_or(0);
            if receipt.sequence <= seen {
                continue;
            }
            self.ranked_seen.insert(account, receipt.sequence);
            let holder = self
                .sessions
                .values_mut()
                .find(|s| s.account.as_ref().is_some_and(|(id, _)| *id == account));
            if let Some(s) = holder {
                s.account = Some((account, profile.clone()));
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
