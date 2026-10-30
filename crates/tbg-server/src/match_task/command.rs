//! Commands the match task receives: intentions, attach/detach, RTT, and
//! the first `MatchStart` of each seat (also re-sent on attach).

use tbg_core::protocol::{MatchMode, ServerMsg};
use tbg_core::{PlayerIntention, Seat};

use crate::chan::MatchCmd;

use super::Match;

impl Match {
    /// P16-C: log a command for the replay, at the tick it is applied.
    /// Logged even after the outcome: a late intention still sets
    /// `last_seq`, which is part of the state hash.
    pub(super) fn record(&mut self, seat: Seat, intent: Option<(u32, PlayerIntention)>) {
        if let Some(r) = &mut self.replay {
            r.record(&self.state, seat, intent);
        }
    }

    pub(super) fn match_start(&self, seat: Seat) -> ServerMsg {
        let me = &self.players[seat.index()];
        let other = &self.players[seat.other().index()];
        let ranked = self.mode == MatchMode::Ranked;
        ServerMsg::MatchStart {
            you: seat,
            you_name: me.name.clone(),
            you_rating: ranked
                .then(|| me.account.map(|(_, rating)| rating))
                .flatten(),
            opponent: other.name.clone(),
            mode: self.mode,
            opponent_rating: ranked
                .then(|| other.account.map(|(_, rating)| rating))
                .flatten(),
            // D-40: both frozen presets on every (re)attach (BR-AR05).
            arcanas: self.state.players.each_ref().map(|p| p.arcanas),
        }
    }

    pub(super) fn command(&mut self, cmd: MatchCmd) {
        match cmd {
            MatchCmd::Intent { seat, seq, intent } => {
                self.stats[seat.index()].intents += 1;
                // P9: any processed intention earns an ack view this tick.
                self.applied[seat.index()] = true;
                // A replayed or out-of-order intention never mutates twice and
                // never enters the replay log.
                let current = self.state.player(seat).last_seq;
                // Zero is an offline simulation sentinel, never an online
                // sequence: accepting it resets last_seq and permits replays.
                let stale = seq == 0 || seq <= current;
                let result = if stale {
                    Err(tbg_core::Rejection::Stale)
                } else {
                    self.record(seat, Some((seq, intent)));
                    self.state.apply_intention(seat, seq, intent)
                };
                if result.is_ok() {
                    self.count_success(seat, intent);
                }
                if let Err(reason) = result {
                    self.stats[seat.index()].rejections += 1;
                    self.metrics
                        .rejections
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    self.send(seat, ServerMsg::Rejected { seq, reason });
                }
            }
            MatchCmd::Surrender { seat } => {
                // BR-M06/BR-M03: forfeit, idempotent (a second surrender or
                // an already-finished match changes nothing).
                if self.state.outcome.is_none() {
                    self.record(seat, None);
                    self.state.forfeit(seat);
                    self.stats[seat.index()].surrendered = true;
                }
            }
            MatchCmd::Detach { seat, conn_id } => {
                let current = self.players[seat.index()].conn.as_ref().map(|c| c.id);
                if current == Some(conn_id) {
                    self.detach(seat);
                }
            }
            MatchCmd::Attach {
                seat,
                conn,
                new_token,
            } => {
                // P11: a login takeover moves the seat to the new session
                // token so the end-of-match report finds the right session.
                if let Some(t) = new_token {
                    self.players[seat.index()].token = t;
                }
                self.players[seat.index()].conn = Some(conn);
                self.detached_at[seat.index()] = None;
                self.stats[seat.index()].reattached += 1;
                self.applied[seat.index()] = false;
                let start = self.match_start(seat);
                self.send(seat, start);
                // Catch up on events buffered while detached.
                self.send_view(seat);
            }
            MatchCmd::Rtt { seat, ms } => self.stats[seat.index()].rtts.push(ms),
        }
    }
}
