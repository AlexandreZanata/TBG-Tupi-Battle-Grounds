//! Freeze terminal delivery for the seat and its existing reconnect window.

use super::Match;
use crate::chan::CompletedResult;
use tbg_core::{celebration::CastPayload, protocol::ServerMsg, Seat};

impl Match {
    pub(super) fn completed(
        &self,
        seat: Seat,
        outcome: tbg_core::Outcome,
        summary: tbg_core::protocol::PersonalMatchSummary,
        envelope: Option<&crate::store::player_history::Envelope>,
    ) -> CompletedResult {
        let cast = if outcome.winner == Some(seat) {
            self.state
                .celebration_cast()
                .as_ref()
                .and_then(CastPayload::new)
        } else {
            None
        };
        CompletedResult {
            delivered_to: self.players[seat.index()].conn.as_ref().map(|c| c.id),
            start: self.match_start(seat),
            personal: Some(ServerMsg::PersonalMatchResult {
                result_id: self.result_id,
                summary: Box::new(summary),
            }),
            history: envelope.and_then(|e| {
                e.sides[seat.index()]
                    .account
                    .map(|_| ServerMsg::MatchHistoryStatus {
                        result_id: self.result_id,
                        entry_id: e.sides[seat.index()].entry_id,
                        state: tbg_core::protocol::HistorySaveState::Pending,
                    })
            }),
            end: ServerMsg::MatchEnd {
                result_id: self.result_id,
                last_tick: self.state.tick,
                outcome,
                cast,
            },
        }
    }
}
