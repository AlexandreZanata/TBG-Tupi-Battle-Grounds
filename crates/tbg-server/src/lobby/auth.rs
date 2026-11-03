//! Family-scoped revocation barriers use ordinary detach and never fabricate forfeits.

use super::*;
use crate::auth::Binding;
use crate::store::auth_sessions::FamilyId;
use tbg_core::protocol::{AuthRefusal, AuthStatus};

/// Silence after which a same-session resume replaces the attached socket;
/// below the client's own 4 s watchdog, so its first reconnect succeeds.
const STALE_LINK: Duration = Duration::from_secs(3);

impl Lobby {
    pub(super) fn auth_can_start(&self, token: Token) -> bool {
        let Some(session) = self.sessions.get(&token) else {
            return false;
        };
        if session
            .conn
            .as_ref()
            .is_some_and(|c| !c.active.load(std::sync::atomic::Ordering::Acquire))
        {
            return false;
        }
        match &session.auth_binding {
            Some(binding) => {
                if self
                    .config
                    .auth_clock
                    .now()
                    .is_ok_and(|now| self.config.moderation.banned(binding.account, now))
                {
                    return false;
                }
                session
                    .account
                    .as_ref()
                    .is_some_and(|(id, _)| *id == binding.account)
                    && self
                        .config
                        .auth_clock
                        .now()
                        .is_ok_and(|now| binding.valid(now))
            }
            None => session.account.is_none(),
        }
    }

    pub(super) fn refuse_expired_pair(
        &mut self,
        tokens: [Token; 2],
        mode: tbg_core::protocol::MatchMode,
        listing: super::live::Listing,
    ) -> bool {
        let valid = tokens.map(|token| self.auth_can_start(token));
        if valid.iter().all(|valid| *valid) {
            return false;
        }
        // Preserve the still-authorized peer's FIFO position. Private invitations
        // must never silently become a public queue after a peer expires.
        for (token, valid) in tokens.into_iter().zip(valid).rev() {
            let queued = self
                .queued_at
                .get(&token)
                .copied()
                .unwrap_or_else(Instant::now);
            let ranked = self
                .ranked_queued_at
                .get(&token)
                .copied()
                .unwrap_or_else(Instant::now);
            self.leave_waiting(token);
            if !valid {
                self.send_to(
                    token,
                    ServerMsg::AuthRequired {
                        reason: AuthRefusal::Expired,
                    },
                );
                continue;
            }
            if listing == super::live::Listing::Public {
                if mode == tbg_core::protocol::MatchMode::Ranked {
                    self.ranked_queue.push_front(token);
                    self.ranked_queued_at.insert(token, ranked);
                } else {
                    self.queue.push_front(token);
                    self.queued_at.insert(token, queued);
                }
                if let Some(s) = self.sessions.get_mut(&token) {
                    s.state = SessionState::Queued;
                }
                self.send_to(token, ServerMsg::Queued);
            } else {
                let message = if mode == tbg_core::protocol::MatchMode::Challenge {
                    ServerMsg::ChallengeUnavailable
                } else {
                    ServerMsg::RoomNotFound
                };
                self.send_to(token, message);
            }
        }
        true
    }

    pub(super) fn auth_hello(
        &mut self,
        resume: Option<Token>,
        conn: ConnHandle,
        ip: IpAddr,
        binding: Binding,
        lease: bool,
    ) -> Option<(Token, String)> {
        let now = self.config.auth_clock.now().ok()?;
        if self.config.moderation.banned(binding.account, now) {
            return None;
        }
        if !binding.valid(now) && !(lease && self.lease_resumes(resume, &binding)) {
            return None;
        }
        if let Some(session) = resume.and_then(|token| self.sessions.get(&token)) {
            if let Some(old) = &session.auth_binding {
                if old.family != binding.family || old.account != binding.account {
                    return None;
                }
            } else if session.account.is_some()
                || matches!(session.state, SessionState::InMatch { .. })
            {
                return None;
            }
        }
        // P71: the match holds a seat until it ends it (window, forfeit); an idle
        // session past the window starts fresh instead of forcing a new sign-in.
        let window = Duration::from_secs(self.config.reconnect_secs);
        let resume = resume.filter(|token| {
            self.sessions.get(token).is_none_or(|s| {
                matches!(s.state, SessionState::InMatch { .. })
                    || s.detached_at.is_none_or(|at| at.elapsed() <= window)
            })
        });
        // Passive restore cannot displace a still-attached controller, even with
        // a copied same-family resume handle. A detached reconnect is distinct,
        // and so is a controller the server no longer hears (P71): it is pinged
        // every second, so this much silence is a dead link TCP has not reported.
        let resume = resume.filter(|token| {
            self.sessions.get(token).is_none_or(|s| {
                s.conn.as_ref().is_none_or(|c| {
                    !c.active.load(std::sync::atomic::Ordering::Acquire)
                        || c.link.silent_for() > STALE_LINK
                })
            })
        });
        // A lease only ever takes its own seat back, never opens a new session.
        if lease && resume.is_none() {
            return None;
        }
        conn.active
            .store(false, std::sync::atomic::Ordering::Release);
        let id = conn.id;
        let pair = self.hello_bound(resume, conn, ip)?;
        // A lease resume renews on the socket (AuthRequired{Expired}), then claims.
        if !lease {
            self.auth_claim(pair.0, id, binding, false);
        }
        Some(pair)
    }

