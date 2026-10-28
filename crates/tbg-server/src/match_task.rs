//! One task per match: owns the `MatchState`, ticks it at a fixed rate,
//! applies seat-stamped intentions, sends per-seat views every 2nd tick,
//! and forfeits a seat that stays detached past the reconnect window.

use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::mpsc;
use tokio::time::MissedTickBehavior;

use tbg_core::protocol::{ServerMsg, Token};
use tbg_core::{Event, MatchState, Seat};

use crate::chan::{ConnEvent, ConnHandle, LobbyCmd, MatchCmd, MATCH_QUEUE};
use crate::metrics::Metrics;
use crate::Config;

mod report;

pub(crate) struct Player {
    pub token: Token,
    pub name: String,
    pub conn: Option<ConnHandle>,
}

#[derive(Default)]
pub(super) struct SeatStats {
    pub(super) intents: u32,
    pub(super) rejections: u32,
    pub(super) rtts: Vec<u16>,
    pub(super) reattached: u32,
}

pub(super) struct Match {
    pub(super) state: MatchState,
    pub(super) players: [Player; 2],
    pub(super) detached_at: [Option<Instant>; 2],
    pub(super) stats: [SeatStats; 2],
    pub(super) events: Vec<Event>,
    pub(super) metrics: Arc<Metrics>,
}

pub(crate) fn spawn(
    seed: u64,
    players: [Player; 2],
    lobby: mpsc::Sender<LobbyCmd>,
    config: Arc<Config>,
    metrics: Arc<Metrics>,
) -> mpsc::Sender<MatchCmd> {
    let (tx, rx) = mpsc::channel(MATCH_QUEUE);
    let m = Match {
        state: MatchState::new(seed),
        players,
        detached_at: [None, None],
        stats: Default::default(),
        events: Vec::new(),
        metrics,
    };
    tokio::spawn(run(m, rx, lobby, config, seed));
    tx
}

async fn run(
    mut m: Match,
    mut rx: mpsc::Receiver<MatchCmd>,
    lobby: mpsc::Sender<LobbyCmd>,
    config: Arc<Config>,
    seed: u64,
) {
    m.metrics.matches_running.fetch_add(1, Relaxed);
    let started = Instant::now();
    let tick = Duration::from_millis(config.tick_ms);
    let window = Duration::from_secs(config.reconnect_secs);
    for seat in Seat::BOTH {
        let opponent = m.players[seat.other().index()].name.clone();
        m.send(
            seat,
            ServerMsg::MatchStart {
                you: seat,
                opponent,
            },
        );
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
        m.events.extend(ev);
        if m.state.tick.is_multiple_of(2) || m.state.outcome.is_some() {
            m.send_views();
        }
        for seat in Seat::BOTH {
            if m.detached_at[seat.index()].is_some_and(|t| t.elapsed() > window) {
                m.state.forfeit(seat);
            }
        }
        if let Some(outcome) = m.state.outcome {
            m.send_views();
            for seat in Seat::BOTH {
                m.send(seat, ServerMsg::MatchEnd { outcome });
            }
            break;
        }
    }
    let ticks = u128::from(m.state.tick.max(1));
    m.metrics.matches_running.fetch_sub(1, Relaxed);
    m.metrics.matches_finished.fetch_add(1, Relaxed);
    let _ = lobby
        .send(LobbyCmd::MatchEnded {
            tokens: m.players.each_ref().map(|p| p.token),
        })
        .await;
    m.log_end(
        seed,
        started,
        (step_ns / ticks) as u64 / 1000,
        (step_max / 1000) as u64,
        late,
        max_late as u64,
    );
}

impl Match {
    fn command(&mut self, cmd: MatchCmd) {
        match cmd {
            MatchCmd::Intent { seat, seq, intent } => {
                self.stats[seat.index()].intents += 1;
                if let Err(reason) = self.state.apply_intention(seat, seq, intent) {
                    self.stats[seat.index()].rejections += 1;
                    self.metrics.rejections.fetch_add(1, Relaxed);
                    self.send(seat, ServerMsg::Rejected { seq, reason });
                }
            }
            MatchCmd::Detach { seat, conn_id } => {
                let current = self.players[seat.index()].conn.as_ref().map(|c| c.id);
                if current == Some(conn_id) {
                    self.detach(seat);
                }
            }
            MatchCmd::Attach { seat, conn } => {
                self.players[seat.index()].conn = Some(conn);
                self.detached_at[seat.index()] = None;
                self.stats[seat.index()].reattached += 1;
                let opponent = self.players[seat.other().index()].name.clone();
                self.send(
                    seat,
                    ServerMsg::MatchStart {
                        you: seat,
                        opponent,
                    },
                );
                self.send(seat, ServerMsg::View(self.state.view_for(seat, &[])));
            }
            MatchCmd::Rtt { seat, ms } => self.stats[seat.index()].rtts.push(ms),
        }
    }

    fn detach(&mut self, seat: Seat) {
        self.players[seat.index()].conn = None;
        self.detached_at[seat.index()] = Some(Instant::now());
    }

    /// Never blocks: a full client queue means a too-slow client; drop it.
    fn send(&mut self, seat: Seat, msg: ServerMsg) {
        let Some(conn) = &self.players[seat.index()].conn else {
            return;
        };
        match conn.tx.try_send(ConnEvent::Send(msg)) {
            Ok(()) => {}
            Err(mpsc::error::TrySendError::Full(_)) => {
                let _ = conn.tx.try_send(ConnEvent::Close);
                self.detach(seat);
            }
            Err(mpsc::error::TrySendError::Closed(_)) => self.detach(seat),
        }
    }
}
