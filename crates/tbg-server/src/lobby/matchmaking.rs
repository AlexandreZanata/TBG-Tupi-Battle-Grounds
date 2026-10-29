//! Quick-match queue, private rooms, and pairing two sessions into a match.

use std::time::{Duration, Instant};

use tbg_core::protocol::{
    valid_room_code, ErrorCode, InviteToken, MatchMode, RoomCode, ServerMsg, Token, ROOM_ALPHABET,
};
use tbg_core::Seat;

use super::{send, Lobby, RematchTicket, SessionState};
use crate::chan::ConnEvent;
use crate::match_task::{self, Player};
use crate::names::random_bytes;

impl Lobby {
    pub(super) fn quick(&mut self, token: Token) {
        self.leave_waiting(token);
        // The token just left the queue above, so the front (if any) is
        // the longest-waiting other session.
        match self.queue.pop_front() {
            Some(other) => {
                self.queued_at.remove(&other);
                self.pair([other, token], MatchMode::Casual, false);
            }
            None => {
                self.queue.push_back(token);
                self.queued_at.insert(token, Instant::now());
                if let Some(s) = self.session(&token) {
                    s.state = SessionState::Queued;
                    send(&s.conn, ConnEvent::Send(ServerMsg::Queued));
                }
            }
        }
    }

    pub(super) fn create_room(&mut self, token: Token) {
        self.leave_waiting(token);
        let code = loop {
            let code =
                random_bytes::<5>().map(|b| ROOM_ALPHABET[usize::from(b) % ROOM_ALPHABET.len()]);
            if !self.rooms.iter().any(|(c, ..)| *c == code) {
                break code;
            }
        };
        self.rooms.push((code, token, Instant::now()));
        if let Some(s) = self.session(&token) {
            s.state = SessionState::InRoom;
            send(&s.conn, ConnEvent::Send(ServerMsg::RoomCreated { code }));
        }
    }

    pub(super) fn join_room(&mut self, token: Token, code: RoomCode) {
        let found = valid_room_code(&code)
            .then(|| {
                self.rooms
                    .iter()
                    .position(|(c, owner, _)| *c == code && *owner != token)
            })
            .flatten();
        match found {
            Some(i) => {
                let (_, owner, _) = self.rooms.remove(i);
                self.leave_waiting(token);
                // Private rooms are always casual (BR-R01).
                self.pair([owner, token], MatchMode::Casual, false);
            }
            None => {
                if let Some(s) = self.session(&token) {
                    send(&s.conn, ConnEvent::Send(ServerMsg::RoomNotFound));
                }
            }
        }
    }

    /// Leave queue, owned room, owned challenge, and any open rematch.
    pub(super) fn leave_waiting(&mut self, token: Token) {
        self.leave_queue_room(token);
        self.challenges.retain(|(_, owner, _)| *owner != token);
        if let Some(s) = self.session(&token) {
            if matches!(s.state, SessionState::InChallenge { .. }) {
                s.state = SessionState::Idle;
            }
        }
        self.decline_rematch(token);
    }

    /// Disconnect cleanup: queue and owned room only. A challenge or rematch
    /// survives so a reload can resume it.
    pub(super) fn leave_queue_room(&mut self, token: Token) {
        self.queue.retain(|t| *t != token);
        // P12: ranked and casual queues never mix, but one cancel leaves both.
        self.ranked_queue.retain(|t| *t != token);
        self.queued_at.remove(&token);
        self.rooms.retain(|(_, owner, _)| *owner != token);
        if let Some(s) = self.session(&token) {
            if matches!(s.state, SessionState::Queued | SessionState::InRoom) {
                s.state = SessionState::Idle;
            }
        }
    }

    /// P39: create a single-use challenge invitation and wait for a guest.
    pub(super) fn create_challenge(&mut self, token: Token) {
        self.leave_waiting(token);
        let invite = loop {
            let invite = random_bytes::<16>();
            if !self.challenges.iter().any(|(t, ..)| *t == invite) {
                break invite;
            }
        };
        self.challenges.push((invite, token, Instant::now()));
        if let Some(s) = self.session(&token) {
            s.state = SessionState::InChallenge { token: invite };
            send(
                &s.conn,
                ConnEvent::Send(ServerMsg::ChallengeWaiting { token: invite }),
            );
        }
    }

    /// P39: consume an invite and pair the owner with one different session.
    pub(super) fn join_challenge(&mut self, token: Token, invite: InviteToken) {
        let Some(ip) = self.sessions.get(&token).map(|s| s.ip) else {
            return;
        };
        if !self
            .limits
            .check_invite(ip, self.config.invite_per_ip_min as u32, Instant::now())
        {
            self.send_to(
                token,
                ServerMsg::Error {
                    code: ErrorCode::RateLimited,
                },
            );
            return;
        }
        let now = Instant::now();
        let ttl = Duration::from_secs(self.config.challenge_ttl_secs);
        let Some(i) = self
            .challenges
            .iter()
            .position(|(t, owner, _)| *t == invite && *owner != token)
        else {
            self.send_to(token, ServerMsg::ChallengeUnavailable);
            return;
        };
        let (_, owner, created) = self.challenges[i];
        if now.saturating_duration_since(created) >= ttl {
            self.challenges.remove(i);
            self.expire_challenge(owner, invite);
            self.send_to(token, ServerMsg::ChallengeUnavailable);
            return;
        }
        let owner_ready = self.sessions.get(&owner).is_some_and(|s| {
            s.conn.is_some()
                && matches!(s.state, SessionState::InChallenge { token: t } if t == invite)
        });
        if !owner_ready {
            self.send_to(token, ServerMsg::ChallengeUnavailable);
            return;
        }
        self.challenges.remove(i);
        self.leave_waiting(token);
        self.pair([owner, token], MatchMode::Challenge, true);
    }

