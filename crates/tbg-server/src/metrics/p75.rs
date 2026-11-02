//! Fixed-cardinality, process-local social/operator counters with no personal labels.
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
pub static MAIL_QUEUED: AtomicU64 = AtomicU64::new(0);
pub static MAIL_DROPPED: AtomicU64 = AtomicU64::new(0);
pub static REPORTS: AtomicU64 = AtomicU64::new(0);
pub static UPLOADS: AtomicU64 = AtomicU64::new(0);
pub static APPROVALS: AtomicU64 = AtomicU64::new(0);
pub static BANS: AtomicU64 = AtomicU64::new(0);
pub static SIGNINS: AtomicU64 = AtomicU64::new(0);
pub static SIGNIN_FAILURES: AtomicU64 = AtomicU64::new(0);
pub fn add(counter: &AtomicU64) {
    counter.fetch_add(1, Relaxed);
}
pub fn snapshot() -> serde_json::Value {
    serde_json::json!({"mail_queued":MAIL_QUEUED.load(Relaxed),"mail_dropped":MAIL_DROPPED.load(Relaxed),
      "reports_filed":REPORTS.load(Relaxed),"uploads":UPLOADS.load(Relaxed),"approvals":APPROVALS.load(Relaxed),
      "bans":BANS.load(Relaxed),"admin_signins":SIGNINS.load(Relaxed),"admin_signin_failures":SIGNIN_FAILURES.load(Relaxed)})
}
