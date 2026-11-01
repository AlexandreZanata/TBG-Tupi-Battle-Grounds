//! Live counters and RTT samples for `GET /metrics` (loopback only).

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
use std::sync::Mutex;
use std::time::{Duration, Instant};

const RTT_WINDOW: Duration = Duration::from_secs(60);

pub struct Metrics {
    pub connections: AtomicU64,
    pub matches_running: AtomicU64,
    pub matches_finished: AtomicU64,
    pub queue_len: AtomicU64,
    pub bytes_in: AtomicU64,
    pub bytes_out: AtomicU64,
    /// P30-A: view messages and their encoded bytes (both seats).
    pub view_messages: AtomicU64,
    pub view_bytes: AtomicU64,
    pub late_ticks: AtomicU64,
    pub ticks: AtomicU64,
    pub rejections: AtomicU64,
    /// P10: worst lobby inbox backlog seen (join bursts must not pile up).
    pub lobby_inbox_max: AtomicU64,
    /// P47: spectators now, listed matches now, sessions online now (the
    /// lobby sets these once a second, with the live snapshot).
    pub spectators: AtomicU64,
    pub live_matches: AtomicU64,
    pub online_players: AtomicU64,
    /// P60: failed settlement attempts, journal files not yet settled
    /// (gauge), idempotent replays, same-key/different-envelope quarantines,
    /// receipts the lobby could not deliver, and journal write failures.
    pub ranked_settlement_failures: AtomicU64,
    pub ranked_pending_files: AtomicU64,
    pub ranked_duplicate_keys: AtomicU64,
    pub ranked_key_mismatches: AtomicU64,
    pub ranked_notify_failures: AtomicU64,
    pub ranked_journal_failures: AtomicU64,
    /// P61: optional summaries stored, omitted (coverage gaps), and failed
    /// inserts kept in the journal for retry.
    pub ranked_stats_saved: AtomicU64,
    pub ranked_stats_gaps: AtomicU64,
    pub ranked_stats_failures: AtomicU64,
    /// P66: ranked results handed to the finalizer and not yet settled,
    /// re-sequenced results, finished results still waiting for a
    /// reconnecting player, and admission answers by kind.
    pub ranked_inflight: AtomicU64,
    pub history_pending: AtomicU64,
    pub history_saved: AtomicU64,
    pub history_gaps: AtomicU64,
    pub history_failures: AtomicU64,
    pub history_conflicts: AtomicU64,

    pub ranked_resequenced: AtomicU64,
    pub result_receipts_pending: AtomicU64,
    pub admit_ok: AtomicU64,
    pub admit_update_required: AtomicU64,
    pub admit_unavailable: AtomicU64,
    /// P66: simulation step time per tick, bucketed (µs upper bounds in
    /// `STEP_BUCKETS_US`, last bucket open), so p99 is cheap to report.
    pub step_hist: [AtomicU64; 8],
    /// P66 §7: aggregate (client build, last update outcome) counters from
    /// admission requests; bounded, never per player or installation.
    update_outcomes: Mutex<std::collections::BTreeMap<(String, String), u64>>,
    rtt: Mutex<VecDeque<(Instant, u16)>>,
    /// P9: lobby queue waits (ms) over the last 60 s.
    queue_wait: Mutex<VecDeque<(Instant, u64)>>,
    /// P60: ranked FIFO waits (ms) over the last 60 s, kept apart from Casual.
    ranked_queue_wait: Mutex<VecDeque<(Instant, u64)>>,
    started: Instant,
}

