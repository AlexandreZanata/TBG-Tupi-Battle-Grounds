//! Socket pump and reconnect backoff: 0.25 s, 0.5 s, 1 s, then every 2 s
//! until `GIVE_UP` (the server window is 30 s). A frame that runs after the
//! tab becomes visible again retries at once when a retry is due. A socket
//! that goes silent is treated as lost (P71): a dropped network often never
//! reports `Closed`, and the seat is only safe if the client comes back.

use ewebsock::{WsEvent, WsMessage};
use tbg_core::protocol::{self, ClientMsg, ServerMsg};

use super::{NetFailure, NetLink, NetStatus};
use crate::config;

/// Give up reconnecting after this long (server window is 30 s).
pub(crate) const GIVE_UP: f64 = 30.0;

/// The server pings every second; this much silence means the link is gone.
/// Above the server's 3 s stale-controller limit, so the resume takes the seat back.
const SILENT: f64 = 4.0;

fn delay(tries: u32) -> f64 {
    match tries {
        0 => 0.25,
        1 => 0.5,
        2 => 1.0,
        _ => 2.0,
    }
}

impl NetLink {
    pub(super) fn connect(&mut self, now: f64) {
        if self.restore_pending {
            return;
        }
        self.opened = false;
        self.heard_at = None;
        self.outq.clear();
        let view = crate::auth::snapshot();
        if view.access.is_none() && view.status != crate::auth::Status::Guest {
            return;
        }
        // P66: no socket until admission chose the slot path.
        let super::admission::Route::Ready { url, endpoint, .. } = &self.route else {
            return;
        };
        let url = url.clone();
        // D-97 + P66: auth HTTP must reach the generation that verifies this Access.
        crate::auth::set_endpoint(endpoint);
        self.auth_epoch = view.epoch;
        self.auth_access = view.access;
        self.auth_family = view.family;
        self.login_in_flight = self.auth_access.is_some();
        match ewebsock::connect(url, ewebsock::Options::default()) {
            Ok(pair) => self.socket = Some(pair),
            Err(_) => self.lost(now),
        }
    }

    pub(super) fn lost(&mut self, now: f64) {
        self.socket = None;
        self.opened = false;
        if matches!(self.status, NetStatus::Failed(_)) {
            return;
        }
        // A failed retry must not restart the backoff or the countdown.
        let first = self.lost_at.is_none();
        let since = *self.lost_at.get_or_insert(now);
        self.status = NetStatus::Reconnecting { since };
        if first {
            self.tries = 0;
            self.next_try = now + delay(0);
        }
    }

    pub(super) fn read_socket(&mut self, now: f64) {
        let mut events = Vec::new();
        if let Some((_, rx)) = &self.socket {
            while let Some(ev) = rx.try_recv() {
                events.push(ev);
            }
        }
        for ev in events {
            match ev {
                WsEvent::Opened => {
                    self.opened = true;
                    self.heard_at = Some(now);
                    let hello = self.hello();
                    self.outq.push_front((now + self.lag, hello));
                }
                WsEvent::Message(WsMessage::Binary(bytes)) => {
                    self.heard_at = Some(now);
                    if let Ok(msg) = protocol::decode_server(&bytes) {
                        self.inq.push_back((now + self.lag, msg));
                    }
                }
                WsEvent::Message(_) => {}
                WsEvent::Closed | WsEvent::Error(_) => self.lost(now),
            }
        }
        if self.opened && self.heard_at.is_some_and(|at| now - at > SILENT) {
            self.lost(now);
        }
    }

    pub(super) fn flush(&mut self, now: f64) {
        let Some((tx, _)) = &mut self.socket else {
            return;
        };
        while self.opened && self.outq.front().is_some_and(|(at, _)| *at <= now) {
            if let Some((_, msg)) = self.outq.pop_front() {
                tx.send(WsMessage::Binary(protocol::encode(&msg)));
            }
        }
    }

    pub(super) fn reconnect(&mut self, now: f64) {
        let Some(lost) = self.lost_at else { return };
        if self.socket.is_some() || matches!(self.status, NetStatus::Failed(_)) {
            return;
        }
        if now - lost > GIVE_UP {
            self.status = NetStatus::Failed(NetFailure::ConnectionLost);
        } else if now >= self.next_try {
            self.connect(now);
            self.tries += 1;
            self.next_try = now + delay(self.tries);
        }
    }

