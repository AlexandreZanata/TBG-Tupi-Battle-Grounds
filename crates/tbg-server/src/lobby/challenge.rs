//! Single-use Quick Challenge invitations and their expiry (BR-R08).

use std::time::{Duration, Instant};

use tbg_core::protocol::{ErrorCode, InviteToken, MatchMode, ServerMsg, Token};

use super::live::Listing;
use super::{send, Lobby, SessionState};
use crate::chan::ConnEvent;
use crate::names::random_bytes;

impl Lobby {
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
        self.pair([owner, token], MatchMode::Challenge, true, Listing::Private);
    }

    pub(super) fn expire_challenge(&mut self, owner: Token, invite: InviteToken) {
        if let Some(s) = self.session(&owner) {
            if matches!(s.state, SessionState::InChallenge { token } if token == invite) {
                s.state = SessionState::Idle;
                send(&s.conn, ConnEvent::Send(ServerMsg::ChallengeUnavailable));
            }
        }
    }
}
