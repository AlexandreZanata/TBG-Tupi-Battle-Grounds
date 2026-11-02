//! P66 release runtime (D-92): this process's slot identity and contract,
//! the admission gate the lobby reads, and the background task that
//! registers the slot, reports its load, follows the shared directory and
//! hands ranked settlement over only when the previous holder is idle.
//! Without `TBG_SLOT` the server keeps its single-process behavior.

use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::watch;

use crate::metrics::Metrics;
use crate::store::release::{self, SlotIdentity, SlotLoad, SlotRow};
use crate::store::Store;

pub mod api;

/// Directory poll and heartbeat period.
const TICK: Duration = Duration::from_secs(1);

/// What new work this process may take; read lock-free on hot paths.
pub struct Gate {
    /// SIGTERM: sticky, never reopened.
    pub shutdown: AtomicBool,
    /// The directory lets this slot start new matches.
    pub open: AtomicBool,
    /// This slot settles ranked results (single writer, P66 §6.2).
    pub ranked: AtomicBool,
    /// Store and ranked recovery finished; the slot can be made ready.
    pub serving: AtomicBool,
}

impl Gate {
    fn new(slot_mode: bool) -> Self {
        Gate {
            shutdown: AtomicBool::new(false),
            // A slot waits for the directory; a single server is open.
            open: AtomicBool::new(!slot_mode),
            ranked: AtomicBool::new(!slot_mode),
            serving: AtomicBool::new(false),
        }
    }

    /// May the lobby start a new match now?
    pub fn admits(&self) -> bool {
        !self.shutdown.load(Relaxed) && self.open.load(Relaxed)
    }

    pub fn ranked_open(&self) -> bool {
        self.admits() && self.ranked.load(Relaxed)
    }
}

/// Process-wide release state shared by the API, lobby and shutdown.
pub struct Runtime {
    pub identity: Option<SlotIdentity>,
    pub contract: tbg_core::contract::MatchContract,
    pub gate: Gate,
    pub directory: watch::Receiver<Arc<Vec<SlotRow>>>,
    directory_tx: watch::Sender<Arc<Vec<SlotRow>>>,
}

impl Runtime {
    pub fn new(config: &crate::Config) -> Arc<Self> {
        let contract = tbg_core::contract::MatchContract::current();
        let identity = config.slot.as_ref().map(|slot| SlotIdentity {
            slot: slot.clone(),
            generation: config.generation.clone(),
            server_build: config.server_build.clone(),
            contract: contract.id(),
            endpoint: config.ws_endpoint.clone(),
        });
        let (directory_tx, directory) = watch::channel(Arc::default());
        Arc::new(Runtime {
            gate: Gate::new(identity.is_some()),
            identity,
            contract,
            directory,
            directory_tx,
        })
    }

    pub fn slot(&self) -> Option<&str> {
        self.identity.as_ref().map(|i| i.slot.as_str())
    }

    /// The load this process reports and the retirement gate checks.
    pub fn load(metrics: &Metrics, store: Option<&Store>) -> SlotLoad {
        let ranked = metrics.ranked_inflight.load(Relaxed) as i64;
        let receipts = metrics.result_receipts_pending.load(Relaxed) as i64;
        let jobs = store.map_or(0, |s| s.jobs_pending()) as i64;
        SlotLoad {
            matches_running: metrics.matches_running.load(Relaxed) as i64,
            results_pending: ranked
                + receipts
                + jobs
                + metrics.history_pending.load(Relaxed) as i64,
            ranked_inflight: ranked,
        }
    }
}

/// Register the slot and follow the directory until the process exits.
pub async fn run(runtime: Arc<Runtime>, store: Arc<Store>, metrics: Arc<Metrics>) {
    let Some(id) = runtime.identity.clone() else {
        return;
    };
    let registered = {
        let id = id.clone();
        store
            .release(move |conn| release::register(conn, &id, release::now_unix()))
            .await
    };
    match registered {
        Some(Ok(row)) => {
            tracing::info!(slot = %id.slot, state = %row.state, "release slot registered")
        }
        other => {
            // Fail closed: an unregistered slot never admits new matches.
            tracing::error!(slot = %id.slot, "release slot registration failed: {other:?}");
        }
    }
    let mut ticks = tokio::time::interval(TICK);
    let mut n: u64 = 0;
    loop {
        ticks.tick().await;
        n += 1;
        tick(&runtime, &store, &metrics, &id, n.is_multiple_of(5)).await;
    }
}

async fn tick(
    runtime: &Runtime,
    store: &Arc<Store>,
    metrics: &Metrics,
    id: &SlotIdentity,
    handoff: bool,
) {
    let load = Runtime::load(metrics, Some(store));
    let serving = runtime.gate.serving.load(Relaxed);
    let job_id = id.clone();
    let result = store
        .release(move |conn| -> rusqlite::Result<(bool, Vec<SlotRow>)> {
            let now = release::now_unix();
            let mine = release::heartbeat(conn, &job_id.slot, &job_id.generation, load, now)?;
            if mine && serving {
                release::mark_ready(conn, &job_id.slot, &job_id.generation)?;
            }
            if mine && handoff {
                release::take_ranked_authority(conn, &job_id.slot, now)?;
            }
            Ok((mine, release::read_directory(conn)?))
        })
        .await;
    let Some(Ok((mine, rows))) = result else {
        // Database unreachable: keep the last decision for running work but
        // refuse new admissions until the directory answers again.
        runtime.gate.open.store(false, Relaxed);
        return;
    };
    let own = rows
        .iter()
        .find(|r| r.slot == id.slot && r.generation == id.generation);
    let open = mine && own.is_some_and(SlotRow::admits);
    runtime.gate.open.store(open, Relaxed);
    runtime
        .gate
        .ranked
        .store(mine && own.is_some_and(|r| r.ranked_authority), Relaxed);
    runtime.directory_tx.send_replace(Arc::new(rows));
}

/// The server's own last step after a drained shutdown.
pub async fn retire(runtime: &Runtime, store: &Store) {
    let Some(id) = runtime.identity.clone() else {
        return;
    };
    // A last heartbeat with the drained (zero) load, then the retirement
    // gate itself checks the stored counters.
    let done = store
        .release(move |conn| {
            let now = release::now_unix();
            release::heartbeat(conn, &id.slot, &id.generation, SlotLoad::default(), now)?;
            release::retire_self(conn, &id.slot, &id.generation)
        })
        .await;
    tracing::info!("release slot retired: {done:?}");
}