impl Default for Metrics {
    fn default() -> Self {
        Metrics {
            connections: AtomicU64::new(0),
            matches_running: AtomicU64::new(0),
            matches_finished: AtomicU64::new(0),
            queue_len: AtomicU64::new(0),
            bytes_in: AtomicU64::new(0),
            bytes_out: AtomicU64::new(0),
            view_messages: AtomicU64::new(0),
            view_bytes: AtomicU64::new(0),
            late_ticks: AtomicU64::new(0),
            ticks: AtomicU64::new(0),
            rejections: AtomicU64::new(0),
            lobby_inbox_max: AtomicU64::new(0),
            spectators: AtomicU64::new(0),
            live_matches: AtomicU64::new(0),
            online_players: AtomicU64::new(0),
            ranked_settlement_failures: AtomicU64::new(0),
            ranked_pending_files: AtomicU64::new(0),
            ranked_duplicate_keys: AtomicU64::new(0),
            ranked_key_mismatches: AtomicU64::new(0),
            ranked_notify_failures: AtomicU64::new(0),
            ranked_journal_failures: AtomicU64::new(0),
            ranked_stats_saved: AtomicU64::new(0),
            ranked_stats_gaps: AtomicU64::new(0),
            ranked_stats_failures: AtomicU64::new(0),
            ranked_inflight: AtomicU64::new(0),
            history_pending: AtomicU64::new(0),
            history_saved: AtomicU64::new(0),
            history_gaps: AtomicU64::new(0),
            history_failures: AtomicU64::new(0),
            history_conflicts: AtomicU64::new(0),

            ranked_resequenced: AtomicU64::new(0),
            result_receipts_pending: AtomicU64::new(0),
            admit_ok: AtomicU64::new(0),
            admit_update_required: AtomicU64::new(0),
            admit_unavailable: AtomicU64::new(0),
            step_hist: Default::default(),
            update_outcomes: Mutex::new(std::collections::BTreeMap::new()),
            rtt: Mutex::new(VecDeque::new()),
            queue_wait: Mutex::new(VecDeque::new()),
            ranked_queue_wait: Mutex::new(VecDeque::new()),
            started: Instant::now(),
        }
    }
}

impl Metrics {
    pub fn record_rtt(&self, ms: u16) {
        let now = Instant::now();
        if let Ok(mut q) = self.rtt.lock() {
            q.push_back((now, ms));
            while q.front().is_some_and(|(t, _)| now - *t > RTT_WINDOW) {
                q.pop_front();
            }
        }
    }

    /// P9: time from `QuickMatch` to `MatchStart` for one session.
    pub fn record_queue_wait(&self, ms: u64) {
        Metrics::push_wait(&self.queue_wait, ms);
    }

    /// P60: a ranked pairing's wait, from `RankedMatch` to pairing.
    pub fn record_ranked_queue_wait(&self, ms: u64) {
        Metrics::push_wait(&self.ranked_queue_wait, ms);
    }

    fn push_wait(window: &Mutex<VecDeque<(Instant, u64)>>, ms: u64) {
        let now = Instant::now();
        if let Ok(mut q) = window.lock() {
            q.push_back((now, ms));
            while q.front().is_some_and(|(t, _)| now - *t > RTT_WINDOW) {
                q.pop_front();
            }
        }
    }

    pub fn add(counter: &AtomicU64, n: u64) {
        counter.fetch_add(n, Relaxed);
    }

    /// P66: one simulation step's duration.
    pub fn record_step(&self, micros: u64) {
        let i = STEP_BUCKETS_US
            .iter()
            .position(|limit| micros <= *limit)
            .unwrap_or(STEP_BUCKETS_US.len());
        self.step_hist[i].fetch_add(1, Relaxed);
    }

    /// Upper bound (µs) of the bucket holding the `q` quantile step.
    pub fn step_quantile_us(&self, q: f64) -> u64 {
        let counts: Vec<u64> = self.step_hist.iter().map(|c| c.load(Relaxed)).collect();
        let total: u64 = counts.iter().sum();
        if total == 0 {
            return 0;
        }
        let want = (total as f64 * q).ceil() as u64;
        let mut seen = 0;
        for (i, n) in counts.iter().enumerate() {
            seen += n;
            if seen >= want {
                return STEP_BUCKETS_US.get(i).copied().unwrap_or(u64::MAX);
            }
        }
        u64::MAX
    }

