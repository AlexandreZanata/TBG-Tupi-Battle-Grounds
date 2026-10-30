//! The single writer thread: owns the SQLite connection, serves every
//! command in order, and rebuilds the leaderboard snapshot at most every
//! 10 s once ranked results landed (P13).

use std::sync::Arc as StdArc;
use std::time::Instant;

use serde_json::Value;
use tokio::sync::{oneshot, watch};

use tbg_core::protocol::{AccountError, Profile, RatingResults};
use tbg_core::{ArcanaPreset, Outcome, StrongholdKind};

use super::leaderboard::{self, Snapshot};
use super::{accounts, arcanas, matches, ranked, strongholds, AccountId};

type Reply<T> = oneshot::Sender<T>;

pub(super) enum StoreCmd {
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
    PlayerStats {
        name_lower: String,
        reply: Reply<Option<ranked::PlayerStats>>,
    },
    RankedResult {
        a: AccountId,
        b: AccountId,
        outcome: Option<Outcome>,
        duration_s: u64,
        factions: [[u32; 4]; 2],
        arcanas: [ArcanaPreset; 2],
        reply: Reply<Result<RatingResults, AccountError>>,
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
}

/// Rebuild the leaderboard at most this often.
pub(super) const SNAPSHOT_EVERY: std::time::Duration = std::time::Duration::from_secs(10);

pub(super) fn worker(
    mut conn: rusqlite::Connection,
    rx: std::sync::mpsc::Receiver<StoreCmd>,
    snapshot_tx: watch::Sender<StdArc<Snapshot>>,
) {
    // Publish stragglers even when no further commands arrive.
    let mut last_build = Instant::now();
    let mut dirty = false;
    loop {
        match rx.recv_timeout(SNAPSHOT_EVERY) {
            Ok(cmd) => match cmd {
                StoreCmd::Register {
                    name,
                    name_lower,
                    key_hash,
                    reply,
                } => {
                    let _ =
                        reply.send(accounts::register(&mut conn, &name, &name_lower, &key_hash));
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
                StoreCmd::PlayerStats { name_lower, reply } => {
                    let stats = ranked::player_stats(&conn, &name_lower).unwrap_or(None);
                    let _ = reply.send(stats);
                }
                StoreCmd::RankedResult {
                    a,
                    b,
                    outcome,
                    duration_s,
                    factions,
                    arcanas,
                    reply,
                } => {
                    let result = ranked::ranked_result(
                        &mut conn, a, b, outcome, duration_s, factions, arcanas,
                    )
                    .map_err(|_| AccountError::Unavailable);
                    if result.is_ok() {
                        dirty = true;
                    }
                    let _ = reply.send(result);
                }
                StoreCmd::SaveArcanas { account, preset } => {
                    if let Err(e) = arcanas::save(&conn, account, preset) {
                        tracing::error!("arcana preset save failed: {e}");
                    }
                }
                StoreCmd::LoadArcanas { account, reply } => {
                    let _ = reply.send(arcanas::load(&conn, account));
                }
                StoreCmd::SaveStronghold { account, kind } => {
                    if let Err(e) = strongholds::save(&conn, account, kind) {
                        tracing::error!("stronghold kind save failed: {e}");
                    }
                }
                StoreCmd::LoadStronghold { account, reply } => {
                    let _ = reply.send(strongholds::load(&conn, account));
                }
            },
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => return,
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
