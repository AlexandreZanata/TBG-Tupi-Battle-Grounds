//! Ban cancellation and the single authorized moderation outcome: a seat forfeit.
use super::*;
impl Lobby {
    pub(super) fn moderation_ban(&mut self, account: AccountId) -> bool {
        let tokens: Vec<_> = self
            .sessions
            .iter()
            .filter(|(_, s)| s.account.as_ref().is_some_and(|(id, _)| *id == account))
            .map(|(t, _)| *t)
            .collect();
        let mut delivered = true;
        for token in tokens {
            let state = self.config.moderation.state(account);
            self.send_to(
                token,
                ServerMsg::AccountSuspended {
                    until_unix: state
                        .ban_until
                        .filter(|v| *v != crate::store::moderation::PERMANENT)
                        .map(|v| v as u64),
                    reason: state
                        .reason
                        .unwrap_or(tbg_core::protocol::SuspensionReason::Other),
                },
            );
            if let Some(s) = self.sessions.get(&token) {
                if let SessionState::InMatch { tx, seat } = &s.state {
                    delivered &= tx
                        .try_send(MatchCmd::ModerationForfeit { seat: *seat })
                        .is_ok();
                }
            }
            self.end_watch(token);
            self.cancel_direct(token);
            self.decline_rematch(token);
            self.leave_waiting(token);
        }
        delivered
    }
}
