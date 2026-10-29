//! Both-accept rematch tickets, requests, decline, and expiry (BR-R09).

use std::sync::atomic::Ordering::Relaxed;
use std::time::Instant;

use tbg_core::protocol::{MatchMode, ServerMsg, Token};

use super::{Lobby, RematchTicket, SessionState};

impl Lobby {
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
        if self.rematches[i]
            .expires
            .is_some_and(|e| Instant::now() >= e)
        {
            let ticket = self.rematches.remove(i);
            self.expire_ticket(ticket);
            return;
        }
        if self.draining.load(Relaxed) || self.rematches[i].requested != [true, true] {
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
}
