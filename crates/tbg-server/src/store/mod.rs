//! The only module that touches SQL (ARCHITECTURE §8): a single writer
//! thread owns the SQLite connection; everyone else talks to it through a
//! bounded channel. Match records stay fire-and-forget; account commands
//! carry a oneshot for their reply, so the lobby never blocks on SQLite.

use std::path::Path;
use std::sync::mpsc::{sync_channel, SyncSender};

use serde_json::Value;
use tokio::sync::oneshot;

use tbg_core::protocol::{AccountError, Profile, RatingResults};
use tbg_core::Outcome;

mod accounts;
mod matches;
mod ranked;
mod schema;

pub type AccountId = i64;

type Reply<T> = oneshot::Sender<T>;

enum StoreCmd {
    Register {
        name: String,
        name_lower: String,
        key_hash: [u8; 32],
        reply: Reply<Result<(AccountId, Profile), AccountError>>,
    },
    Login {
        name_lower: String,
        key_hash: [u8; 32],
        reply: Reply<Result<(AccountId, Profile), AccountError>>,
    },
    InsertMatch(Value),
    CountMatches(Reply<rusqlite::Result<i64>>),
    CountRanked(Reply<rusqlite::Result<i64>>),
    RankedResult {
        a: AccountId,
        b: AccountId,
        outcome: Option<Outcome>,
        duration_s: u64,
        factions: [[u32; 4]; 2],
        reply: Reply<Result<RatingResults, AccountError>>,
    },
}

/// Handle to the single writer thread.
#[derive(Clone)]
pub struct Store {
    tx: SyncSender<StoreCmd>,
}

impl Store {
    pub fn open(path: &Path) -> rusqlite::Result<Store> {
        let mut conn = rusqlite::Connection::open(path)?;
        schema::migrate(&mut conn)?;
        let (tx, rx) = sync_channel::<StoreCmd>(1024);
        std::thread::Builder::new()
            .name("tbg-store".into())
            .spawn(move || worker(conn, rx))
            .map_err(|_| rusqlite::Error::InvalidPath(path.to_path_buf()))?;
        Ok(Store { tx })
    }

    /// Fire and forget (match records are non-critical).
    pub fn insert_match(&self, m: &Value) {
        let _ = self.tx.try_send(StoreCmd::InsertMatch(m.clone()));
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

    /// Settle a ranked match in one transaction: both Elo updates plus the
    /// `ranked_matches` row. Returns per-seat `(before, after, profile)`.
    /// Only [`Outcome`]-level failures surface as `Unavailable`.
    pub async fn ranked_result(
        &self,
        a: AccountId,
        b: AccountId,
        outcome: Option<Outcome>,
        duration_s: u64,
        factions: [[u32; 4]; 2],
    ) -> Result<RatingResults, AccountError> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.tx
            .try_send(StoreCmd::RankedResult {
                a,
                b,
                outcome,
                duration_s,
                factions,
                reply: reply_tx,
            })
            .map_err(|_| AccountError::Unavailable)?;
        reply_rx.await.map_err(|_| AccountError::Unavailable)?
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

fn worker(mut conn: rusqlite::Connection, rx: std::sync::mpsc::Receiver<StoreCmd>) {
    for cmd in rx {
        match cmd {
            StoreCmd::Register {
                name,
                name_lower,
                key_hash,
                reply,
            } => {
                let _ = reply.send(accounts::register(&mut conn, &name, &name_lower, &key_hash));
            }
            StoreCmd::Login {
                name_lower,
                key_hash,
                reply,
            } => {
                let _ = reply.send(accounts::login(&conn, &name_lower, &key_hash));
            }
            StoreCmd::InsertMatch(m) => {
                if let Err(e) = matches::insert(&conn, &m) {
                    tracing::error!("match record insert failed: {e}");
                }
            }
            StoreCmd::CountMatches(reply) => {
                let _ = reply.send(matches::count(&conn));
            }
            StoreCmd::CountRanked(reply) => {
                let _ = reply.send(ranked::count(&conn));
            }
            StoreCmd::RankedResult {
                a,
                b,
                outcome,
                duration_s,
                factions,
                reply,
            } => {
                let result = ranked::ranked_result(&mut conn, a, b, outcome, duration_s, factions)
                    .map_err(|_| AccountError::Unavailable);
                let _ = reply.send(result);
            }
        }
    }
}
