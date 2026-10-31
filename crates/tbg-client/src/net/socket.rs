//! Socket pump and reconnect backoff: 0.25 s, 0.5 s, 1 s, then every 2 s
//! until `GIVE_UP` (the server window is 30 s). A frame that runs after the
//! tab becomes visible again retries at once when a retry is due.

use ewebsock::{WsEvent, WsMessage};
use tbg_core::protocol::{self, ClientMsg, ServerMsg};

use super::{NetFailure, NetLink, NetStatus};
use crate::config;

/// Give up reconnecting after this long (server window is 30 s).
pub(crate) const GIVE_UP: f64 = 30.0;

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
        self.opened = false;
        self.outq.clear();
        match ewebsock::connect(config::server_url(), ewebsock::Options::default()) {
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
                    let hello = ClientMsg::Hello {
                        version: super::PROTOCOL_VERSION,
                        resume: self.token,
                    };
                    self.outq.push_front((now + self.lag, hello));
                }
                WsEvent::Message(WsMessage::Binary(bytes)) => {
                    if let Ok(msg) = protocol::decode_server(&bytes) {
                        self.inq.push_back((now + self.lag, msg));
                    }
                }
                WsEvent::Message(_) => {}
                WsEvent::Closed | WsEvent::Error(_) => self.lost(now),
            }
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

    /// Protocol housekeeping stays here; everything else goes to the game.
    pub(super) fn handle(&mut self, msg: ServerMsg, now: f64) -> Option<ServerMsg> {
        match msg {
            ServerMsg::Welcome { token, .. } => {
                let previous = self.token;
                config::save_token(&token);
                self.token = Some(token);
                self.status = NetStatus::Ready;
                self.lost_at = None;
                // Outside a match (first connect, or dropped while queued,
                // which takes us out of the queue) ask for the mode again.
                // A probe only listens: it resumes if a match finds it.
                // Post-P14 (BR-P03): a saved key logs in before the mode
                // request; a resumed session already carries its account.
                let key = (!self.in_match).then(config::load_account_key).flatten();
                let retry = self.login_in_flight;
                if key.is_some()
                    && (retry || super::needs_login(previous, token, true, self.in_match))
                {
                    if let Some(key) = key {
                        self.send(ClientMsg::Login { key }, now);
                    }
                    self.login_in_flight = true;
                    self.login_deadline = Some(now + super::LOGIN_TIMEOUT);
                    self.held_request |= !matches!(self.mode, super::Mode::Probe);
                } else if !self.in_match && !matches!(self.mode, super::Mode::Probe) {
                    self.login_in_flight = false;
                    self.send_request(now);
                }
                None
            }
            ServerMsg::LoggedIn { .. } | ServerMsg::AccountError { .. } => {
                self.login_in_flight = false;
                self.login_deadline = None;
                if !self.in_match {
                    self.release_request(now);
                }
                Some(msg)
            }
            ServerMsg::Ping { id } => {
                self.send(ClientMsg::Pong { id }, now);
                None
            }
            ServerMsg::VersionMismatch { .. } => {
                self.status = NetStatus::Failed(NetFailure::VersionMismatch);
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
                    ServerMsg::MatchStart { .. } => {
                        self.mode = super::Mode::Probe;
                        self.in_match = true;
                        self.login_in_flight = false;
                        self.held_request = false;
                        self.login_deadline = None;
                    }
                    ServerMsg::MatchEnd { .. } => self.in_match = false,
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
            mode,
            socket: None,
            opened: false,
            token: Some([1; 16]),
            in_match: false,
            status: NetStatus::Ready,
            lost_at: None,
            tries: 0,
            next_try: 0.0,
            lag: 0.0,
            ack: Default::default(),
            login_in_flight: false,
            held_request: false,
            login_deadline: None,
            outq: Default::default(),
            inq: Default::default(),
            watching: false,
        }
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
}
