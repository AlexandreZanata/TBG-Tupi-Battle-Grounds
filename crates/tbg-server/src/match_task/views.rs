//! Per-seat sending: ack views, catch-up views, slow-client drops.
//! The `pending` buffers are the only event source; a view to a seat
//! drains only that seat's buffer, so events are never lost or duplicated.

use tokio::sync::mpsc;

use tbg_core::protocol::ServerMsg;
use tbg_core::Seat;

use crate::chan::ConnEvent;

use super::Match;

impl Match {
    /// Send this tick's view to one seat and drain its buffer.
    pub(super) fn send_view(&mut self, seat: Seat) {
        let events = std::mem::take(&mut self.pending[seat.index()]);
        let view = self.state.view_for(seat, &events);
        self.send(seat, ServerMsg::View(Box::new(view)));
    }

    pub(super) fn count_success(&mut self, seat: Seat, intent: tbg_core::PlayerIntention) {
        let stats = &mut self.stats[seat.index()];
        match intent {
            tbg_core::PlayerIntention::Purchase { .. } => {
                if let Some(u) = self.state.units.last() {
                    *stats
                        .purchases
                        .entry(format!("{kind:?}", kind = u.kind))
                        .or_default() += 1;
                    // P12: faction mix for the ranked result row.
                    stats.factions[tbg_core::content::def(u.kind).faction.index()] += 1;
                }
            }
            tbg_core::PlayerIntention::Evolve { .. } => stats.evolutions += 1,
            tbg_core::PlayerIntention::ChooseAugment { .. }
            | tbg_core::PlayerIntention::Forgo { .. }
            | tbg_core::PlayerIntention::Fortify { .. } => {}
        }
    }

    pub(super) fn detach(&mut self, seat: Seat) {
        self.players[seat.index()].conn = None;
        self.detached_at[seat.index()] = Some(std::time::Instant::now());
    }

    /// Never blocks: a full client queue means a too-slow client; drop it.
    pub(super) fn send(&mut self, seat: Seat, msg: ServerMsg) {
        let Some(conn) = &self.players[seat.index()].conn else {
            return;
        };
        match conn.tx.try_send(ConnEvent::Send(msg)) {
            Ok(()) => {}
            Err(mpsc::error::TrySendError::Full(_)) => {
                let _ = conn.tx.try_send(ConnEvent::Close);
                self.detach(seat);
            }
            Err(mpsc::error::TrySendError::Closed(_)) => self.detach(seat),
        }
    }
}
