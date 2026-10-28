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
                })
            })
            .collect();
        let line = serde_json::json!({
            "seed": seed, "duration_s": started.elapsed().as_secs(), "ticks": self.state.tick,
            "winner": outcome.and_then(|o| o.winner).map(|w| format!("{w:?}")),
            "reason": outcome.map(|o| format!("{:?}", o.reason)),
            "seats": seats, "step_us_mean": step_us_mean, "step_us_max": step_us_max,
            "late_ticks": late, "max_lateness_ms": max_late_ms,
        });
        tracing::info!(target: "match_end", "{line}");
        line
    }
}
