//! Session resume, disconnects, and periodic cleanup.

use std::net::IpAddr;
use std::time::{Duration, Instant};

use tbg_core::protocol::{InviteToken, ServerMsg, Token};

use super::{send, Lobby, Session, SessionState, ROOM_TTL};
use crate::chan::{ConnEvent, ConnHandle, MatchCmd};

impl Lobby {
    /// Resume a known session (taking over any older socket) or create one.
    pub(super) fn hello(
        &mut self,
        resume: Option<Token>,
        conn: ConnHandle,
        ip: IpAddr,
    ) -> Option<(Token, String)> {
        let reserved = resume.is_some_and(|t| {
            self.sessions
                .get(&t)
                .is_some_and(|s| s.account.is_some() || s.auth_binding.is_some())
        });
        let resume = if reserved {
            let _ = conn.tx.try_send(ConnEvent::Send(ServerMsg::AuthRequired {
                reason: tbg_core::protocol::AuthRefusal::Invalid,
            }));
            None
        } else {
            resume
        };
        self.hello_bound(resume, conn, ip)
    }

    pub(super) fn hello_bound(
        &mut self,
        resume: Option<Token>,
        conn: ConnHandle,
        ip: IpAddr,
    ) -> Option<(Token, String)> {
        // A reload does not resume a watch: the new socket starts `Idle` (BR-LV02).
        if let Some(t) = resume {
            self.end_watch(t);
        }
        if let Some(s) = resume.and_then(|t| self.sessions.get_mut(&t)) {
            if let Some(old) = &s.conn {
                old.active
                    .store(false, std::sync::atomic::Ordering::Release);
            }
            if s.account.is_some() {
                conn.active
                    .store(true, std::sync::atomic::Ordering::Release);
            }
            send(&s.conn, ConnEvent::Close);
            if let SessionState::InMatch { tx, seat } = &s.state {
                let _ = tx.try_send(MatchCmd::Attach {
                    seat: *seat,
                    conn: conn.clone(),
                    new_token: None,
                });
                let _ = conn.tx.try_send(ConnEvent::EnterMatch {
                    seat: *seat,
                    tx: tx.clone(),
                });
            }
            // P66: a resumed signed-in session claims its account again.
            let reclaim = s.account.as_ref().map(|(id, _)| (s.token, *id));
            // P14: a resumed session that holds an account tells the new
            // socket (a reload keeps the token but not the client's profile).
            if let Some((_, profile)) = &s.account {
                let msg = ServerMsg::LoggedIn {
                    profile: profile.clone(),
                };
                let _ = conn.tx.try_send(ConnEvent::Send(msg));
            }
            let token = s.token;
            let name = s.name.clone();
            let challenge = match s.state {
                SessionState::InChallenge { token } => Some(token),
                _ => None,
            };
            let idle = matches!(s.state, SessionState::Idle);
            if idle {
                if let Some((until, result)) = &s.finished {
                    if Instant::now() < *until {
                        send(&Some(conn.clone()), ConnEvent::Send(result.start.clone()));
                        send(&Some(conn.clone()), ConnEvent::Send(result.end.clone()));
                    }
                }
            }
            s.conn = Some(conn);
            s.ip = ip;
            s.detached_at = None;
            if let Some(invite) = challenge {
                if self
                    .challenges
                    .iter()
                    .any(|(t, owner, _)| *t == invite && *owner == token)
                {
                    self.send_to(token, ServerMsg::ChallengeWaiting { token: invite });
                } else if let Some(s) = self.session(&token) {
                    if matches!(s.state, SessionState::InChallenge { token } if token == invite) {
                        s.state = SessionState::Idle;
                    }
                }
            } else if idle {
                self.resume_rematch(token);
            }
            if let Some((t, id)) = reclaim {
                self.reclaim(t, id);
            }
            return Some((token, name));
        }
        let (token, name) = crate::names::session_identity()?;
        let session = Session {
            auth_binding: None,
            token,
            name,
            conn: Some(conn),
            state: SessionState::Idle,
            account: None,
            ip,
            detached_at: None,
            arcanas: tbg_core::ArcanaPreset::DEFAULT,
            stronghold: tbg_core::StrongholdKind::DEFAULT,
            last_spectate: None,
            finished: None,
        };
        let pair = (session.token, session.name.clone());
        self.sessions.insert(session.token, session);
        Some(pair)
    }

