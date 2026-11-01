//! The single writer thread: owns the SQLite connection, serves every
//! command in order, and rebuilds the leaderboard snapshot at most every
//! 10 s once ranked results landed (P13).

use std::sync::Arc as StdArc;
use std::time::Instant;

use serde_json::Value;
use tokio::sync::{oneshot, watch};

use tbg_core::protocol::{AccountError, Profile};
use tbg_core::{ArcanaPreset, StrongholdKind};

use crate::ranked::{JournalEntry, RankedReceipt};

use super::leaderboard::{self, Snapshot};
use super::{accounts, arcanas, matches, ranked, stats, strongholds, AccountId};

type Reply<T> = oneshot::Sender<T>;

pub(super) enum StoreCmd {
    Auth {
        request: super::auth_sessions::Request,
        reply: Reply<Result<super::auth_sessions::Reply, super::auth_sessions::Error>>,
    },
    Register {
        name: String,
        name_lower: String,
        key_hash: [u8; 32],
        reply: Reply<Result<(AccountId, Profile), AccountError>>,
    },
    Login {
        name_lower: String,
        key_hash: [u8; 32],
        reply: Reply<Result<(AccountId, Profile, i64), AccountError>>,
    },
    InsertMatch(Value),
    CountMatches(Reply<rusqlite::Result<i64>>),
    CountRanked(Reply<rusqlite::Result<i64>>),
    PlayerStats {
        name_lower: String,
        reply: Reply<Option<ranked::PlayerStats>>,
    },
    Settle {
        entry: Box<JournalEntry>,
        reply: Reply<Result<RankedReceipt, ranked::SettleError>>,
    },
    HighWater(Reply<rusqlite::Result<i64>>),
    SaveStats {
        match_id: i64,
        summary: Box<serde_json::Value>,
        reply: Reply<Result<(), stats::StatsWriteError>>,
    },
    SaveArcanas {
        account: AccountId,
        preset: ArcanaPreset,
    },
    LoadArcanas {
        account: AccountId,
        reply: Reply<Option<ArcanaPreset>>,
    },
    SaveStronghold {
        account: AccountId,
        kind: StrongholdKind,
    },
    LoadStronghold {
        account: AccountId,
        reply: Reply<Option<StrongholdKind>>,
    },
    /// P66: release directory and ownership work (`store::release`).
    Release(Job),
}

/// A closure run on the writer thread with its connection.
pub(super) type Job = Box<dyn FnOnce(&mut rusqlite::Connection) + Send>;

/// Rebuild the leaderboard at most this often.
pub(super) const SNAPSHOT_EVERY: std::time::Duration = std::time::Duration::from_secs(10);

pub(super) fn worker(
    mut conn: rusqlite::Connection,
    rx: std::sync::mpsc::Receiver<StoreCmd>,
    snapshot_tx: watch::Sender<StdArc<Snapshot>>,
    policy: crate::ranked::policy::RankedPolicy,
    pending: StdArc<std::sync::atomic::AtomicU64>,
) {
    // Publish stragglers even when no further commands arrive.
    let mut last_build = Instant::now();
    let mut dirty = false;
    // P66: another slot process writing the same file bumps `data_version`;
    // its ranked results must reach this process's leaderboard too.
    let mut seen_version = data_version(&conn);
    loop {
        match rx.recv_timeout(SNAPSHOT_EVERY) {
            Ok(cmd) => {
                serve(&mut conn, cmd, &policy, &mut dirty);
                pending.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => return,
        }
        let version = data_version(&conn);
        if version != seen_version {
            seen_version = version;
            dirty = true;
        }
        if dirty && last_build.elapsed() >= SNAPSHOT_EVERY {
            match leaderboard::build(&conn) {
                Ok(snap) => {
                    // `send_replace`: readers may not exist yet, and plain
                    // `send` drops the value when receiver_count is zero.
                    snapshot_tx.send_replace(StdArc::new(snap));
                    last_build = Instant::now();
                    dirty = false;
                }
                Err(e) => tracing::error!("leaderboard rebuild failed: {e}"),
            }
        }
    }
}

/// `PRAGMA data_version`: changes when another connection commits.
fn data_version(conn: &rusqlite::Connection) -> i64 {
    conn.query_row("PRAGMA data_version", [], |r| r.get(0))
        .unwrap_or(0)
}

/// Serve one command; `dirty` marks the leaderboard for a rebuild.
fn serve(
    conn: &mut rusqlite::Connection,
    cmd: StoreCmd,
    policy: &crate::ranked::policy::RankedPolicy,
    dirty: &mut bool,
) {
    match cmd {
        StoreCmd::Auth { request, reply } => {
            let _ = reply.send(super::auth_sessions::apply(conn, request));
        }
        StoreCmd::Register {
            name,
            name_lower,
            key_hash,
            reply,
        } => {
            let _ = reply.send(accounts::register(conn, &name, &name_lower, &key_hash));
        }
        StoreCmd::Login {
            name_lower,
            key_hash,
            reply,
        } => {
            let result = accounts::login(conn, &name_lower, &key_hash).and_then(|(id, profile)| {
                ranked::high_water(conn)
                    .map(|seq| (id, profile, seq))
                    .map_err(|_| AccountError::Unavailable)
            });
            let _ = reply.send(result);
        }
        StoreCmd::InsertMatch(m) => {
            if let Err(e) = matches::insert(conn, &m) {
                tracing::error!("match record insert failed: {e}");
            }
        }
        StoreCmd::CountMatches(reply) => {
            let _ = reply.send(matches::count(conn));
        }
        StoreCmd::CountRanked(reply) => {
            let _ = reply.send(ranked::count(conn));
        }
        StoreCmd::PlayerStats { name_lower, reply } => {
            let stats = ranked::player_stats(conn, &name_lower).unwrap_or(None);
            let _ = reply.send(stats);
        }
        StoreCmd::Settle { entry, reply } => {
            let result = ranked::settle(conn, &entry, policy);
            match &result {
                Ok(_) => *dirty = true,
                // Full error kept internally (P60 step 2), never keys.
                Err(e) => {
                    tracing::error!("ranked settlement {} failed: {e:?}", entry.completion_seq)
                }
            }
            let _ = reply.send(result);
        }
        StoreCmd::HighWater(reply) => {
            let _ = reply.send(ranked::high_water(conn));
        }
        StoreCmd::SaveStats {
            match_id,
            summary,
            reply,
        } => {
            let _ = reply.send(stats::save(conn, match_id, &summary));
        }
        StoreCmd::SaveArcanas { account, preset } => {
            if let Err(e) = arcanas::save(conn, account, preset) {
                tracing::error!("arcana preset save failed: {e}");
            }
        }
        StoreCmd::LoadArcanas { account, reply } => {
            let _ = reply.send(arcanas::load(conn, account));
        }
        StoreCmd::SaveStronghold { account, kind } => {
            if let Err(e) = strongholds::save(conn, account, kind) {
                tracing::error!("stronghold kind save failed: {e}");
            }
        }
        StoreCmd::LoadStronghold { account, reply } => {
            let _ = reply.send(strongholds::load(conn, account));
        }
        StoreCmd::Release(job) => job(conn),
    }
}
