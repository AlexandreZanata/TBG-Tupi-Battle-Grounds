//! Fan-out to spectators (P47, BR-LV02/BR-LV03/BR-LV09): one public view per
//! view tick, encoded once and shared as a frame. A spectator never reaches
//! `apply_intention`, and a full or closed spectator queue drops only that
//! spectator: it can never block or detach a seat.

use axum::body::Bytes;
use tokio::sync::mpsc::error::TrySendError;

use tbg_core::protocol::{self, LivePlayer, ServerMsg, SpectateRefusal};
use tbg_core::{Event, Outcome};

use crate::chan::{ConnEvent, ConnHandle};

use super::Match;

impl Match {
    /// A new spectator gets its identity and a full view at once. Refused
    /// (`NotWatchable`) only if the match is not listed, which the lobby
    /// already prevents (BR-LV01).
    pub(super) fn watch(&mut self, conn: ConnHandle) {
        let Some(id) = self.live else {
            let refusal = SpectateRefusal::NotWatchable;
            let _ = conn
                .tx
                .try_send(ConnEvent::Send(ServerMsg::SpectateUnavailable {
                    reason: refusal,
                }));
            return;
        };
        let start = ServerMsg::SpectateStart {
            id,
            mode: self.mode,
            arena: self.state.arena,
            players: self.players.each_ref().map(|p| LivePlayer {
                name: p.name.clone(),
                rating: p.account.map(|(_, rating)| rating),
            }),
            arcanas: self.state.players.each_ref().map(|p| p.arcanas),
        };
        let count = self.spectators.len() as u16 + 1;
        let view = ServerMsg::SpectatorView(Box::new(self.state.public_view(&[], count)));
        if conn.tx.try_send(ConnEvent::Send(start)).is_err()
            || conn.tx.try_send(ConnEvent::Send(view)).is_err()
        {
            return;
        }
        // Events kept for the others belong to their next view, not this one.
        self.spectators.push(conn);
    }

    /// Keep public events for the spectators' next view; only while someone
    /// watches, so an unwatched match pays nothing.
    pub(super) fn note_spectate_events(&mut self, events: &[Event]) {
        if !self.spectators.is_empty() {
            self.pending_spectate.extend(events.iter().cloned());
        }
    }

    /// One shared frame per view tick, whatever the spectator count (budget:
    /// exactly one encode per match per view tick).
    pub(super) fn send_spectator_views(&mut self) {
        if self.spectators.is_empty() {
            self.pending_spectate.clear();
            return;
        }
        let count = self.spectators.len() as u16;
        let view = self.state.public_view(&self.pending_spectate, count);
        self.pending_spectate.clear();
        let frame = Bytes::from(protocol::encode(&ServerMsg::SpectatorView(Box::new(view))));
        self.spectators
            .retain(|conn| deliver(conn, ConnEvent::Frame(frame.clone())));
    }

    /// The match ended: every spectator learns the outcome and stops watching.
    pub(super) fn end_spectators(&mut self, outcome: Outcome) {
        for conn in self.spectators.drain(..) {
            let _ = conn.tx.try_send(ConnEvent::Send(ServerMsg::SpectateEnd {
                outcome: Some(outcome),
            }));
        }
        self.pending_spectate.clear();
    }
}

/// `false` drops the spectator: a full queue (too slow) or a closed one.
/// Asking a full queue to close can fail too; the socket then dies on write.
// ponytail: a full queue whose close cannot be queued waits for the socket's
// write-buffer error to clean up; upgrade: a per-spectator lag counter.
fn deliver(conn: &ConnHandle, event: ConnEvent) -> bool {
    match conn.tx.try_send(event) {
        Ok(()) => true,
        Err(TrySendError::Full(_)) => {
            let _ = conn.tx.try_send(ConnEvent::Close);
            false
        }
        Err(TrySendError::Closed(_)) => false,
    }
}
