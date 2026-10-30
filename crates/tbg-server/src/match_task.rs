//! One task per match: owns the `MatchState`, ticks it at a fixed rate,
//! applies seat-stamped intentions and sends per-seat views (incl. ack views).

use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::mpsc;
use tokio::time::MissedTickBehavior;

use tbg_core::protocol::{MatchMode, ServerMsg, Token};
use tbg_core::replay::Replay;
use tbg_core::{Event, MatchState, Seat};

use crate::chan::{ConnHandle, LobbyCmd, MatchCmd, MATCH_QUEUE};
use crate::metrics::Metrics;
use crate::store::{AccountId, Store};
use crate::Config;

mod command;
mod report;
#[cfg(test)]
mod tests;
mod views;
use report::SeatStats;

pub(crate) struct Player {
    pub token: Token,
    pub name: String,
    pub conn: Option<ConnHandle>,
    /// P12: account id + rating at pairing time (`None` for guests/casual).
    pub account: Option<(AccountId, u16)>,
    /// D-40: the Arcanas frozen at pairing (BR-AR05).
    pub arcanas: tbg_core::ArcanaPreset,
    /// D-46: the Stronghold kind frozen at pairing (BR-SK01).
    pub stronghold: tbg_core::StrongholdKind,
}

pub(super) struct Match {
    pub(super) state: MatchState,
    pub(super) players: [Player; 2],
    pub(super) detached_at: [Option<Instant>; 2],
    pub(super) stats: [SeatStats; 2],
    /// P9: per-seat buffered events; cleared on every view to that seat.
    pub(super) pending: [Vec<Event>; 2],
    /// P9: a seat gets an extra view when its intention was applied/rejected.
    pub(super) applied: [bool; 2],
    /// P12: ranked matches report the result to the store transaction.
    pub(super) mode: MatchMode,
    pub(super) metrics: Arc<Metrics>,
    /// P16-C: recorded only when a test asked for replays (`Config::replays`).
    pub(super) replay: Option<Replay>,
}

pub(crate) fn spawn(
    seed: u64,
    players: [Player; 2],
    mode: MatchMode,
    lobby: mpsc::Sender<LobbyCmd>,
    config: Arc<Config>,
    metrics: Arc<Metrics>,
    store: Option<Arc<Store>>,
) -> mpsc::Sender<MatchCmd> {
    let (tx, rx) = mpsc::channel(MATCH_QUEUE);
    let detached_at = players
        .each_ref()
        .map(|p| p.conn.is_none().then(Instant::now));
    let arcanas = players.each_ref().map(|p| p.arcanas);
    let strongholds = players.each_ref().map(|p| p.stronghold);
    let m = Match {
        state: MatchState::with_arcanas(seed, arcanas).with_strongholds(strongholds),
        players,
        detached_at,
        stats: Default::default(),
        pending: [Vec::new(), Vec::new()],
        applied: [false; 2],
        mode,
        metrics,
        replay: config.replays.as_ref().map(|_| Replay {
            seed,
            arcanas,
            strongholds,
            ..Replay::default()
        }),
    };
    tokio::spawn(run(m, rx, lobby, config, seed, store));
    tx
}

async fn run(
    mut m: Match,
    mut rx: mpsc::Receiver<MatchCmd>,
    lobby: mpsc::Sender<LobbyCmd>,
    config: Arc<Config>,
    seed: u64,
    store: Option<Arc<Store>>,
) {
    m.metrics.matches_running.fetch_add(1, Relaxed);
    let started = Instant::now();
    let tick = Duration::from_millis(config.tick_ms);
    let window = Duration::from_secs(config.reconnect_secs);
    for seat in Seat::BOTH {
        let start = m.match_start(seat);
        m.send(seat, start);
    }
    let mut interval = tokio::time::interval(tick);
    interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
    let (mut last, mut step_ns, mut step_max, mut late, mut max_late) =
        (Instant::now(), 0u128, 0u128, 0u32, 0u128);
    loop {
        interval.tick().await;
        let now = Instant::now();
        let lateness = now.duration_since(last).saturating_sub(tick).as_millis();
        last = now;
        if lateness > (tick.as_millis() / 2).max(1) {
            late += 1;
            m.metrics.late_ticks.fetch_add(1, Relaxed);
        }
        max_late = max_late.max(lateness);
        m.metrics.ticks.fetch_add(1, Relaxed);
        while let Ok(cmd) = rx.try_recv() {
            m.command(cmd);
        }
        let t0 = Instant::now();
        let ev = m.state.step();
        let ns = t0.elapsed().as_nanos();
        (step_ns, step_max) = (step_ns + ns, step_max.max(ns));
        for p in &mut m.pending {
            p.extend(ev.iter().cloned());
        }
        // Forfeits first, so a forfeit's final view still goes out below.
        for seat in Seat::BOTH {
            if m.detached_at[seat.index()].is_some_and(|t| t.elapsed() > window) {
                m.record(seat, None);
                m.state.forfeit(seat);
            }
        }
        // 10 Hz views, plus an ack view to a seat whose intention was processed.
        let ended = m.state.outcome.is_some();
        // The selection clock keeps 10 Hz views flowing during an Augment
        // pause, when the combat tick is frozen (BR-AG13).
        let regular = m.state.clock.is_multiple_of(2) || ended;
        for seat in Seat::BOTH {
            let ack = m.applied[seat.index()] && !regular;
            if regular || m.applied[seat.index()] {
                m.send_view(seat, ack);
            }
        }
        m.applied = [false; 2];
        if let Some(outcome) = m.state.outcome {
            for seat in Seat::BOTH {
                m.send(seat, ServerMsg::MatchEnd { outcome });
            }
            break;
        }
    }
    if let (Some(mut replay), Some(tx)) = (m.replay.take(), &config.replays) {
        replay.final_hash = m.state.hash();
        let _ = tx.send(replay);
    }
    let ticks = u128::from(m.state.clock.max(1));
    m.metrics.matches_running.fetch_sub(1, Relaxed);
    m.metrics.matches_finished.fetch_add(1, Relaxed);
    // P12: ranked results settle in one store transaction; the lobby
    // delivers `RatingChanged` when it lands (after `MatchEnded` below).
    // A store failure logs inside and sends nothing — the match already
    // ended normally.
    if m.mode == MatchMode::Ranked {
        if let (Some((a_id, _)), Some((b_id, _))) = (m.players[0].account, m.players[1].account) {
            if let Some(db) = store.clone() {
                let outcome = m.state.outcome;
                let tokens = m.players.each_ref().map(|p| p.token);
                let duration_s = started.elapsed().as_secs();
                let factions = [m.stats[0].factions, m.stats[1].factions];
                let arcanas = m.state.players.each_ref().map(|p| p.arcanas);
                let tx = lobby.clone();
                tokio::spawn(async move {
                    match db
                        .ranked_result(a_id, b_id, outcome, duration_s, factions, arcanas)
                        .await
                    {
                        Ok(results) => {
                            let _ = tx.send(LobbyCmd::RankedDone { tokens, results }).await;
                        }
                        Err(e) => tracing::error!("ranked result failed: {e:?}"),
                    }
                });
            }
        }
    }
    let _ = lobby
        .send(LobbyCmd::MatchEnded {
            tokens: m.players.each_ref().map(|p| p.token),
        })
        .await;
    let line = m.log_end(
        seed,
        started,
        (step_ns / ticks) as u64 / 1000,
        (step_max / 1000) as u64,
        late,
        max_late as u64,
    );
    // P11: fire-and-forget over the store channel; never blocks the loop.
    if let Some(db) = store {
        db.insert_match(&line);
    }
}
