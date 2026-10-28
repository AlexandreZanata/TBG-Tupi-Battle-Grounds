//! Per-seat views and the one JSON log line per finished match.

use std::time::Instant;

use tbg_core::protocol::ServerMsg;
use tbg_core::Seat;

use super::Match;
use crate::metrics::percentile;

impl Match {
    pub(super) fn send_views(&mut self) {
        let events = std::mem::take(&mut self.events);
        for seat in Seat::BOTH {
            let view = self.state.view_for(seat, &events);
            self.send(seat, ServerMsg::View(view));
        }
    }

    pub(super) fn log_end(
        &mut self,
        seed: u64,
        started: Instant,
        step_us_mean: u64,
        step_us_max: u64,
        late: u32,
        max_late_ms: u64,
    ) {
        let outcome = self.state.outcome;
        let seats: Vec<serde_json::Value> = self
            .stats
            .iter_mut()
            .map(|s| {
                s.rtts.sort_unstable();
                serde_json::json!({
                    "rtt_p50": percentile(&s.rtts, 0.5), "rtt_p95": percentile(&s.rtts, 0.95),
                    "intents": s.intents, "rejections": s.rejections, "reconnects": s.reattached,
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
    }
}
