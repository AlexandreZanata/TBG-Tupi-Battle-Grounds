//! One task per match: owns the `MatchState`, ticks it at a fixed rate,
//! applies seat-stamped intentions and sends per-seat views (incl. ack views).

use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::mpsc;
use tokio::time::MissedTickBehavior;

use tbg_core::protocol::{LiveId, MatchMode, Token};
use tbg_core::replay::Replay;
use tbg_core::{Event, MatchState, Phase, Seat};

use crate::chan::{ConnHandle, LobbyCmd, MatchCmd, MATCH_QUEUE};
use crate::metrics::Metrics;
use crate::ranked::{finalizer, EndDetail, MatchKey};
use crate::store::{AccountId, Store};
use crate::Config;

mod celebration;
mod command;
mod live;
mod personal;
mod report;
mod settle;
mod social;
mod spectators;
pub(crate) mod summary;
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
    pub(super) social: social::Social,
    pub(super) result_id: tbg_core::protocol::ResultId,
    pub(super) state: MatchState,
    pub(super) players: [Player; 2],
    pub(super) detached_at: [Option<Instant>; 2],
    /// P71 (BR-M03, Proposed): connection-pause budget left per seat.
    pub(super) pause_left: [Duration; 2],
    /// P71: only a seat that has been connected can pause the match.
    pub(super) seen: [bool; 2],
    /// P71: this tick is a connection pause (both clocks stand still).
    pub(super) paused: bool,
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
    /// P47: the listed-match id; `None` for a private match, which reports nothing.
    pub(super) live: Option<LiveId>,
    /// P47 (BR-LV02): listen-only connections; frames only, never a seat.
    pub(super) spectators: Vec<ConnHandle>,
    /// P47: public events since the last spectator view.
    pub(super) pending_spectate: Vec<Event>,
    /// P60: the first cause that actually ended the match.
    pub(super) end_detail: Option<EndDetail>,
    /// P60: latched by `PhaseChanged { Running }`, never inferred.
    pub(super) reached_running: bool,
    /// P61: bounded side summary (ranked matches only).
    pub(super) summary: summary::SummaryCollector,
}

/// Where a finished match's records go: the fire-and-forget match row
/// (P11) and, for ranked play, the ordered finalizer (P60) under the
/// match's OS-random key drawn at pairing.
#[derive(Clone, Default)]
pub(crate) struct Records {
    pub store: Option<Arc<Store>>,
    pub history: Option<crate::player_history::Handle>,
    pub origin: tbg_core::protocol::MatchOrigin,
    pub ranked: Option<finalizer::Handle>,
    pub key: MatchKey,
    /// P61: collect the optional summary (`TBG_RANKED_STATS=collect`).
    pub stats: bool,
}

/// `(seed, arena, live)` are the match's pairing draws (ARENAS-G) and, for a
/// public match, its listed id (P47).
pub(crate) fn spawn(
    (seed, arena, live): (u64, tbg_core::Arena, Option<LiveId>),
    players: [Player; 2],
    mode: MatchMode,
    lobby: mpsc::Sender<LobbyCmd>,
    config: Arc<Config>,
    metrics: Arc<Metrics>,
    records: Records,
) -> mpsc::Sender<MatchCmd> {
    let (tx, rx) = mpsc::channel(MATCH_QUEUE);
    let detached_at = players
        .each_ref()
        .map(|p| p.conn.is_none().then(Instant::now));
    let seen = players.each_ref().map(|p| p.conn.is_some());
    let arcanas = players.each_ref().map(|p| p.arcanas);
    let strongholds = players.each_ref().map(|p| p.stronghold);
    let m = Match {
        result_id: crate::names::random_bytes::<16>(),
        state: MatchState::with_arcanas(seed, arcanas)
            .with_strongholds(strongholds)
            .with_arena(arena),
        players,
        detached_at,
        pause_left: [Duration::from_secs(config.pause_secs); 2],
        seen,
        paused: false,
        stats: Default::default(),
        pending: [Vec::new(), Vec::new()],
        applied: [false; 2],
        mode,
        metrics,
        replay: config.replays.as_ref().map(|_| Replay {
            seed,
            arcanas,
            strongholds,
            arena,
            ..Replay::default()
        }),
        live,
        spectators: Vec::new(),
        pending_spectate: Vec::new(),
        end_detail: None,
        reached_running: false,
        summary: Default::default(),
        social: Default::default(),
    };
    tokio::spawn(run(m, rx, lobby, config, seed, records));
    tx
}

