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
    pub late_ticks: AtomicU64,
    pub ticks: AtomicU64,
    pub rejections: AtomicU64,
    /// P10: worst lobby inbox backlog seen (join bursts must not pile up).
    pub lobby_inbox_max: AtomicU64,
    rtt: Mutex<VecDeque<(Instant, u16)>>,
    /// P9: lobby queue waits (ms) over the last 60 s.
    queue_wait: Mutex<VecDeque<(Instant, u64)>>,
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
            late_ticks: AtomicU64::new(0),
            ticks: AtomicU64::new(0),
            rejections: AtomicU64::new(0),
            lobby_inbox_max: AtomicU64::new(0),
            rtt: Mutex::new(VecDeque::new()),
            queue_wait: Mutex::new(VecDeque::new()),
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
        let now = Instant::now();
        if let Ok(mut q) = self.queue_wait.lock() {
            q.push_back((now, ms));
            while q.front().is_some_and(|(t, _)| now - *t > RTT_WINDOW) {
                q.pop_front();
            }
        }
    }

    pub fn add(counter: &AtomicU64, n: u64) {
        counter.fetch_add(n, Relaxed);
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
        let get = |c: &AtomicU64| c.load(Relaxed);
        serde_json::json!({
            "uptime_s": self.started.elapsed().as_secs(),
            "connections": get(&self.connections),
            "matches_running": get(&self.matches_running),
            "matches_finished": get(&self.matches_finished),
            "queue_len": get(&self.queue_len),
            "bytes_in": get(&self.bytes_in),
            "bytes_out": get(&self.bytes_out),
            "ticks": get(&self.ticks),
            "late_ticks": get(&self.late_ticks),
            "lobby_inbox_max": get(&self.lobby_inbox_max),
            "rejections": get(&self.rejections),
            "rtt_p50_ms": percentile(&rtts, 0.50),
            "rtt_p95_ms": percentile(&rtts, 0.95),
            "rtt_max_ms": rtts.last().copied().unwrap_or(0),
            "queue_wait_p50_ms": percentile(&waits, 0.50),
            "queue_wait_p95_ms": percentile(&waits, 0.95),
        })
    }
}

/// Nearest-rank percentile of a sorted slice (0 when empty).
pub fn percentile<T: Copy + Default>(sorted: &[T], q: f64) -> T {
    if sorted.is_empty() {
        return T::default();
    }
    let i = ((sorted.len() - 1) as f64 * q).round() as usize;
    sorted[i.min(sorted.len() - 1)]
}
