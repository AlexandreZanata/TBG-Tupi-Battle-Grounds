//! The only module that touches SQL (ARCHITECTURE §8): a single writer
//! thread owns the SQLite connection; everyone else talks to it through a
//! bounded channel. Match records stay fire-and-forget; account commands
//! carry a oneshot for their reply, so the lobby never blocks on SQLite.

use std::path::Path;
use std::sync::mpsc::{sync_channel, SyncSender};
use std::sync::Arc as StdArc;

use serde_json::Value;
use tokio::sync::{oneshot, watch};

use tbg_core::protocol::{AccountError, Profile};
use tbg_core::{ArcanaPreset, StrongholdKind};

use crate::ranked::{JournalEntry, RankedReceipt};

mod accounts;
mod arcanas;
pub mod leaderboard;
mod matches;
mod ranked;
mod rulesets;
mod schema;
mod strongholds;
mod worker;

use worker::{worker, StoreCmd};

pub use leaderboard::{LeaderRow, Snapshot};
pub use ranked::{PlayerStats, RecentMatch, SettleError, POLICY_VERSION};

pub type AccountId = i64;

/// Handle to the single writer thread.
#[derive(Clone)]
pub struct Store {
    tx: SyncSender<StoreCmd>,
    /// P13: published leaderboard; readers clone the `Arc`, never query.
    snapshot_tx: watch::Sender<StdArc<Snapshot>>,
}

impl Store {
    pub fn open(path: &Path) -> rusqlite::Result<Store> {
        let mut conn = rusqlite::Connection::open(path)?;
        schema::migrate(&mut conn)?;
        // P61 §3: this binary's ruleset revision, registered once.
        rulesets::register(
            &conn,
            tbg_core::ruleset_version::RULESET_VERSION,
            &format!("{:016x}", crate::ranked::source_hash()),
            &crate::ranked::catalog::catalog_json(),
        )?;
        let (tx, rx) = sync_channel::<StoreCmd>(1024);
        let (snapshot_tx, _) = watch::channel(StdArc::new(leaderboard::build(&conn)?));
        let snapshots = snapshot_tx.clone();
        std::thread::Builder::new()
            .name("tbg-store".into())
            .spawn(move || worker(conn, rx, snapshots))
            .map_err(|_| rusqlite::Error::InvalidPath(path.to_path_buf()))?;
        Ok(Store { tx, snapshot_tx })
    }

    /// Current leaderboard snapshot (no SQL on this path).
    pub fn snapshot(&self) -> StdArc<Snapshot> {
        self.snapshot_tx.borrow().clone()
    }

    /// Watch for rebuilt snapshots.
    pub fn subscribe(&self) -> watch::Receiver<StdArc<Snapshot>> {
        self.snapshot_tx.subscribe()
    }

    /// Fire and forget (match records are non-critical).
    pub fn insert_match(&self, m: &Value) {
        let _ = self.tx.try_send(StoreCmd::InsertMatch(m.clone()));
    }

    /// D-40: remember an account's preset (fire and forget, BR-AR08).
    pub fn save_arcanas(&self, account: AccountId, preset: ArcanaPreset) {
        let _ = self.tx.try_send(StoreCmd::SaveArcanas { account, preset });
    }

    /// D-46: remember the account's Stronghold kind (fire-and-forget).
    pub fn save_stronghold(&self, account: AccountId, kind: StrongholdKind) {
        let _ = self.tx.try_send(StoreCmd::SaveStronghold { account, kind });
    }

    /// D-46: the account's saved kind, if any and still selectable.
    pub async fn load_stronghold(&self, account: AccountId) -> Option<StrongholdKind> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.tx
            .try_send(StoreCmd::LoadStronghold {
                account,
                reply: reply_tx,
            })
            .ok()?;
        reply_rx.await.ok()?
    }

    /// D-40: the account's saved preset, if any (validated on read).
    pub async fn load_arcanas(&self, account: AccountId) -> Option<ArcanaPreset> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.tx
            .try_send(StoreCmd::LoadArcanas {
                account,
                reply: reply_tx,
            })
            .ok()?;
        reply_rx.await.ok()?
    }

    pub async fn register(
        &self,
        name: String,
        key_hash: [u8; 32],
    ) -> Result<(AccountId, Profile), AccountError> {
        let name_lower = name.to_ascii_lowercase();
        let (reply_tx, reply_rx) = oneshot::channel();
        self.tx
            .try_send(StoreCmd::Register {
                name,
                name_lower,
                key_hash,
                reply: reply_tx,
            })
            .map_err(|_| AccountError::Unavailable)?;
        reply_rx.await.map_err(|_| AccountError::Unavailable)?
    }

    pub async fn login(
        &self,
        name_lower: String,
        key_hash: [u8; 32],
    ) -> Result<(AccountId, Profile), AccountError> {
        self.login_fresh(name_lower, key_hash)
            .await
            .map(|(id, profile, _)| (id, profile))
    }

    /// Login plus the committed ranked high-water mark read in the same
    /// worker turn: any settlement at or below it is already in `Profile`.
    pub async fn login_fresh(
        &self,
        name_lower: String,
        key_hash: [u8; 32],
    ) -> Result<(AccountId, Profile, i64), AccountError> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.tx
            .try_send(StoreCmd::Login {
                name_lower,
                key_hash,
                reply: reply_tx,
            })
            .map_err(|_| AccountError::Unavailable)?;
        reply_rx.await.map_err(|_| AccountError::Unavailable)?
    }

    /// P60: settle one journaled ranked result (or return its committed
    /// receipt again). Only the finalizer calls this, in completion order.
    pub async fn settle_ranked(&self, entry: JournalEntry) -> Result<RankedReceipt, SettleError> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.tx
            .try_send(StoreCmd::Settle {
                entry: Box::new(entry),
                reply: reply_tx,
            })
            .map_err(|_| SettleError::Sql(rusqlite::Error::InvalidQuery))?;
        reply_rx
            .await
            .map_err(|_| SettleError::Sql(rusqlite::Error::InvalidQuery))?
    }

    /// P60: highest committed ranked sequence (finalizer start-up mark).
    pub async fn ranked_high_water(&self) -> rusqlite::Result<i64> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.tx
            .try_send(StoreCmd::HighWater(reply_tx))
            .map_err(|_| rusqlite::Error::InvalidQuery)?;
        reply_rx.await.map_err(|_| rusqlite::Error::InvalidQuery)?
    }

    /// Public profile for `/api/players/{name}` (`None` when unknown).
    /// At most three indexed queries; the rank comes from the snapshot.
    pub async fn player_stats(&self, name_lower: String) -> Option<ranked::PlayerStats> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.tx
            .try_send(StoreCmd::PlayerStats {
                name_lower,
                reply: reply_tx,
            })
            .ok()?;
        reply_rx.await.ok()?
    }

    /// Test/support helper: ranked-row count over the writer thread.
    pub async fn ranked_match_count(&self) -> rusqlite::Result<i64> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.tx
            .try_send(StoreCmd::CountRanked(reply_tx))
            .map_err(|_| rusqlite::Error::InvalidQuery)?;
        reply_rx.await.map_err(|_| rusqlite::Error::InvalidQuery)?
    }

    /// Test/support helper: row count over the writer thread. The count
    /// goes through the same queue, so it observes every insert sent before.
    pub async fn match_count(&self) -> rusqlite::Result<i64> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.tx
            .try_send(StoreCmd::CountMatches(reply_tx))
            .map_err(|_| rusqlite::Error::InvalidQuery)?;
        reply_rx.await.map_err(|_| rusqlite::Error::InvalidQuery)?
    }
}