async fn run(
    mut m: Match,
    mut rx: mpsc::Receiver<MatchCmd>,
    lobby: mpsc::Sender<LobbyCmd>,
    config: Arc<Config>,
    seed: u64,
    records: Records,
) {
    m.social.config = Some(config.clone());
    if let Some(store) = &records.store {
        let accounts = m.players.each_ref().map(|p| p.account.map(|(id, _)| id));
        if let Some(Ok(profiles)) = store
            .release(move |db| crate::store::social::profiles(db, accounts))
            .await
        {
            for (seat, (verified, slots)) in profiles.into_iter().enumerate() {
                m.social.verified[seat] = verified;
                m.social.loadouts[seat] = slots;
            }
        }
    }
    m.metrics.matches_running.fetch_add(1, Relaxed);
    let started = Instant::now();
    let tick = Duration::from_millis(config.tick_ms);
    let window = Duration::from_secs(config.reconnect_secs);
    for seat in Seat::BOTH {
        let start = m.match_start(seat);
        m.send(seat, start);
        m.send_social(seat);
    }
    let mut interval = tokio::time::interval(tick);
    interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
    let (mut last, mut step_ns, mut step_max, mut late, mut max_late) =
        (Instant::now(), 0u128, 0u128, 0u32, 0u128);
    let mut held = 0u64;
    let (finished, history_envelope) = loop {
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
        m.paused = m.connection_pause(tick);
        let ev = if m.paused {
            held += 1;
            Vec::new()
        } else {
            let t0 = Instant::now();
            let ev = m.state.step();
            let ns = t0.elapsed().as_nanos();
            m.metrics.record_step((ns / 1_000) as u64);
            (step_ns, step_max) = (step_ns + ns, step_max.max(ns));
            ev
        };
        if ev.iter().any(|e| {
            matches!(
                e,
                Event::PhaseChanged {
                    phase: Phase::Running
                }
            )
        }) {
            m.reached_running = true;
        }
        // P61: fold this tick's semantic events once, before fan-out.
        m.summary.observe(&ev, &m.state);
        m.report_progress(&lobby, started);
        m.note_spectate_events(&ev);
        for p in &mut m.pending {
            p.extend(ev.iter().cloned());
        }
        // Forfeits first, so a forfeit's final view still goes out below.
        for seat in Seat::BOTH {
            let pausing = m.seen[seat.index()] && !m.pause_left[seat.index()].is_zero();
            if !pausing && m.detached_at[seat.index()].is_some_and(|t| t.elapsed() > window) {
                m.record(seat, None);
                let unfinished = m.state.outcome.is_none();
                m.state.forfeit(seat);
                if unfinished && m.state.outcome.is_some() {
                    m.end_detail.get_or_insert(EndDetail::Disconnect);
                }
            }
        }
        // 10 Hz views, plus an ack view to a seat whose intention was processed.
        let ended = m.state.outcome.is_some();
        // The selection clock keeps 10 Hz views flowing during an Augment
        // pause, when the combat tick is frozen (BR-AG13).
        // P71: a connection pause freezes the clock; views keep their 10 Hz.
        let beat = if m.paused {
            held
        } else {
            u64::from(m.state.clock)
        };
        let regular = beat.is_multiple_of(2) || ended;
        for seat in Seat::BOTH {
            let ack = m.applied[seat.index()] && !regular;
            if regular || m.applied[seat.index()] {
                m.send_view(seat, ack);
            }
        }
        m.applied = [false; 2];
        if regular {
            m.send_spectator_views();
        }
        if let Some(outcome) = m.state.outcome {
            let summaries = m.personal_summaries(
                records.origin,
                started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64,
            );
            let envelope = records
                .history
                .as_ref()
                .and_then(|_| m.history_envelope(records.key, summaries.clone()));
            let finished = Box::new(Seat::BOTH.map(|seat| {
                m.completed(
                    seat,
                    outcome,
                    summaries[seat.index()].clone(),
                    envelope.as_ref(),
                )
            }));
            for seat in Seat::BOTH {
                let result = &finished[seat.index()];
                if let Some(summary) = &result.personal {
                    m.send(seat, summary.clone());
                }
                if let Some(status) = &result.history {
                    m.send(seat, status.clone());
                }
                m.send(seat, result.end.clone());
            }
            m.end_spectators(outcome);
            break (finished, envelope);
        }
    };
    if let (Some(mut replay), Some(tx)) = (m.replay.take(), &config.replays) {
        replay.final_hash = m.state.hash();
        let _ = tx.send(replay);
    }
    if let Some(store) = &records.store {
        m.social_record(
            store,
            config.auth_clock.now().unwrap_or(0),
            started.elapsed().as_secs(),
        );
    }
    let ticks = u128::from(m.state.clock.max(1));
    m.metrics.matches_finished.fetch_add(1, Relaxed);
    // P60: a ranked result goes to the single finalizer, which journals it
    // durably and settles results in completion order; the lobby delivers
    // `RatingChanged` by frozen account id once it commits. Awaiting here
    // only delays this finished task, never a running match.
    if m.mode == MatchMode::Ranked {
        if let Some(result) = m.ranked_final(
            records.key,
            started,
            records.stats,
            records.history.is_some(),
        ) {
            match &records.ranked {
                Some(handle) => {
                    // P66: counted before the hand-off so retirement never
                    // sees a gap between "match ended" and "result queued".
                    m.metrics.ranked_inflight.fetch_add(1, Relaxed);
                    if handle.tx.send(result).await.is_err() {
                        m.metrics.ranked_inflight.fetch_sub(1, Relaxed);
                        tracing::error!("ranked finalizer stopped; result not settled");
                    }
                }
                None => tracing::error!("ranked match ended without a ranked store"),
            }
        }
    }
    let _ = lobby
        .send(LobbyCmd::MatchEnded {
            tokens: m.players.each_ref().map(|p| p.token),
            live: m.live,
            finished,
        })
        .await;
    if let (Some(handle), Some(envelope)) = (&records.history, history_envelope) {
        if !handle.submit(envelope.clone()) {
            for side in &envelope.sides {
                if let Some(account) = side.account {
                    let _ = lobby
                        .send(LobbyCmd::HistoryDone {
                            account,
                            result_id: m.result_id,
                            entry_id: side.entry_id,
                            state: tbg_core::protocol::HistorySaveState::Delayed,
                        })
                        .await;
                }
            }
        }
    }
    let line = m.log_end(
        seed,
        started,
        (step_ns / ticks) as u64 / 1000,
        (step_max / 1000) as u64,
        late,
        max_late as u64,
    );
    // P11: fire-and-forget over the store channel; never blocks the loop.
    if let Some(db) = records.store {
        db.insert_match(&line);
    }
    // P66: "running" ends only once the ranked result and the match record
    // are queued (both counted as pending), so the retirement gate never
    // sees an idle slot while a result is between the two.
    m.metrics.matches_running.fetch_sub(1, Relaxed);
}
