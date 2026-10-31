//! The registry of running public matches (BR-LV01): who plays, the progress
//! the match task reports, and who watches. Only `Listing::Public` matches
//! enter it; a private match never reaches this map, so its id never leaves
//! the server (BR-LV06).

use std::time::Instant;

use tokio::sync::mpsc;

use tbg_core::protocol::{LiveId, LivePlayer, MatchMode, Token};
use tbg_core::{Arena, StrongholdKind};

use super::{Lobby, SessionState};
use crate::chan::MatchCmd;
use crate::live::LiveProgress;

/// Where a match came from, decided where it is paired (BR-LV01). Never
/// inferred from `MatchMode`: rooms share `MatchMode::Casual` with the quick
/// queue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Listing {
    Public,
    Private,
}

pub(crate) struct LiveMatch {
    pub(super) tx: mpsc::Sender<MatchCmd>,
    pub(super) listing: Listing,
    pub(super) mode: MatchMode,
    pub(super) arena: Arena,
    pub(super) players: [LivePlayer; 2],
    pub(super) kinds: [StrongholdKind; 2],
    pub(super) started: Instant,
    pub(super) progress: LiveProgress,
    /// Sessions watching this match; caps are checked against it (BR-LV09).
    pub(super) watchers: Vec<Token>,
}

impl Lobby {
    /// Make a freshly paired public match visible. Private matches never call it.
    pub(super) fn register_live(&mut self, id: LiveId, m: LiveMatch) {
        self.live.insert(id, m);
    }

    pub(super) fn live_progress(&mut self, id: LiveId, progress: LiveProgress) {
        if let Some(m) = self.live.get_mut(&id) {
            m.progress = progress;
        }
    }

    /// A listed match ended: drop it from the registry and return its
    /// watchers to `Idle` (their `SpectateEnd` already went out).
    pub(super) fn live_ended(&mut self, id: LiveId) {
        let Some(m) = self.live.remove(&id) else {
            return;
        };
        for token in m.watchers {
            if let Some(s) = self.sessions.get_mut(&token) {
                if matches!(s.state, SessionState::Watching { id: w } if w == id) {
                    s.state = SessionState::Idle;
                }
            }
        }
    }

    /// Draw a random, non-sequential id no live match uses (BR-LV08).
    pub(super) fn fresh_live_id(&self) -> LiveId {
        loop {
            let id = u64::from_le_bytes(crate::names::random_bytes::<8>());
            if id != 0 && !self.live.contains_key(&id) {
                return id;
            }
        }
    }
}
