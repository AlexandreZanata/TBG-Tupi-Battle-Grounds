//! Quick-match queue, private rooms, and pairing two sessions into a match.

use std::time::Instant;

use tbg_core::protocol::{valid_room_code, MatchMode, RoomCode, ServerMsg, Token, ROOM_ALPHABET};
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
