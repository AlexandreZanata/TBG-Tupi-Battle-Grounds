//! The one JSON log line per finished match.

use std::time::Instant;

use super::Match;
use crate::metrics::percentile;

#[derive(Default)]
pub(crate) struct SeatStats {
    pub(super) intents: u32,
    pub(super) rejections: u32,
    pub(super) rtts: Vec<u16>,
    pub(super) reattached: u32,
    pub(super) purchases: std::collections::BTreeMap<String, u32>,
    pub(super) evolutions: u32,
    /// P12: purchases per faction index (for the ranked result row).
    pub(super) factions: [u32; 4],
    /// BR-M06: this seat tapped Surrender (additive report key).
    pub(super) surrendered: bool,
}

impl Match {
    pub(super) fn log_end(
        &mut self,
        seed: u64,
        started: Instant,
        step_us_mean: u64,
        step_us_max: u64,
        late: u32,
        max_late_ms: u64,
    ) -> serde_json::Value {
        let outcome = self.state.outcome;
        let seats: Vec<serde_json::Value> = self
            .stats
            .iter_mut()
            .map(|s| {
                s.rtts.sort_unstable();
                serde_json::json!({
                    "rtt_p50": percentile(&s.rtts, 0.5), "rtt_p95": percentile(&s.rtts, 0.95),
                    "intents": s.intents, "rejections": s.rejections, "reconnects": s.reattached,
                    "purchases": s.purchases, "evolutions": s.evolutions,
                    "surrendered": s.surrendered,
                })
            })
            .collect();
        let augments: Vec<Vec<(u8, u16)>> = self
            .state
            .players
            .iter()
            .map(|p| {
                p.augments
                    .iter()
                    .enumerate()
                    .filter(|(_, copies)| **copies > 0)
                    .map(|(id, copies)| (id as u8, *copies))
                    .collect()
            })
            .collect();
        let line = serde_json::json!({
            "seed": seed, "duration_s": started.elapsed().as_secs(), "ticks": self.state.tick,
            "winner": outcome.and_then(|o| o.winner).map(|w| format!("{w:?}")),
            "reason": outcome.map(|o| format!("{:?}", o.reason)),
            "seats": seats, "step_us_mean": step_us_mean, "step_us_max": step_us_max,
            "late_ticks": late, "max_lateness_ms": max_late_ms,
            // D-40 (BR-AR07): the presets this match applied.
            "arcanas": self.state.players.each_ref().map(|p| p.arcanas.ranks),
            // Owner revision 2026-10-29: economy and Augment telemetry.
            "interest_paid": self.state.players.each_ref().map(|p| p.interest_paid),
            "augments": augments,
        });
        tracing::info!(target: "match_end", "{line}");
        line
    }
}