    /// P71: views that keep arriving with a clock that stands still mean the
    /// server paused the match for a player's reconnect (BR-M03).
    fn note_clock(&mut self, clock: u64, now: f64) {
        self.view_at = now;
        if self.clock_moved.is_none_or(|(moved, _)| moved != clock) {
            self.clock_moved = Some((clock, now));
        }
    }

    pub fn match_paused(&self, now: f64) -> bool {
        self.in_match
            && now - self.view_at < 0.5
            && self.clock_moved.is_some_and(|(_, since)| now - since > 1.0)
    }

    /// Protocol housekeeping stays here; everything else goes to the game.
    pub(super) fn handle(&mut self, msg: ServerMsg, now: f64) -> Option<ServerMsg> {
        if let ServerMsg::View(v) = &msg {
            self.note_clock(u64::from(v.clock), now);
        }
        match msg {
            ServerMsg::MatchStart { result_id, .. } | ServerMsg::MatchEnd { result_id, .. }
                if self.discarded_result == Some(result_id) =>
            {
                None
            }
            ServerMsg::RematchOpen { .. } if self.discarded_result.is_some() && !self.in_match => {
                None
            }
            ServerMsg::Welcome { token, .. } => {
                config::save_token(&token);
                // P66: remember where this session lives (its generation).
                if let super::admission::Route::Ready { endpoint, .. } = &self.route {
                    config::save_resume_endpoint(endpoint);
                }
                self.token = Some(token);
                self.save_resume_family();
                self.status = NetStatus::Ready;
                self.lost_at = None;
                if !self.in_match && !matches!(self.mode, super::Mode::Probe) {
                    if self.auth_access.is_some() {
                        self.held_request = true;
                    } else {
                        self.send_request(now);
                    }
                }
                None
            }
            ServerMsg::AuthState { status, .. } => {
                crate::auth::socket_status(status);
                if status == protocol::AuthStatus::Active {
                    self.login_in_flight = false;
                    if !self.in_match {
                        self.release_request(now);
                    }
                } else {
                    self.login_in_flight = true;
                }
                None
            }
            ServerMsg::AuthRequired { reason } => {
                crate::auth::authentication_required(reason);
                self.login_in_flight = true;
                self.held_request |= !self.in_match && !matches!(self.mode, super::Mode::Probe);
                None
            }
            ServerMsg::LoggedIn { .. } | ServerMsg::AccountError { .. } => Some(msg),
            ServerMsg::Ping { id } => {
                self.send(ClientMsg::Pong { id }, now);
                None
            }
            ServerMsg::VersionMismatch { .. } => {
                self.status = NetStatus::Failed(NetFailure::VersionMismatch);
                crate::platform::storage::set(
                    crate::platform::storage::Scope::Session,
                    "update.required",
                    "1",
                );
                None
            }
            ServerMsg::View(view) => {
                self.ack.resolved(view.last_seq, now);
                Some(ServerMsg::View(view))
            }
            ServerMsg::Rejected { seq, reason } => {
                self.ack.resolved(seq, now);
                Some(ServerMsg::Rejected { seq, reason })
            }
            other => {
                match other {
                    // Once acknowledged, invites and result offers are resumed
                    // by the server. Reissuing the original request replaces
                    // the host invite or consumes the guest invite twice.
                    ServerMsg::ChallengeWaiting { .. } | ServerMsg::RematchOpen { .. } => {
                        self.mode = super::Mode::Probe;
                    }
                    ServerMsg::MatchStart { result_id, .. } => {
                        self.discarded_result = None;
                        config::clear_dismissed_result();
                        self.result_id = Some(result_id);
                        self.mode = super::Mode::Probe;
                        self.in_match = true;
                        self.login_in_flight = false;
                        self.held_request = false;
                        self.login_deadline = None;
                    }
                    ServerMsg::MatchEnd { result_id, .. } if self.result_id == Some(result_id) => {
                        self.in_match = false
                    }
                    // P47: a watch starts a listen-only link; the end or a refusal
                    // leaves a probe that never asks for the watch again.
                    ServerMsg::SpectateStart { .. } => self.watching = true,
                    ServerMsg::SpectateEnd { .. } | ServerMsg::SpectateUnavailable { .. } => {
                        self.watching = false;
                        self.mode = super::Mode::Probe;
                    }
                    _ => {}
                }
                Some(other)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::Mode;

    fn link(mode: Mode) -> NetLink {
        NetLink {
            restore_pending: false,
            mode,
            socket: None,
            opened: false,
            heard_at: None,
            clock_moved: None,
            view_at: 0.0,
            token: Some([1; 16]),
            in_match: false,
            result_id: None,
            discarded_result: None,
            status: NetStatus::Ready,
            lost_at: None,
            tries: 0,
            next_try: 0.0,
            lag: 0.0,
            ack: Default::default(),
            login_in_flight: false,
            auth_epoch: crate::auth::epoch(),
            auth_access: None,
            auth_family: None,
            held_request: false,
            login_deadline: None,
            outq: Default::default(),
            inq: Default::default(),
            watching: false,
            route: crate::net::admission::Route::Ready {
                url: "ws://127.0.0.1:8080/ws".into(),
                endpoint: "/ws".into(),
                fresh: true,
            },
        }
    }

    /// P71 (BR-M03): a clock that stands still while views keep arriving is the
    /// server's connection pause; a clock that moves again ends it.
    #[test]
    fn p71_a_frozen_clock_with_live_views_reads_as_a_connection_pause() {
        let mut net = link(Mode::Quick);
        net.in_match = true;
        net.note_clock(40, 0.0);
        net.note_clock(40, 1.2);
        assert!(net.match_paused(1.3));
        assert!(
            !net.match_paused(2.0),
            "no views for 0.8 s: not a pause signal"
        );
        net.note_clock(41, 2.1);
        assert!(!net.match_paused(2.2));
    }

    /// P71: a socket that stops delivering is dropped and reconnected even
    /// though it never reported `Closed` (a lost network rarely does).
    #[test]
    fn p71_a_silent_socket_is_treated_as_lost() {
        let mut net = link(Mode::Quick);
        net.opened = true;
        net.heard_at = Some(10.0);
        net.read_socket(13.9);
        assert_eq!(
            net.status,
            NetStatus::Ready,
            "one missed second is not a loss"
        );
        net.read_socket(14.1);
        assert!(!net.opened);
        assert!(matches!(net.status, NetStatus::Reconnecting { since } if since == 14.1));
    }

    #[test]
    fn p66_updates_wait_for_restore_login_join_and_reconnect() {
        let mut net = link(Mode::Probe);
        assert!(net.safe_for_update());
        net.restore_pending = true;
        assert!(!net.safe_for_update());
        net.restore_pending = false;
        net.login_in_flight = true;
        assert!(!net.safe_for_update());
        net.login_in_flight = false;
        net.mode = Mode::Quick;
        assert!(!net.safe_for_update());
        net.mode = Mode::Probe;
        net.status = NetStatus::Reconnecting { since: 0. };
        assert!(!net.safe_for_update());
        net.status = NetStatus::Ready;
        net.in_match = true;
        assert!(!net.safe_for_update());
    }

    #[test]
    fn an_expired_queue_request_waits_for_authorization_and_is_reissued_after_renewal() {
        let mut net = link(Mode::Quick);
        net.handle(
            ServerMsg::AuthRequired {
                reason: protocol::AuthRefusal::Expired,
            },
            0.,
        );
        assert!(net.held_request && net.login_in_flight);
        assert!(!net
            .outq
            .iter()
            .any(|(_, m)| matches!(m, ClientMsg::QuickMatch)));
        net.handle(
            ServerMsg::AuthState {
                status: protocol::AuthStatus::Active,
                access_expires_at: 1900,
                session_expires_at: 2_593_000,
            },
            1.,
        );
        assert!(net
            .outq
            .iter()
            .any(|(_, m)| matches!(m, ClientMsg::QuickMatch)));
    }

    #[test]
    fn acknowledged_challenge_reconnects_without_recreating_the_invite() {
        // BR-R08: reconnect must preserve the link already shared.
        let mut net = link(Mode::CreateChallenge);
        net.handle(ServerMsg::ChallengeWaiting { token: [7; 16] }, 0.0);
        net.lost(1.0);
        net.handle(
            ServerMsg::Welcome {
                token: [1; 16],
                name: "A".into(),
            },
            2.0,
        );
        assert_eq!(net.mode, Mode::Probe);
        assert!(net.outq.is_empty(), "Hello alone resumes the invitation");
    }

    #[test]
    fn challenge_result_reconnects_without_consuming_the_invite_again() {
        // BR-R09: both host and guest resume the result offer, not a mode.
        for mode in [Mode::CreateChallenge, Mode::JoinChallenge([7; 16])] {
            let mut net = link(mode);
            net.handle(
                ServerMsg::MatchStart {
                    result_id: [63; 16],
                    you: tbg_core::Seat::A,
                    you_name: "A".into(),
                    you_rating: None,
                    opponent: "B".into(),
                    mode: protocol::MatchMode::Challenge,
                    opponent_rating: None,
                    arcanas: [tbg_core::ArcanaPreset::DEFAULT; 2],
                    arena: tbg_core::Arena::Forest,
                },
                0.0,
            );
            net.handle(
                ServerMsg::MatchEnd {
                    result_id: [63; 16],
                    last_tick: 0,
                    cast: None,
                    outcome: tbg_core::Outcome {
                        winner: Some(tbg_core::Seat::B),
                        reason: tbg_core::EndReason::Forfeit,
                    },
                },
                1.0,
            );
            net.handle(
                ServerMsg::RematchOpen {
                    opponent: "B".into(),
                },
                1.0,
            );
            net.lost(2.0);
            net.handle(
                ServerMsg::Welcome {
                    token: [1; 16],
                    name: "A".into(),
                },
                3.0,
            );
            assert_eq!(net.mode, Mode::Probe);
            assert!(net.outq.is_empty());
        }
    }

    #[test]
    fn a_spectator_sends_no_intention_but_can_leave() {
        // BR-LV02: the seat-only messages never leave a spectator's client.
        let mut net = link(Mode::Watch(7));
        net.handle(
            ServerMsg::SpectateStart {
                id: 7,
                mode: protocol::MatchMode::Casual,
                arena: tbg_core::Arena::Forest,
                players: [
                    protocol::LivePlayer {
                        name: "A".into(),
                        rating: None,
                    },
                    protocol::LivePlayer {
                        name: "B".into(),
                        rating: None,
                    },
                ],
                arcanas: [tbg_core::ArcanaPreset::DEFAULT; 2],
            },
            0.0,
        );
        net.send(
            ClientMsg::Intent {
                seq: 1,
                intent: tbg_core::PlayerIntention::Purchase { offer: 0 },
            },
            0.0,
        );
        net.send(ClientMsg::Surrender, 0.0);
        assert!(net.outq.is_empty(), "no seat message leaves a spectator");
        net.send(ClientMsg::StopSpectating, 0.0);
        assert_eq!(net.outq.len(), 1, "leaving the watch is still sent");
    }

    #[test]
    fn a_reconnected_watch_asks_for_the_match_again() {
        // A reload never resumes a watch on the server: the client asks again.
        let mut net = link(Mode::Watch(7));
        net.handle(
            ServerMsg::Welcome {
                token: [1; 16],
                name: "A".into(),
            },
            0.0,
        );
        assert_eq!(
            net.outq.front().map(|(_, m)| m.clone()),
            Some(ClientMsg::Spectate { id: 7 })
        );
    }
    #[test]
    fn celebration_retry_ignores_terminal_replay_without_canceling_the_new_request() {
        for mode in [Mode::Quick, Mode::Ranked] {
            let mut net = link(mode);
            net.discard_completed([63; 16]);
            net.login_in_flight = true;
            net.held_request = true;
            let old = ServerMsg::MatchStart {
                result_id: [63; 16],
                you: tbg_core::Seat::A,
                you_name: "A".into(),
                you_rating: None,
                opponent: "B".into(),
                mode: protocol::MatchMode::Casual,
                opponent_rating: None,
                arcanas: [tbg_core::ArcanaPreset::DEFAULT; 2],
                arena: tbg_core::Arena::Forest,
            };
            assert!(net.handle(old, 0.).is_none());
            assert!(net
                .handle(
                    ServerMsg::MatchEnd {
                        result_id: [63; 16],
                        last_tick: 100,
                        cast: None,
                        outcome: tbg_core::Outcome {
                            winner: Some(tbg_core::Seat::A),
                            reason: tbg_core::EndReason::Forfeit
                        }
                    },
                    0.
                )
                .is_none());
            assert_eq!(net.mode, mode);
            assert!(net.login_in_flight && net.held_request);
            let profile = protocol::Profile {
                name: "A".into(),
                rating: 1000,
                ranked_games: 0,
                wins: 0,
                losses: 0,
                draws: 0,
            };
            net.handle(ServerMsg::LoggedIn { profile }, 1.);
            assert!(
                net.login_in_flight,
                "D-97: profile alone cannot authorize a new match"
            );
            net.handle(
                ServerMsg::AuthState {
                    status: protocol::AuthStatus::Active,
                    access_expires_at: 900,
                    session_expires_at: 2_592_000,
                },
                1.,
            );
            assert!(net.outq.iter().any(|(_, m)| matches!(
                (mode, m),
                (Mode::Quick, ClientMsg::QuickMatch) | (Mode::Ranked, ClientMsg::RankedMatch)
            )));
        }
    }
}
