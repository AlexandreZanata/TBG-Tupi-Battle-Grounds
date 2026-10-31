//! The one ranked finalizer task (P60 §7): assigns each completed ranked
//! result a strictly increasing completion sequence, freezes its UTC
//! second, journals it, then settles pending results strictly in sequence
//! order. A failing head is retried (1 s, 5 s, then every 30 s) and never
//! skipped; while anything is pending and unsettled, new ranked admission
//! reports `Unavailable`. Matches already running still finish.

use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tokio::sync::mpsc;

use super::{journal, JournalEntry, RankedFinal};
use crate::chan::LobbyCmd;
use crate::metrics::Metrics;
use crate::store::{SettleError, StatsWriteError, Store};

/// Results waiting for a slot in the finalizer inbox; the sender awaits,
/// so a full inbox delays a finished match task, never a running one.
const INBOX: usize = 256;
const RETRY: [Duration; 3] = [
    Duration::from_secs(1),
    Duration::from_secs(5),
    Duration::from_secs(30),
];

/// What match tasks and the lobby hold.
#[derive(Clone)]
pub(crate) struct Handle {
    pub tx: mpsc::Sender<RankedFinal>,
    /// Set while a pending result cannot settle; ranked admission refuses.
    pub unavailable: Arc<AtomicBool>,
}

struct Pending {
    entry: JournalEntry,
    journaled: bool,
}

/// Recover the journal, then run the finalizer. Awaited before the lobby
/// serves commands, so ranked admission is never offered on a stale view.
pub(crate) async fn start(
    store: Arc<Store>,
    db: PathBuf,
    lobby: mpsc::Sender<LobbyCmd>,
    metrics: Arc<Metrics>,
) -> Handle {
    let (tx, rx) = mpsc::channel(INBOX);
    let unavailable = Arc::new(AtomicBool::new(true));
    let mut worker = Finalizer {
        store,
        dir: journal::dir_for(&db),
        lobby,
        metrics,
        unavailable: unavailable.clone(),
        pending: VecDeque::new(),
        high_water: 0,
        last_unix: 0,
        healthy: false,
    };
    // Unavailable until recovery succeeded and every recovered file settled.
    let healthy = worker.recover().await;
    worker.healthy = healthy;
    worker
        .unavailable
        .store(!healthy || !worker.pending.is_empty(), Relaxed);
    tokio::spawn(worker.run(rx));
    Handle { tx, unavailable }
}

struct Finalizer {
    store: Arc<Store>,
    dir: PathBuf,
    lobby: mpsc::Sender<LobbyCmd>,
    metrics: Arc<Metrics>,
    unavailable: Arc<AtomicBool>,
    pending: VecDeque<Pending>,
    high_water: i64,
    last_unix: i64,
    /// Start-up recovery succeeded; otherwise ranked stays unavailable
    /// until the operator restarts after fixing storage.
    healthy: bool,
}

impl Finalizer {
    async fn run(mut self, mut rx: mpsc::Receiver<RankedFinal>) {
        let mut attempt = 0usize;
        loop {
            if self.pending.is_empty() {
                attempt = 0;
                if self.healthy {
                    self.unavailable.store(false, Relaxed);
                }
                match rx.recv().await {
                    Some(result) => self.accept(result).await,
                    None => return,
                }
                continue;
            }
            if self.settle_head().await {
                attempt = 0;
                continue;
            }
            self.unavailable.store(true, Relaxed);
            let wait = RETRY[attempt.min(RETRY.len() - 1)];
            attempt += 1;
            // Keep journaling new results while the head waits to retry.
            tokio::select! {
                next = rx.recv() => match next {
                    Some(result) => self.accept(result).await,
                    None => return,
                },
                _ = tokio::time::sleep(wait) => {}
            }
        }
    }

    /// Start-up: pending files in sequence order, then the high-water mark
    /// from both committed rows and outstanding files. `false` when either
    /// source failed: ranked stays unavailable (a guessed mark could reuse
    /// a committed sequence).
    async fn recover(&mut self) -> bool {
        let dir = self.dir.clone();
        let loaded = tokio::task::spawn_blocking(move || journal::load_all(&dir))
            .await
            .unwrap_or_else(|e| Err(std::io::Error::other(e)));
        let committed = self.store.ranked_high_water().await;
        let healthy = loaded.is_ok() && committed.is_ok();
        match (&loaded, &committed) {
            (Ok(_), Ok(_)) => {}
            _ => tracing::error!(
                "ranked recovery incomplete (journal {:?}, store {:?}); ranked stays unavailable \
                 until it succeeds",
                loaded.as_ref().err(),
                committed.as_ref().err()
            ),
        }
        let loaded = loaded.unwrap_or_default();
        self.high_water = committed.unwrap_or(0);
        for entry in loaded {
            self.high_water = self.high_water.max(entry.completion_seq);
            self.last_unix = self.last_unix.max(entry.ended_unix);
            self.pending.push_back(Pending {
                entry,
                journaled: true,
            });
        }
        self.metrics
            .ranked_pending_files
            .store(self.pending.len() as u64, Relaxed);
        healthy
    }

