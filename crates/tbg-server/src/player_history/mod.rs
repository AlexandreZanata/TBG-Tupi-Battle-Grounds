//! Bounded post-combat history finalization, recovery and participant-only acknowledgments.

use crate::{
    chan::LobbyCmd,
    metrics::Metrics,
    store::{
        player_history::{self, Envelope, Error},
        Store,
    },
};
use std::{
    path::PathBuf,
    sync::{atomic::Ordering::Relaxed, Arc},
    time::Duration,
};
use tbg_core::protocol::HistorySaveState;
use tokio::sync::mpsc;
pub(crate) mod api;
pub mod journal;

#[derive(Clone)]
pub(crate) struct Handle {
    tx: mpsc::Sender<Envelope>,
    metrics: Arc<Metrics>,
}
impl Handle {
    pub fn submit(&self, entry: Envelope) -> bool {
        self.metrics.history_pending.fetch_add(1, Relaxed);
        if self.tx.try_send(entry).is_err() {
            self.metrics.history_pending.fetch_sub(1, Relaxed);
            self.metrics.history_gaps.fetch_add(1, Relaxed);
            tracing::error!("history finalizer queue full or stopped; terminal history delayed with no durable journal");
            return false;
        }
        true
    }
}

pub(crate) async fn start(
    store: Arc<Store>,
    dir: PathBuf,
    lobby: mpsc::Sender<LobbyCmd>,
    metrics: Arc<Metrics>,
) -> Handle {
    let (tx, mut rx) = mpsc::channel(64);
    let handle = Handle {
        tx,
        metrics: metrics.clone(),
    };
    // Startup recovery precedes admission. Corruption stays on disk and blocks retirement.
    let path = dir.clone();
    let recovered = tokio::task::spawn_blocking(move || journal::load(&path)).await;
    let recovered = match recovered {
        Ok(Ok(v)) => v,
        _ => {
            metrics.history_pending.fetch_add(1, Relaxed);
            tracing::error!(
                "history recovery failed; journal retained and slot retirement blocked"
            );
            Vec::new()
        }
    };
    for entry in recovered {
        metrics.history_pending.fetch_add(1, Relaxed);
        finalize(&store, &dir, &lobby, &metrics, entry, true).await;
    }
    // Each batch is bounded; no foreground SQL read triggers a backfill.
    loop {
        match store.release(player_history::backfill::batch).await {
            Some(Ok(true)) => tokio::task::yield_now().await,
            Some(Ok(false)) => break,
            _ => {
                tracing::error!("history backfill deferred after store failure");
                break;
            }
        }
    }
    tokio::spawn(async move {
        // Reuse the directory's one-second cadence to catch pre-P73 results
        // completed by draining writers after the migration's initial batch.
        let mut backfill = tokio::time::interval(Duration::from_secs(1));
        backfill.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            tokio::select! {
                entry=rx.recv()=>match entry {Some(entry)=>finalize(&store,&dir,&lobby,&metrics,entry,false).await,None=>return},
                _=backfill.tick()=>{
                    if !matches!(store.release(player_history::backfill::batch).await,Some(Ok(_))) {metrics.history_failures.fetch_add(1,Relaxed);}
                }
            }
        }
    });
    handle
}

async fn finalize(
    store: &Store,
    dir: &std::path::Path,
    lobby: &mpsc::Sender<LobbyCmd>,
    metrics: &Metrics,
    entry: Envelope,
    recovered: bool,
) {
    if !recovered {
        let (path, copy) = (dir.to_path_buf(), entry.clone());
        if !matches!(
            tokio::task::spawn_blocking(move || journal::write(&path, &copy)).await,
            Ok(Ok(()))
        ) {
            metrics.history_gaps.fetch_add(1, Relaxed);
            metrics.history_pending.fetch_sub(1, Relaxed);
            tracing::error!("history journal unavailable; result preserved but durable history has a coverage gap");
            notify(lobby, &entry, HistorySaveState::Delayed).await;
            return;
        }
    }
    for wait in [0, 100, 400, 1000, 2000] {
        if wait > 0 {
            tokio::time::sleep(Duration::from_millis(wait)).await;
        }
        let copy = entry.clone();
        match store
            .release(move |conn| player_history::insert(conn, &copy))
            .await
        {
            Some(Ok(())) => {
                metrics.history_saved.fetch_add(1, Relaxed);
                notify(lobby, &entry, HistorySaveState::Stored).await;
                let (path, copy) = (dir.to_path_buf(), entry.clone());
                if matches!(
                    tokio::task::spawn_blocking(move || journal::remove(&path, &copy)).await,
                    Ok(Ok(()))
                ) {
                    metrics.history_pending.fetch_sub(1, Relaxed);
                } else {
                    tracing::error!("committed history journal retained for idempotent recovery");
                }
                return;
            }
            Some(Err(Error::Conflict | Error::Invalid)) => {
                metrics.history_conflicts.fetch_add(1, Relaxed);
                let (path, copy) = (dir.to_path_buf(), entry.clone());
                let _ =
                    tokio::task::spawn_blocking(move || journal::quarantine(&path, &copy)).await;
                tracing::error!(
                    "history envelope quarantined; operator review required before slot retirement"
                );
                break;
            }
            _ => {
                metrics.history_failures.fetch_add(1, Relaxed);
            }
        }
    }
    notify(lobby, &entry, HistorySaveState::Delayed).await;
    tracing::error!(
        "history write delayed; bounded retries exhausted, durable journal retained for restart"
    );
}

async fn notify(lobby: &mpsc::Sender<LobbyCmd>, e: &Envelope, state: HistorySaveState) {
    if let Some(result_id) = e.result_id {
        for side in &e.sides {
            if let Some(account) = side.account {
                let _ = lobby
                    .send(LobbyCmd::HistoryDone {
                        account,
                        result_id,
                        entry_id: side.entry_id,
                        state,
                    })
                    .await;
            }
        }
    }
}
