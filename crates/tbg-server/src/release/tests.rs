//! Regression: heartbeat retirement counters describe work still pending when the writer serves it.

use super::*;
use std::sync::atomic::{AtomicU64, Ordering};

#[tokio::test]
async fn heartbeat_does_not_retain_a_job_that_finished_before_its_snapshot() {
    let row = snapshot(false).await;
    assert_eq!(row.matches_running, 0);
    assert_eq!(
        row.results_pending, 0,
        "D-104: a completed prior job and the observation itself cannot block retirement"
    );
}

#[tokio::test]
async fn heartbeat_still_counts_other_jobs_queued_behind_it() {
    let row = snapshot(true).await;
    assert_eq!(
        row.results_pending, 1,
        "D-104: queued player work must continue protecting retirement"
    );
}

async fn snapshot(later: bool) -> SlotRow {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let path = std::env::temp_dir().join(format!(
        "tbg-heartbeat-snapshot-{}-{}.sqlite3",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let config = crate::Config {
        slot: Some("a".into()),
        generation: "snapshot-test".into(),
        ws_endpoint: "/ws/slot-a".into(),
        ..crate::Config::default()
    };
    let runtime = Runtime::new(&config);
    let store = Arc::new(Store::open(&path).unwrap());
    let metrics = Arc::new(Metrics::default());
    let id = runtime.identity.clone().unwrap();
    let registered = id.clone();
    store
        .release(move |c| release::register(c, &registered, release::now_unix()))
        .await
        .unwrap()
        .unwrap();
    let (entered_tx, entered_rx) = std::sync::mpsc::channel();
    let (continue_tx, continue_rx) = std::sync::mpsc::channel();
    let earlier_store = store.clone();
    let earlier = tokio::spawn(async move {
        earlier_store
            .release(move |_| {
                entered_tx.send(()).unwrap();
                continue_rx.recv().unwrap();
            })
            .await
    });
    tokio::task::spawn_blocking(move || entered_rx.recv().unwrap())
        .await
        .unwrap();
    let (rt, db, counts) = (runtime.clone(), store.clone(), metrics.clone());
    let heartbeat = tokio::spawn(async move { tick(&rt, &db, &counts, &id, false).await });
    tokio::time::timeout(Duration::from_secs(2), async {
        while store.jobs_pending() < 2 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    let later_job = if later {
        let later_store = store.clone();
        let task = tokio::spawn(async move { later_store.release(|_| ()).await });
        tokio::time::timeout(Duration::from_secs(2), async {
            while store.jobs_pending() < 3 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        Some(task)
    } else {
        None
    };
    continue_tx.send(()).unwrap();
    earlier.await.unwrap().unwrap();
    heartbeat.await.unwrap();
    let rows = store
        .release(|c| release::read_directory(c))
        .await
        .unwrap()
        .unwrap();
    if let Some(task) = later_job {
        task.await.unwrap().unwrap();
    }
    rows.into_iter().next().unwrap()
}
