//! Tap-to-acknowledgement latency: send time per intention seq, resolved
//! by the first `View` (or `Rejected`) that covers it. Keeps the last 20.

use std::collections::VecDeque;

#[derive(Default)]
pub(super) struct AckTracker {
    pending: Vec<(u32, f64)>,
    samples: VecDeque<f64>,
}

impl AckTracker {
    pub(super) fn sent(&mut self, seq: u32, now: f64) {
        self.pending.push((seq, now));
        if self.pending.len() > 64 {
            self.pending.remove(0);
        }
    }

    pub(super) fn resolved(&mut self, last_seq: u32, now: f64) {
        let mut i = 0;
        while i < self.pending.len() {
            if self.pending[i].0 <= last_seq {
                let (_, t0) = self.pending.remove(i);
                self.samples.push_back(now - t0);
                if self.samples.len() > 20 {
                    self.samples.pop_front();
                }
            } else {
                i += 1;
            }
        }
    }

    /// p50 of the last 20 ack latencies in ms (`None` before any ack).
    pub(super) fn ms(&self) -> Option<f64> {
        if self.samples.is_empty() {
            return None;
        }
        let mut v: Vec<f64> = self.samples.iter().copied().collect();
        v.sort_by(f64::total_cmp);
        Some(v[v.len() / 2] * 1000.0)
    }
}
