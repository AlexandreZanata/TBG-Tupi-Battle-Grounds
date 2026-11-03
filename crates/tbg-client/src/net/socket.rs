//! New play follows active admission; existing matches reconnect to their original slot.
//! Socket pump and reconnect backoff: 0.25 s, 0.5 s, 1 s, then every 2 s
//! through the server's pause, reconnect and terminal replay windows. A frame that runs after the
//! tab becomes visible again retries at once when a retry is due. A socket
//! that goes silent is treated as lost (P71): a dropped network often never
//! reports `Closed`, and the seat is only safe if the client comes back.

use ewebsock::{WsEvent, WsMessage};
use tbg_core::protocol::{self, ClientMsg, ServerMsg};

use super::{NetFailure, NetLink, NetStatus};
use crate::config;

/// Ordinary reconnect/replay window (BR-M03); an active seat also has P71's pause budget.
pub(crate) const GIVE_UP: f64 = 30.0;

/// A draining slot refuses new play with `ServerFull`; the request is asked again through admission
/// (which names the promoted slot) at most this often per window before the player is told.
const READMIT_MAX: u32 = 3;
const READMIT_WINDOW: f64 = 30.0;

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
        #[cfg(target_os = "android")]
        if !crate::platform::android::socket_endpoint_allowed(&url) {
            self.lost(now);
            return;
        }
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
        if matches!(self.status, NetStatus::Failed(_)) {
            return;
        }
        if now - lost > self.reconnect_window() {
            self.socket = None;
            self.status = NetStatus::Failed(NetFailure::ConnectionLost);
        } else if self.socket.is_none() && now >= self.next_try {
            self.connect(now);
            self.tries += 1;
            self.next_try = now + delay(self.tries);
        }
    }

    /// BR-M03: keep trying while the server can still resume this seat or replay its result.
    pub(crate) fn reconnect_window(&self) -> f64 {
        if self.in_match {
            // The shared canonical maximum pause is120s, followed by the normal
            // reconnect window and its terminal replay retention. This grants no server grace.
            120.0 + 2.0 * GIVE_UP
        } else {
            GIVE_UP
        }
    }

    /// P66 (D-92): new play was refused because this slot is being drained for a release. Drop the
    /// link to it and ask admission again; the next `Welcome` repeats the same request there.
    fn readmit(&mut self, now: f64) -> bool {
        let asking = !self.in_match
            && !self.watching
            && !matches!(self.mode, super::Mode::Probe | super::Mode::Watch(_))
            && matches!(
                self.route,
                super::admission::Route::Ready { fresh: true, .. }
            );
        if !asking {
            return false;
        }
        if now - self.readmit_at > READMIT_WINDOW {
            self.readmits = 0;
        }
        if self.readmits >= READMIT_MAX {
            return false;
        }
        self.readmits += 1;
        self.readmit_at = now;
        self.socket = None;
        self.opened = false;
        self.heard_at = None;
        self.outq.clear();
        self.route = super::admission::Route::start(None, now);
        self.status = NetStatus::Connecting;
        true
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
            ServerMsg::MatchStart { result_id, .. }
            | ServerMsg::MatchEnd { result_id, .. }
            | ServerMsg::PersonalMatchResult { result_id, .. }
            | ServerMsg::MatchHistoryStatus { result_id, .. }
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
                // A live lobby socket alone does not restore an authoritative match.
                if !self.in_match {
                    self.status = NetStatus::Ready;
                    self.lost_at = None;
                }
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
            ServerMsg::Error {
                code: protocol::ErrorCode::ServerFull,
            } if self.readmit(now) => None,
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
                        self.status = NetStatus::Ready;
                        self.lost_at = None;
                        self.login_in_flight = false;
                        self.held_request = false;
                        self.login_deadline = None;
                    }
                    ServerMsg::MatchEnd { result_id, .. } if self.result_id == Some(result_id) => {
                        self.in_match = false;
                        self.status = NetStatus::Ready;
                        self.lost_at = None;
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
            readmits: 0,
            readmit_at: 0.0,
        }
    }

    fn full() -> ServerMsg {
        ServerMsg::Error {
            code: protocol::ErrorCode::ServerFull,
        }
    }

    /// P66 (D-92): a slot drained for a release refuses new play; the link asks admission again
    /// (the promoted slot answers) and repeats the request there, instead of showing "server full".
    #[test]
    fn p66_a_draining_slots_refusal_is_asked_again_through_admission() {
        let mut net = link(Mode::Quick);
        net.opened = true;
        assert!(net.handle(full(), 5.0).is_none(), "the player is not told");
        assert!(matches!(
            net.route,
            crate::net::admission::Route::Pending { .. }
        ));
        assert_eq!(net.status, NetStatus::Connecting);
        assert!(net.socket.is_none() && !net.opened);
        assert_eq!(
            net.mode,
            Mode::Quick,
            "the same request is repeated after the next Welcome"
        );
    }

    /// Only a link that asked for new play on a slot admission chose is moved; a match, a watch, an
    /// idle probe and a resumed route report the error, and a server that stays full tells the player.
    #[test]
    fn p66_other_refusals_and_repeated_ones_reach_the_player() {
        let mut in_match = link(Mode::Quick);
        in_match.in_match = true;
        let mut resumed = link(Mode::Quick);
        resumed.route = crate::net::admission::Route::Ready {
            url: "ws://127.0.0.1:8080/ws/slot-a".into(),
            endpoint: "/ws/slot-a".into(),
            fresh: false,
        };
        let mut watching = link(Mode::Quick);
        watching.watching = true;
        for mut net in [
            link(Mode::Probe),
            link(Mode::Watch(7)),
            in_match,
            resumed,
            watching,
        ] {
            assert!(matches!(
                net.handle(full(), 5.0),
                Some(ServerMsg::Error { .. })
            ));
        }
        let mut net = link(Mode::Quick);
        for second in 0..READMIT_MAX {
            net.route = crate::net::admission::Route::Ready {
                url: "ws://127.0.0.1:8080/ws/slot-b".into(),
                endpoint: "/ws/slot-b".into(),
                fresh: true,
            };
            assert!(net.handle(full(), 10.0 + f64::from(second)).is_none());
        }
        net.route = crate::net::admission::Route::Ready {
            url: "ws://127.0.0.1:8080/ws/slot-b".into(),
            endpoint: "/ws/slot-b".into(),
            fresh: true,
        };
        assert!(matches!(
            net.handle(full(), 14.0),
            Some(ServerMsg::Error { .. })
        ));
        assert!(
            net.handle(full(), 14.0 + READMIT_WINDOW + 1.0).is_none(),
            "a later window asks again"
        );
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
    fn p71_retries_through_the_authoritative_pause_and_terminal_replay() {
        // BR-M03: the old30s client cutoff stranded a recoverable seat during its120s pause.
        let mut net = link(Mode::Quick);
        net.in_match = true;
        net.lost(0.);
        net.next_try = 1000.;
        for now in [35., 90., 160.] {
            net.reconnect(now);
            assert!(matches!(net.status, NetStatus::Reconnecting { .. }));
        }
        net.reconnect(181.);
        assert_eq!(net.status, NetStatus::Failed(NetFailure::ConnectionLost));

        let mut queue = link(Mode::Quick);
        queue.lost(0.);
        queue.next_try = 1000.;
        queue.reconnect(31.);
        assert_eq!(queue.status, NetStatus::Failed(NetFailure::ConnectionLost));
    }

    #[test]
    fn p71_welcome_requires_match_recovery_before_clearing_the_deadline() {
        // BR-M03: a replacement lobby without the old seat cannot leave a frozen battle marked Ready.
        let mut net = link(Mode::Probe);
        net.in_match = true;
        net.lost(0.);
        net.next_try = 1000.;
        net.handle(
            ServerMsg::Welcome {
                token: [1; 16],
                name: "A".into(),
            },
            35.,
        );
        assert!(matches!(net.status, NetStatus::Reconnecting { since } if since == 0.));
        assert_eq!(net.lost_at, Some(0.));
        net.reconnect(181.);
        assert_eq!(net.status, NetStatus::Failed(NetFailure::ConnectionLost));

        let mut recovered = link(Mode::Probe);
        recovered.in_match = true;
        recovered.lost(0.);
        recovered.handle(
            ServerMsg::Welcome {
                token: [1; 16],
                name: "A".into(),
            },
            35.,
        );
        recovered.handle(
            ServerMsg::MatchStart {
                result_id: [71; 16],
                you: tbg_core::Seat::A,
                you_name: "A".into(),
                you_rating: None,
                opponent: "B".into(),
                opponent_rating: None,
                mode: protocol::MatchMode::Casual,
                arcanas: [tbg_core::ArcanaPreset::DEFAULT; 2],
                arena: tbg_core::Arena::Forest,
            },
            35.1,
        );
        assert_eq!(recovered.status, NetStatus::Ready);
        assert!(recovered.lost_at.is_none());
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