    /// P66 §7: count an admission's reported build and update outcome.
    /// Both labels are validated short slugs and at most
    /// `MAX_OUTCOME_LABELS` pairs are kept, so cardinality stays bounded.
    pub fn record_update_outcome(&self, build: &str, outcome: &str) {
        let slug = |s: &str| {
            !s.is_empty()
                && s.len() <= 64
                && s.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"._-:".contains(&b))
        };
        if !slug(build) || !slug(outcome) {
            return;
        }
        let Ok(mut map) = self.update_outcomes.lock() else {
            return;
        };
        let key = (build.to_string(), outcome.to_string());
        if map.len() >= MAX_OUTCOME_LABELS && !map.contains_key(&key) {
            return;
        }
        *map.entry(key).or_insert(0) += 1;
    }

    fn update_outcomes_json(&self) -> serde_json::Value {
        let map = self
            .update_outcomes
            .lock()
            .map(|m| m.clone())
            .unwrap_or_default();
        serde_json::Value::Array(
            map.into_iter()
                .map(|((build, outcome), n)| serde_json::json!({"build": build, "outcome": outcome, "n": n}))
                .collect(),
        )
    }

    pub fn snapshot(&self) -> serde_json::Value {
        let mut rtts: Vec<u16> = self
            .rtt
            .lock()
            .map(|q| q.iter().map(|(_, ms)| *ms).collect())
            .unwrap_or_default();
        rtts.sort_unstable();
        let mut waits: Vec<u64> = self
            .queue_wait
            .lock()
            .map(|q| q.iter().map(|(_, ms)| *ms).collect())
            .unwrap_or_default();
        waits.sort_unstable();
        let mut ranked_waits: Vec<u64> = self
            .ranked_queue_wait
            .lock()
            .map(|q| q.iter().map(|(_, ms)| *ms).collect())
            .unwrap_or_default();
        ranked_waits.sort_unstable();
        let get = |c: &AtomicU64| c.load(Relaxed);
        let mut out = serde_json::json!({
            "uptime_s": self.started.elapsed().as_secs(),
            "connections": get(&self.connections),
            "matches_running": get(&self.matches_running),
            "matches_finished": get(&self.matches_finished),
            "queue_len": get(&self.queue_len),
            "bytes_in": get(&self.bytes_in),
            "bytes_out": get(&self.bytes_out),
            "view_messages": get(&self.view_messages),
            "view_bytes": get(&self.view_bytes),
            "ticks": get(&self.ticks),
            "late_ticks": get(&self.late_ticks),
            "lobby_inbox_max": get(&self.lobby_inbox_max),
            "spectators": get(&self.spectators),
            "live_matches": get(&self.live_matches),
            "online_players": get(&self.online_players),
            "rejections": get(&self.rejections),
            "ranked_settlement_failures": get(&self.ranked_settlement_failures),
            "ranked_pending_files": get(&self.ranked_pending_files),
            "ranked_duplicate_keys": get(&self.ranked_duplicate_keys),
            "ranked_key_mismatches": get(&self.ranked_key_mismatches),
            "ranked_notify_failures": get(&self.ranked_notify_failures),
            "ranked_journal_failures": get(&self.ranked_journal_failures),
            "ranked_stats_saved": get(&self.ranked_stats_saved),
            "ranked_stats_gaps": get(&self.ranked_stats_gaps),
            "ranked_stats_failures": get(&self.ranked_stats_failures),
            "rtt_p50_ms": percentile(&rtts, 0.50),
            "rtt_p95_ms": percentile(&rtts, 0.95),
            "rtt_max_ms": rtts.last().copied().unwrap_or(0),
            "queue_wait_p50_ms": percentile(&waits, 0.50),
            "queue_wait_p95_ms": percentile(&waits, 0.95),
            "ranked_queue_wait_n": ranked_waits.len(),
            "ranked_queue_wait_p50_ms": percentile(&ranked_waits, 0.50),
            "ranked_queue_wait_p95_ms": percentile(&ranked_waits, 0.95),
        });
        // P66 release fields (a second literal keeps `json!` within its
        // macro recursion limit).
        let release = serde_json::json!({
            "ranked_inflight": get(&self.ranked_inflight),
            "history_pending": get(&self.history_pending),
            "history_saved": get(&self.history_saved),
            "history_gaps": get(&self.history_gaps),
            "history_failures": get(&self.history_failures),
            "history_conflicts": get(&self.history_conflicts),

            "ranked_resequenced": get(&self.ranked_resequenced),
            "result_receipts_pending": get(&self.result_receipts_pending),
            "admit_ok": get(&self.admit_ok),
            "admit_update_required": get(&self.admit_update_required),
            "admit_unavailable": get(&self.admit_unavailable),
            "step_us_p50": self.step_quantile_us(0.50),
            "step_us_p99": self.step_quantile_us(0.99),
            "update_outcomes": self.update_outcomes_json(),
        });
        if let (Some(out), serde_json::Value::Object(extra)) = (out.as_object_mut(), release) {
            out.extend(extra);
        }
        out
    }
}

/// P66: step-time bucket upper bounds in microseconds.
pub const STEP_BUCKETS_US: [u64; 7] = [100, 250, 500, 1_000, 2_000, 5_000, 10_000];
/// P66: distinct (build, outcome) pairs kept in memory.
pub const MAX_OUTCOME_LABELS: usize = 64;

/// Nearest-rank percentile of a sorted slice (0 when empty).
pub fn percentile<T: Copy + Default>(sorted: &[T], q: f64) -> T {
    if sorted.is_empty() {
        return T::default();
    }
    let i = ((sorted.len() - 1) as f64 * q).round() as usize;
    sorted[i.min(sorted.len() - 1)]
}