    pub(super) fn disconnected(&mut self, token: Token, conn_id: u64) {
        let Some(s) = self.session(&token) else {
            return;
        };
        if s.conn.as_ref().is_none_or(|c| c.id != conn_id) {
            return;
        }
        // While the socket is still attached, so the match drops this conn.
        self.end_watch(token);
        let Some(s) = self.session(&token) else {
            return;
        };
        s.conn = None;
        s.detached_at = Some(Instant::now());
        // P66: an idle signed-in session frees its account for other slots;
        // a resume re-claims it. A seat in a match keeps it.
        let idle_account = matches!(s.state, SessionState::Idle)
            .then(|| s.account.as_ref().map(|(id, _)| *id))
            .flatten();
        self.leave_queue_room(token);
        if let Some(id) = idle_account {
            self.release_owner(id);
        }
    }

    /// A reload on the result screen recovers the open rematch offer.
    fn resume_rematch(&mut self, token: Token) {
        let Some(i) = self
            .rematches
            .iter()
            .position(|r| r.expires.is_some() && r.players.contains(&token))
        else {
            return;
        };
        if self.rematches[i]
            .expires
            .is_some_and(|e| Instant::now() >= e)
            || self.rematches[i]
                .players
                .iter()
                .any(|p| !self.sessions.contains_key(p))
        {
            let ticket = self.rematches.remove(i);
            self.expire_ticket(ticket);
            return;
        }
        let ticket = &self.rematches[i];
        let seat = usize::from(ticket.players[1] == token);
        let other = ticket.players[1 - seat];
        let opponent = self
            .sessions
            .get(&other)
            .map_or_else(String::new, |s| s.name.clone());
        let you_requested = ticket.requested[seat];
        let opponent_requested = ticket.requested[1 - seat];
        let players = ticket.players;
        self.send_to(token, ServerMsg::RematchOpen { opponent });
        self.send_to(
            token,
            ServerMsg::RematchState {
                you_requested,
                opponent_requested,
            },
        );
        self.try_start_rematch(players);
    }

    /// Drop expired rooms, invites, rematches, and long-gone sessions.
    pub(super) fn cleanup(&mut self) {
        let now = Instant::now();
        self.limits.sweep(now);
        self.rooms.retain(|(.., created)| now - *created < ROOM_TTL);
        let window = Duration::from_secs(self.config.reconnect_secs);
        let mut released = Vec::new();
        self.sessions.retain(|_, s| {
            if s.finished.as_ref().is_some_and(|(until, _)| now >= *until) {
                s.finished = None;
            }
            let gone = s.detached_at.is_some_and(|t| now - t > window);
            let keep = !gone || matches!(s.state, SessionState::InMatch { .. });
            if !keep {
                released.extend(s.account.as_ref().map(|(id, _)| *id));
            }
            keep
        });
        for id in released {
            self.release_owner(id);
        }
        self.queue.retain(|t| self.sessions.contains_key(t));
        self.ranked_queue.retain(|t| self.sessions.contains_key(t));
        self.queued_at.retain(|t, _| self.queue.contains(t));
        self.ranked_queued_at
            .retain(|t, _| self.ranked_queue.contains(t));

        let ttl = Duration::from_secs(self.config.challenge_ttl_secs);
        let expired: Vec<(InviteToken, Token)> = self
            .challenges
            .iter()
            .filter(|(_, owner, created)| {
                now.saturating_duration_since(*created) >= ttl || !self.sessions.contains_key(owner)
            })
            .map(|(invite, owner, _)| (*invite, *owner))
            .collect();
        self.challenges.retain(|(_, owner, created)| {
            now.saturating_duration_since(*created) < ttl && self.sessions.contains_key(owner)
        });
        for (invite, owner) in expired {
            self.expire_challenge(owner, invite);
        }

        let mut closed = Vec::new();
        self.rematches.retain(|ticket| {
            let expired = ticket.expires.is_some_and(|e| now >= e);
            let gone = ticket
                .players
                .iter()
                .any(|p| !self.sessions.contains_key(p));
            if expired {
                closed.extend(ticket.players);
                false
            } else if gone {
                closed.extend(
                    ticket
                        .players
                        .iter()
                        .copied()
                        .filter(|p| self.sessions.contains_key(p)),
                );
                false
            } else {
                true
            }
        });
        for token in closed {
            self.send_to(token, ServerMsg::RematchClosed { declined: false });
        }
    }
}