    /// P71 (BR-AS07): an expired Access may only take back its own admitted seat.
    fn lease_resumes(&self, resume: Option<Token>, binding: &Binding) -> bool {
        resume
            .and_then(|token| self.sessions.get(&token))
            .is_some_and(|s| {
                matches!(s.state, SessionState::InMatch { .. })
                    && s.auth_binding.as_ref().is_some_and(|old| {
                        old.family == binding.family && old.account == binding.account
                    })
            })
    }

    pub(super) fn auth_claim(
        &mut self,
        token: Token,
        conn_id: u64,
        binding: Binding,
        take_over: bool,
    ) {
        let valid = self
            .config
            .auth_clock
            .now()
            .is_ok_and(|now| binding.valid(now));
        let Some(session) = self.sessions.get(&token) else {
            return;
        };
        if session.conn.as_ref().is_none_or(|conn| conn.id != conn_id) {
            return;
        }
        let wrong = session
            .auth_binding
            .as_ref()
            .is_some_and(|old| old.account != binding.account || old.family != binding.family);
        if !valid
            || wrong
            || self
                .config
                .auth_clock
                .now()
                .is_ok_and(|now| self.config.moderation.banned(binding.account, now))
        {
            self.send_to(
                token,
                ServerMsg::AuthRequired {
                    reason: AuthRefusal::Invalid,
                },
            );
            return;
        }
        let already = session
            .account
            .as_ref()
            .is_some_and(|(id, _)| *id == binding.account);
        if !already && !matches!(session.state, SessionState::Idle) {
            self.send_to(
                token,
                ServerMsg::AuthRequired {
                    reason: AuthRefusal::Invalid,
                },
            );
            return;
        }
        let conn = session.conn.clone();
        // BR-AP03/07: consent was given to the previous identity (often a
        // guest); a sign-in into another account must not inherit it.
        if take_over || !already {
            self.cancel_direct(token);
        }
        if !already {
            self.decline_rematch(token);
        }
        if let Some(session) = self.sessions.get_mut(&token) {
            session.auth_binding = Some(binding.clone());
        }
        let conflict = self.sessions.iter().any(|(other, s)| {
            *other != token
                && s.account
                    .as_ref()
                    .is_some_and(|(id, _)| *id == binding.account)
                && (s
                    .conn
                    .as_ref()
                    .is_some_and(|c| c.active.load(std::sync::atomic::Ordering::Acquire))
                    || matches!(s.state, SessionState::InMatch { .. }))
        });
        if conflict && !take_over {
            if let Some(conn) = conn {
                conn.active
                    .store(false, std::sync::atomic::Ordering::Release);
            }
            self.auth_status(token, &binding, AuthStatus::Conflict);
            return;
        }
        if let Some(conn) = conn {
            conn.active
                .store(true, std::sync::atomic::Ordering::Release);
        }
        if !already {
            self.login_done(
                token,
                conn_id,
                Ok((binding.account, binding.profile.clone(), binding.high_water)),
            );
            // P66: a release slot also holds cross-slot ownership; losing it signs
            // this session out exactly like a resumed session (no-op single-server).
            self.reclaim(token, binding.account);
        }
        self.auth_status(token, &binding, AuthStatus::Active);
    }

    fn auth_status(&mut self, token: Token, binding: &Binding, status: AuthStatus) {
        self.send_to(
            token,
            ServerMsg::AuthState {
                status,
                access_expires_at: binding.expires_at as u64,
                session_expires_at: binding.session_expires_at as u64,
            },
        );
    }

    pub(super) fn auth_idle(&self, ids: &[FamilyId]) -> bool {
        !self.sessions.values().any(|s| {
            s.auth_binding
                .as_ref()
                .is_some_and(|b| ids.contains(&b.family))
                && matches!(s.state, SessionState::InMatch { .. })
        })
    }

    pub(super) fn auth_revoke(&mut self, scope: &crate::auth::RevocationScope) -> bool {
        let tokens: Vec<_> = self
            .sessions
            .iter()
            .filter(|(_, s)| {
                s.auth_binding
                    .as_ref()
                    .is_some_and(|b| scope.includes(b.family, b.account))
            })
            .map(|(t, _)| *t)
            .collect();
        let mut delivered = true;
        for token in tokens {
            let Some(session) = self.sessions.get(&token) else {
                continue;
            };
            let conn = session.conn.clone();
            if let Some(conn) = conn {
                conn.active
                    .store(false, std::sync::atomic::Ordering::Release);
                if let SessionState::InMatch { tx, seat } = &session.state {
                    delivered &= tx
                        .try_send(MatchCmd::Detach {
                            seat: *seat,
                            conn_id: conn.id,
                        })
                        .is_ok();
                }
                delivered &= conn.tx.try_send(ConnEvent::Close).is_ok();
                self.disconnected(token, conn.id);
            }
            self.leave_waiting(token);
        }
        delivered
    }
}
