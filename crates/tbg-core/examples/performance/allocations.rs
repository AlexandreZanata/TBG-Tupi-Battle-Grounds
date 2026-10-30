//! Benchmark-only counting allocator. Unsafe calls forward the unchanged
//! pointer/layout contract to System; no custom allocation or simulation
//! unsafe code. Build separately with --features bench-alloc.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed};

use super::measure::Counts;

struct Counting;
#[global_allocator]
static GLOBAL: Counting = Counting;
static ACTIVE: AtomicBool = AtomicBool::new(false);
static CALLS: AtomicU64 = AtomicU64::new(0);
static BYTES: AtomicU64 = AtomicU64::new(0);
static LIVE: AtomicU64 = AtomicU64::new(0);
static PEAK: AtomicU64 = AtomicU64::new(0);

fn allocated(bytes: usize) {
    let live = LIVE.fetch_add(bytes as u64, Relaxed) + bytes as u64;
    if ACTIVE.load(Relaxed) {
        CALLS.fetch_add(1, Relaxed);
        BYTES.fetch_add(bytes as u64, Relaxed);
        PEAK.fetch_max(live, Relaxed);
    }
}

// SAFETY: every operation delegates to System with precisely its caller's
// pointer, layout, and size. Counters allocate nothing and cannot panic.
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let p = unsafe { System.alloc(layout) };
        if !p.is_null() {
            allocated(layout.size());
        }
        p
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let p = unsafe { System.alloc_zeroed(layout) };
        if !p.is_null() {
            allocated(layout.size());
        }
        p
    }

    unsafe fn dealloc(&self, p: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size() as u64, Relaxed);
        unsafe { System.dealloc(p, layout) };
    }

    unsafe fn realloc(&self, p: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let next = unsafe { System.realloc(p, layout, size) };
        if !next.is_null() {
            LIVE.fetch_sub(layout.size() as u64, Relaxed);
            allocated(size);
        }
        next
    }
}

pub fn start() {
    CALLS.store(0, Relaxed);
    BYTES.store(0, Relaxed);
    PEAK.store(LIVE.load(Relaxed), Relaxed);
    ACTIVE.store(true, Relaxed);
}

pub fn stop() -> Counts {
    ACTIVE.store(false, Relaxed);
    Counts {
        calls: CALLS.load(Relaxed),
        bytes: BYTES.load(Relaxed),
        peak: PEAK.load(Relaxed),
        operations: tbg_core::profiling::snapshot(),
    }
}
