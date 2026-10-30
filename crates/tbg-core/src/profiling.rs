//! Opt-in operation counts for single-threaded native experiments. These
//! counters are not simulation state, serialized, or enabled online.

use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

pub static ID_SEARCHES: AtomicU64 = AtomicU64::new(0);
pub static STATS: AtomicU64 = AtomicU64::new(0);
pub static TARGET_CANDIDATES: AtomicU64 = AtomicU64::new(0);
pub static FORMATION_CANDIDATES: AtomicU64 = AtomicU64::new(0);

pub fn reset() {
    for counter in [
        &ID_SEARCHES,
        &STATS,
        &TARGET_CANDIDATES,
        &FORMATION_CANDIDATES,
    ] {
        counter.store(0, Relaxed);
    }
}

pub fn snapshot() -> [u64; 4] {
    [
        &ID_SEARCHES,
        &STATS,
        &TARGET_CANDIDATES,
        &FORMATION_CANDIDATES,
    ]
    .map(|counter| counter.load(Relaxed))
}
