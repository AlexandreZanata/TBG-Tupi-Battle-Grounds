//! Quick-match queue, private rooms, and pairing two sessions into a match.

use std::time::Instant;

use tbg_core::protocol::{
    valid_room_code, LivePlayer, MatchMode, RoomCode, ServerMsg, Token, ROOM_ALPHABET,
};
use tbg_core::Seat;

use super::live::{Listing, LiveMatch};
use super::{send, Lobby, RematchTicket, SessionState};
use crate::chan::ConnEvent;
use crate::live::LiveProgress;
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
                self.pair([other, token], MatchMode::Casual, false, Listing::Public);
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
                // Private rooms are always casual (BR-R01), and never listed (BR-LV01).
                self.pair([owner, token], MatchMode::Casual, false, Listing::Private);
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
        if let Some(s) = self.session(&token) {
            s.finished = None;
        }
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
        self.ranked_queued_at.remove(&token);
        self.rooms.retain(|(_, owner, _)| *owner != token);
        if let Some(s) = self.session(&token) {
            if matches!(s.state, SessionState::Queued | SessionState::InRoom) {
                s.state = SessionState::Idle;
            }
        }
    }

    /// Start a match. `listing` decides visibility (BR-LV01): only the public
    /// queues pass `Listing::Public`; never infer it from `mode`.
    pub(super) fn pair(
        &mut self,
        tokens: [Token; 2],
        mode: MatchMode,
        rematchable: bool,
        listing: Listing,
    ) {
        // P9: queue wait per paired session (the joiner waited ~0 ms).
        for t in tokens {
            if let Some(t0) = self.queued_at.remove(&t) {
                self.metrics
                    .record_queue_wait(t0.elapsed().as_millis().min(u128::from(u64::MAX)) as u64);
            }
            // P60: ranked waits are measured separately (no await, no SQL).
            if let Some(t0) = self.ranked_queued_at.remove(&t) {
                self.metrics.record_ranked_queue_wait(
                    t0.elapsed().as_millis().min(u128::from(u64::MAX)) as u64,
                );
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
                // D-46 (BR-SK01): frozen here like the Arcanas.
                stronghold: s.map_or(tbg_core::StrongholdKind::DEFAULT, |s| s.stronghold),
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
        // ARENAS-G (D-55): a fresh draw for every match, rematches
        // included, from the OS RNG: never the seed or a simulation stream.
        let arena = draw_arena();
        let live = (listing == Listing::Public).then(|| self.fresh_live_id());
        let listed = live.map(|_| {
            (
                live_seats(&players),
                players.each_ref().map(|p| p.stronghold),
            )
        });
        let tx = match_task::spawn(
            (seed, arena, live),
            players,
            mode,
            self.self_tx.clone(),
            self.config.clone(),
            self.metrics.clone(),
            match_task::Records {
                store: self.store.clone(),
                ranked: self.ranked.clone(),
                // P60: OS-random, separate from the public LiveId.
                key: random_bytes::<16>(),
                stats: self.config.stats == crate::ranked::policy::StatsMode::Collect,
            },
        );
        if let (Some(id), Some((seats, kinds))) = (live, listed) {
            self.register_live(
                id,
                LiveMatch {
                    tx: tx.clone(),
                    listing,
                    mode,
                    arena,
                    players: seats,
                    kinds,
                    started: Instant::now(),
                    progress: LiveProgress::default(),
                    watchers: Vec::new(),
                },
            );
        }
        for (seat, token) in Seat::BOTH.into_iter().zip(tokens) {
            if let Some(s) = self.session(&token) {
                s.finished = None;
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

/// What a listed match shows per seat: the name, and the rating for an account.
fn live_seats(players: &[Player; 2]) -> [LivePlayer; 2] {
    players.each_ref().map(|p| LivePlayer {
        name: p.name.clone(),
        rating: p.account.map(|(_, rating)| rating),
    })
}

/// Uniform over the five arenas: one OS-random byte, rejecting 255 so the
/// remaining 255 values split evenly (51 each).
pub(crate) fn draw_arena() -> tbg_core::Arena {
    loop {
        let [byte] = random_bytes::<1>();
        if byte < 255 {
            return tbg_core::Arena::ALL[usize::from(byte) % tbg_core::Arena::ALL.len()];
        }
    }
}

#[cfg(test)]
mod arena_tests {
    #[test]
    fn the_draw_is_uniform_over_the_five_arenas() {
        // 50 000 draws: 10 000 expected each, σ ≈ 89; ±6 % is ~6.7 σ.
        let mut counts = [0u32; 5];
        for _ in 0..50_000 {
            counts[super::draw_arena().index()] += 1;
        }
        for (i, n) in counts.iter().enumerate() {
            assert!((9_400..=10_600).contains(n), "arena {i}: {n} of 50 000");
        }
    }
}
