//! P60: the frozen ranked envelope of a finished match. Pure: no I/O, no
//! SQL; the finalizer journals and settles it.

use std::time::Instant;

use tbg_core::content::RUNNING_START;
use tbg_core::ruleset_version::RULESET_VERSION;
use tbg_core::Seat;

use super::Match;
use crate::ranked::{source_hash, EndDetail, IntegritySeat, MatchKey, RankedFinal};

impl Match {
    /// `None` for a match without two accounts or without an outcome. Equal
    /// frozen account ids are an internal invariant failure: quarantined
    /// here (logged), never settled as a result.
    pub(super) fn ranked_final(&self, key: MatchKey, started: Instant) -> Option<RankedFinal> {
        let (Some((a, _)), Some((b, _))) = (self.players[0].account, self.players[1].account)
        else {
            return None;
        };
        if a == b {
            tracing::error!("ranked match froze one account in both seats; not settled");
            return None;
        }
        let outcome = self.state.outcome?;
        let seats = Seat::BOTH.map(|seat| {
            let st = &self.stats[seat.index()];
            let p = self.state.player(seat);
            IntegritySeat {
                purchases: st.purchases.values().sum(),
                evolutions: st.evolutions,
                fortifications: st.fortifications,
                augments_applied: p.augments.iter().map(|c| u32::from(*c)).sum(),
                reconnects: st.reattached,
                surrender_tick: st.surrender_tick,
                final_hp: p.stronghold_hp,
                final_max_hp: self.state.stronghold(seat).max_hp,
            }
        });
        Some(RankedFinal {
            key,
            accounts: [a, b],
            outcome,
            end_detail: self.end_detail.unwrap_or(EndDetail::Stronghold),
            combat_ticks: self.state.tick.saturating_sub(RUNNING_START),
            wall_ms: u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
            reached_running: self.reached_running,
            ruleset_version: RULESET_VERSION,
            ruleset_source_hash: source_hash(),
            seats,
            factions: [self.stats[0].factions, self.stats[1].factions],
            arcanas: self.state.players.each_ref().map(|p| p.arcanas),
        })
    }
}