    /// P39: an explicit Menu/Cancel or a new match declines the open offer.
    pub(super) fn decline_rematch(&mut self, token: Token) {
        let Some(i) = self
            .rematches
            .iter()
            .position(|r| r.expires.is_some() && r.players.contains(&token))
        else {
            return;
        };
        let ticket = self.rematches.remove(i);
        let other = if ticket.players[0] == token {
            ticket.players[1]
        } else {
            ticket.players[0]
        };
        self.send_to(other, ServerMsg::RematchClosed { declined: true });
    }

    /// P39: request or accept a rematch; starts only with both requests and
    /// both connections.
    pub(super) fn request_rematch(&mut self, token: Token) {
        let now = Instant::now();
        let Some(i) = self
            .rematches
            .iter()
            .position(|r| r.expires.is_some() && r.players.contains(&token))
        else {
            self.send_to(token, ServerMsg::RematchClosed { declined: false });
            return;
        };
        if self.rematches[i].expires.is_some_and(|e| now >= e) {
            let ticket = self.rematches.remove(i);
            self.expire_ticket(ticket);
            return;
        }
        let players = self.rematches[i].players;
        let ready = players.iter().all(|p| {
            self.sessions
                .get(p)
                .is_some_and(|s| matches!(s.state, SessionState::Idle))
        });
        if !ready {
            self.rematches.remove(i);
            self.send_to(token, ServerMsg::RematchClosed { declined: true });
            return;
        }
        let seat = usize::from(players[1] == token);
        self.rematches[i].requested[seat] = true;
        self.send_rematch_state(i);
        self.try_start_rematch(players);
    }

    /// P39: pair the ticket's sessions when both requested and connected.
    pub(super) fn try_start_rematch(&mut self, players: [Token; 2]) {
        let Some(i) = self
            .rematches
            .iter()
            .position(|r| r.players == players && r.expires.is_some())
        else {
            return;
        };
        if self.rematches[i].requested != [true, true] {
            return;
        }
        let connected = players.iter().all(|p| {
            self.sessions
                .get(p)
                .is_some_and(|s| s.conn.is_some() && matches!(s.state, SessionState::Idle))
        });
        if !connected {
            return;
        }
        let ticket = self.rematches.remove(i);
        self.pair(ticket.players, MatchMode::Challenge, true);
    }

    /// P39 (BR-P03): a login takeover moves a seat to a new session token.
    pub(super) fn replace_rematch_player(&mut self, old: Token, new: Token) {
        for ticket in &mut self.rematches {
            for player in &mut ticket.players {
                if *player == old {
                    *player = new;
                }
            }
        }
    }

    pub(super) fn expire_challenge(&mut self, owner: Token, invite: InviteToken) {
        if let Some(s) = self.session(&owner) {
            if matches!(s.state, SessionState::InChallenge { token } if token == invite) {
                s.state = SessionState::Idle;
                send(&s.conn, ConnEvent::Send(ServerMsg::ChallengeUnavailable));
            }
        }
    }

    pub(super) fn expire_ticket(&mut self, ticket: RematchTicket) {
        for token in ticket.players {
            self.send_to(token, ServerMsg::RematchClosed { declined: false });
        }
    }

    fn send_rematch_state(&mut self, i: usize) {
        let (players, requested) = {
            let ticket = &self.rematches[i];
            (ticket.players, ticket.requested)
        };
        for (seat, token) in players.iter().enumerate() {
            let other = 1 - seat;
            self.send_to(
                *token,
                ServerMsg::RematchState {
                    you_requested: requested[seat],
                    opponent_requested: requested[other],
                },
            );
        }
    }

    pub(super) fn send_to(&mut self, token: Token, msg: ServerMsg) {
        if let Some(s) = self.session(&token) {
            send(&s.conn, ConnEvent::Send(msg));
        }
    }

    pub(super) fn pair(&mut self, tokens: [Token; 2], mode: MatchMode, rematchable: bool) {
        // P9: queue wait per paired session (the joiner waited ~0 ms).
        for t in tokens {
            if let Some(t0) = self.queued_at.remove(&t) {
                self.metrics
                    .record_queue_wait(t0.elapsed().as_millis().min(u128::from(u64::MAX)) as u64);
            }
        }
        let players = tokens.map(|t| {
            let s = self.sessions.get(&t);
            Player {
                token: t,
                name: s.map_or_else(String::new, |s| s.name.clone()),
                conn: s.and_then(|s| s.conn.clone()),
                // P12: account id + rating at pairing time for ranked rows.
                account: s.and_then(|s| s.account.as_ref().map(|(id, p)| (*id, p.rating))),
                // D-40 (BR-AR05): frozen here; later edits stay in the lobby.
                arcanas: s.map_or(tbg_core::ArcanaPreset::DEFAULT, |s| s.arcanas),
            }
        });
        if rematchable {
            self.rematches.retain(|r| r.players != tokens);
            self.rematches.push(RematchTicket {
                players: tokens,
                requested: [false; 2],
                expires: None,
            });
        }
        let seed = u64::from_le_bytes(random_bytes::<8>());
        let tx = match_task::spawn(
            seed,
            players,
            mode,
            self.self_tx.clone(),
            self.config.clone(),
            self.metrics.clone(),
            self.store.clone(),
        );
        for (seat, token) in Seat::BOTH.into_iter().zip(tokens) {
            if let Some(s) = self.session(&token) {
                s.state = SessionState::InMatch {
                    tx: tx.clone(),
                    seat,
                };
                send(
                    &s.conn,
                    ConnEvent::EnterMatch {
                        seat,
                        tx: tx.clone(),
                    },
                );
            }
        }
    }
}