    /// Freeze sequence and completion second, then journal before any
    /// settlement attempt. A journal failure keeps the entry in memory.
    async fn accept(&mut self, result: RankedFinal) {
        self.high_water += 1;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_secs() as i64);
        self.last_unix = self.last_unix.max(now);
        let entry = JournalEntry {
            completion_seq: self.high_water,
            ended_unix: self.last_unix,
            result,
        };
        let (dir, copy) = (self.dir.clone(), entry.clone());
        let journaled = match tokio::task::spawn_blocking(move || journal::write(&dir, &copy)).await
        {
            Ok(Ok(())) => {
                self.metrics.ranked_pending_files.fetch_add(1, Relaxed);
                true
            }
            outcome => {
                self.metrics.ranked_journal_failures.fetch_add(1, Relaxed);
                tracing::error!(
                    "ranked journal write failed for sequence {}: {outcome:?}; kept in memory only",
                    entry.completion_seq
                );
                false
            }
        };
        self.pending.push_back(Pending { entry, journaled });
    }

    /// Settle the head. `true` when it left the queue (committed, replayed
    /// or quarantined), `false` when it must be retried.
    async fn settle_head(&mut self) -> bool {
        let Some(head) = self.pending.front() else {
            return true;
        };
        let (entry, journaled) = (head.entry.clone(), head.journaled);
        match self.store.settle_ranked(entry.clone()).await {
            Ok(receipt) => {
                if receipt.sequence != entry.completion_seq {
                    // The key was committed earlier (crash after commit).
                    self.metrics.ranked_duplicate_keys.fetch_add(1, Relaxed);
                }
                // P61: the optional summary follows the commit; a transient
                // failure keeps the journal file (replay is idempotent) but
                // never holds back the receipt or later settlements.
                let keep = match entry.result.stats.clone() {
                    None => false,
                    Some(summary) => self.save_stats(receipt.sequence, summary).await,
                };
                if keep {
                    self.pending.pop_front();
                } else {
                    self.forget(&entry, journaled, false).await;
                }
                if self
                    .lobby
                    .send(LobbyCmd::RankedDone {
                        receipt: Box::new(receipt),
                    })
                    .await
                    .is_err()
                {
                    self.metrics.ranked_notify_failures.fetch_add(1, Relaxed);
                }
                true
            }
            Err(SettleError::ConflictingDuplicate) | Err(SettleError::SameAccount) => {
                self.metrics.ranked_key_mismatches.fetch_add(1, Relaxed);
                tracing::error!(
                    "ranked result {} quarantined: contradictory identity or envelope",
                    entry.completion_seq
                );
                self.forget(&entry, journaled, true).await;
                true
            }
            Err(SettleError::Sql(_)) => {
                self.metrics
                    .ranked_settlement_failures
                    .fetch_add(1, Relaxed);
                false
            }
        }
    }

    /// `true` when the summary must stay journaled for a later retry.
    async fn save_stats(&self, match_id: i64, summary: serde_json::Value) -> bool {
        match self.store.save_ranked_stats(match_id, summary).await {
            Ok(()) => {
                self.metrics.ranked_stats_saved.fetch_add(1, Relaxed);
                false
            }
            Err(StatsWriteError::Unavailable) => {
                self.metrics.ranked_stats_failures.fetch_add(1, Relaxed);
                tracing::warn!("ranked summary {match_id} not stored; kept in the journal");
                true
            }
            Err(e) => {
                // Malformed or contradictory evidence: a gap, never retried.
                self.metrics.ranked_stats_gaps.fetch_add(1, Relaxed);
                tracing::warn!("ranked summary {match_id} rejected: {e:?}");
                false
            }
        }
    }

    async fn forget(&mut self, entry: &JournalEntry, journaled: bool, quarantine: bool) {
        self.pending.pop_front();
        if !journaled {
            return;
        }
        let (dir, copy) = (self.dir.clone(), entry.clone());
        let done = tokio::task::spawn_blocking(move || {
            if quarantine {
                journal::quarantine(&dir, &copy)
            } else {
                journal::remove(&dir, &copy)
            }
        })
        .await;
        if !matches!(done, Ok(Ok(()))) {
            // Harmless: a replay of a committed key returns its receipt.
            tracing::warn!(
                "ranked journal cleanup failed for sequence {}",
                entry.completion_seq
            );
        }
        // Only this task writes the gauge, so load-then-store cannot race.
        let gauge = &self.metrics.ranked_pending_files;
        gauge.store(gauge.load(Relaxed).saturating_sub(1), Relaxed);
    }
}
