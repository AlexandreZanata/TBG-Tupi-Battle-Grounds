//! Watching a listed match (P47, BR-LV01, BR-LV02, BR-LV09). A spectator
//! keeps its session but holds no seat: the match only sends it frames. The
//! refusals run in the order the spec fixes: rate limit, idle, live, public,
//! caps.

use std::time::{Duration, Instant};

use tbg_core::protocol::{LiveId, ServerMsg, SpectateRefusal, Token};

use super::live::Listing;
use super::{Lobby, SessionState};
use crate::chan::MatchCmd;

/// BR-LV09: one `Spectate` per second per session.
const SPECTATE_EVERY: Duration = Duration::from_secs(1);

impl Lobby {
    pub(super) fn spectate(&mut self, token: Token, id: LiveId) {
        if let Err(reason) = self.try_watch(token, id) {
            self.send_to(token, ServerMsg::SpectateUnavailable { reason });
        }
    }

    fn try_watch(&mut self, token: Token, id: LiveId) -> Result<(), SpectateRefusal> {
        let now = Instant::now();
        let total: usize = self.live.values().map(|m| m.watchers.len()).sum();
        let per_match = self.config.spectators_per_match;
        let max = self.config.max_spectators;
        let s = self
            .sessions
            .get_mut(&token)
            .ok_or(SpectateRefusal::NotIdle)?;
        if s.last_spectate
            .is_some_and(|t| now.duration_since(t) < SPECTATE_EVERY)
        {
            return Err(SpectateRefusal::RateLimited);
        }
        s.last_spectate = Some(now);
        if !matches!(s.state, SessionState::Idle) {
            return Err(SpectateRefusal::NotIdle);
        }
        let conn = s.conn.clone().ok_or(SpectateRefusal::NotIdle)?;
        let m = self.live.get_mut(&id).ok_or(SpectateRefusal::NotFound)?;
        if m.listing != Listing::Public {
            return Err(SpectateRefusal::NotWatchable);
        }
        if m.watchers.len() >= per_match || total >= max {
            return Err(SpectateRefusal::Full);
        }
        // ponytail: `try_send` can fail on a full match inbox; the refusal then
        // reads as `NotFound`. Upgrade: a retry queue if this ever shows up.
        m.tx.try_send(MatchCmd::Watch { conn })
            .map_err(|_| SpectateRefusal::NotFound)?;
        m.watchers.push(token);
        s.state = SessionState::Watching { id };
        Ok(())
    }

    /// Return a watcher to `Idle` and detach its connection from the match.
    /// A no-op for any other session. Must run while the connection is still
    /// the session's current one, so the match can match the `conn_id`.
    pub(super) fn end_watch(&mut self, token: Token) {
        let Some(s) = self.sessions.get_mut(&token) else {
            return;
        };
        let SessionState::Watching { id } = s.state else {
            return;
        };
        s.state = SessionState::Idle;
        let conn_id = s.conn.as_ref().map(|c| c.id);
        let Some(m) = self.live.get_mut(&id) else {
            return;
        };
        m.watchers.retain(|t| *t != token);
        if let Some(conn_id) = conn_id {
            // ponytail: a full match inbox can drop this `Unwatch`; the match
            // then keeps sending frames until the socket closes.
            let _ = m.tx.try_send(MatchCmd::Unwatch { conn_id });
        }
    }
}
