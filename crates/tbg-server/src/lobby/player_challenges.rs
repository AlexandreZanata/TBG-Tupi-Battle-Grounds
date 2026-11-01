//! Participant-bound, bounded direct invitations; no record is a bearer seat token.

use super::{live::Listing, Lobby, SessionState};
use std::time::{Duration, Instant};
use tbg_core::protocol::{
    MatchMode, PlayerInviteId, PlayerInviteNotice, PlayerInviteOutcome as Outcome, ServerMsg, Token,
};

const TTL: Duration = Duration::from_secs(60);
const COOLDOWN: Duration = Duration::from_secs(15);
const CAP: usize = 512;

pub(crate) struct DirectedInvite {
    id: PlayerInviteId,
    sender: Token,
    recipient: Token,
    deadline: Instant,
}

impl Lobby {
    /// Snapshot eligibility is advisory; acceptance repeats every gate in the actor.
    pub(super) fn direct_ready(&self, token: Token) -> bool {
        self.direct_available(token)
            && self
                .sessions
                .get(&token)
                .is_some_and(|s| s.player_challenges_enabled)
    }

    fn direct_available(&self, token: Token) -> bool {
        self.release.gate.admits()
            && self.auth_can_start(token)
            && self
                .sessions
                .get(&token)
                .is_some_and(|s| s.conn.is_some() && matches!(s.state, SessionState::Idle))
    }

    pub(super) fn direct_pending(&self, token: Token) -> bool {
        self.player_challenges
            .iter()
            .any(|r| r.sender == token || r.recipient == token)
    }

    fn direct_limit(&mut self, token: Token) -> bool {
        let now = Instant::now();
        let Some(s) = self.sessions.get_mut(&token) else {
            return false;
        };
        let window = Duration::from_secs(60);
        let max = self.config.invite_per_ip_min.min(30) as usize;
        if !rate_log(&mut s.player_challenge_actions, now, max) {
            return false;
        }
        self.player_challenge_accounts.retain(|_, log| {
            log.retain(|at| now.duration_since(*at) < window);
            !log.is_empty()
        });
        if let Some((account, _)) = &s.account {
            if self.player_challenge_accounts.len() >= CAP
                && !self.player_challenge_accounts.contains_key(account)
            {
                return false;
            }
            if !rate_log(
                self.player_challenge_accounts.entry(*account).or_default(),
                now,
                max,
            ) {
                return false;
            }
        }
        self.limits
            .check_invite(s.ip, self.config.invite_per_ip_min as u32, now)
    }

    pub(super) fn direct_action(&mut self, token: Token, msg: tbg_core::protocol::ClientMsg) {
        self.sweep_direct();
        // Closing eligibility is always honored even if a shared IP exhausted
        // its create budget; the connection bucket still bounds responses.
        if matches!(
            msg,
            tbg_core::protocol::ClientMsg::SetPlayerChallengesEnabled { enabled: false }
        ) {
            if let Some(s) = self.sessions.get_mut(&token) {
                s.player_challenges_enabled = false;
            }
            self.cancel_direct(token);
            self.send_direct_state(token);
            return;
        }
        if !self.direct_limit(token) {
            self.direct_result(token, None, Outcome::RateLimited);
            return;
        }
        use tbg_core::protocol::ClientMsg::*;
        match msg {
            ChallengePlayer { name } => self.challenge_player(token, name),
            AnswerPlayerChallenge { id, accept } => self.answer_player(token, id, accept),
            CancelPlayerChallenge { id } => {
                if let Some(i) = self
                    .player_challenges
                    .iter()
                    .position(|r| r.id == id && r.sender == token)
                {
                    self.finish_direct(i, Outcome::Cancelled);
                } else {
                    self.direct_result(token, Some(id), Outcome::Unavailable);
                }
            }
            SetPlayerChallengesEnabled { enabled } => {
                if let Some(s) = self.sessions.get_mut(&token) {
                    s.player_challenges_enabled = enabled;
                }
                if !enabled {
                    self.cancel_direct(token);
                }
                self.send_direct_state(token);
            }
            _ => {}
        }
    }

    fn challenge_player(&mut self, sender: Token, name: String) {
        let target = tbg_core::account::valid_name_shape(&name)
            .then(|| {
                self.sessions
                    .iter()
                    .find(|(_, s)| {
                        s.account.is_some()
                            && s.name.eq_ignore_ascii_case(&name)
                            && s.conn.is_some()
                    })
                    .map(|(t, _)| *t)
            })
            .flatten();
        let Some(recipient) = target.filter(|t| *t != sender) else {
            self.direct_result(sender, None, Outcome::Unavailable);
            return;
        };
        let pair = pair_key(sender, recipient);
        // Repeated and crossed Send acknowledge the same invitation, never Accept.
        if self
            .player_challenges
            .iter()
            .any(|r| pair_key(r.sender, r.recipient) == pair)
        {
            self.send_direct_state(sender);
            return;
        }
        let full = self.player_challenges.len() >= CAP
            || self.player_challenge_cooldowns.len() >= CAP
            || self.player_challenges.iter().any(|r| r.sender == recipient)
            || self.player_challenges.iter().any(|r| r.sender == sender)
            || self
                .player_challenges
                .iter()
                .filter(|r| r.recipient == recipient)
                .count()
                >= 5;
        let cooling = self
            .player_challenge_cooldowns
            .iter()
            .any(|(a, b, _)| (*a, *b) == pair);
        if full
            || cooling
            || self.direct_pending(sender)
            || !self.direct_available(sender)
            || !self.direct_ready(recipient)
        {
            self.direct_result(sender, None, Outcome::Unavailable);
            return;
        }
        let id = crate::names::random_bytes::<16>();
        self.player_challenges.push(DirectedInvite {
            id,
            sender,
            recipient,
            deadline: Instant::now() + TTL,
        });
        self.player_challenge_cooldowns
            .push((pair.0, pair.1, Instant::now() + COOLDOWN));
        self.send_direct_state(sender);
        self.send_direct_state(recipient);
    }

    fn answer_player(&mut self, token: Token, id: PlayerInviteId, accept: bool) {
        let Some(i) = self
            .player_challenges
            .iter()
            .position(|r| r.id == id && r.recipient == token)
        else {
            self.direct_result(token, Some(id), Outcome::Unavailable);
            return;
        };
        if !accept {
            self.finish_direct(i, Outcome::Declined);
            return;
        }
        let r = &self.player_challenges[i];
        if [r.sender, r.recipient]
            .iter()
            .any(|t| self.sessions.get(t).is_some_and(|s| s.conn.is_none()))
        {
            self.direct_result(token, Some(id), Outcome::Unavailable);
            self.send_direct_state(token);
            return;
        }
        if Instant::now() >= r.deadline
            || !self.direct_available(r.sender)
            || !self.direct_ready(r.recipient)
        {
            self.finish_direct(i, Outcome::Unavailable);
            return;
        }
        let r = self.player_challenges.remove(i);
        let players = [r.sender, r.recipient];
        for t in players {
            self.leave_waiting(t);
            self.send_direct_state(t);
        }
        self.pair(players, MatchMode::Challenge, true, Listing::Private);
    }

    pub(super) fn cancel_direct(&mut self, token: Token) {
        while let Some(i) = self
            .player_challenges
            .iter()
            .position(|r| r.sender == token || r.recipient == token)
        {
            self.finish_direct(i, Outcome::Cancelled);
        }
    }

    pub(super) fn sweep_direct(&mut self) {
        let now = Instant::now();
        self.player_challenge_cooldowns
            .retain(|(_, _, until)| *until > now);
        while let Some(i) = self.player_challenges.iter().position(|r| {
            now >= r.deadline
                || !self.sessions.contains_key(&r.sender)
                || !self.sessions.contains_key(&r.recipient)
                || !self.auth_can_start(r.sender)
                || !self.auth_can_start(r.recipient)
        }) {
            let outcome = if now >= self.player_challenges[i].deadline {
                Outcome::Expired
            } else {
                Outcome::Unavailable
            };
            self.finish_direct(i, outcome);
        }
    }

    fn finish_direct(&mut self, i: usize, outcome: Outcome) {
        let r = self.player_challenges.remove(i);
        for token in [r.sender, r.recipient] {
            self.direct_result(token, Some(r.id), outcome);
            self.send_direct_state(token);
        }
    }

    fn direct_result(&mut self, token: Token, id: Option<PlayerInviteId>, outcome: Outcome) {
        self.send_to(token, ServerMsg::PlayerChallengeResult { id, outcome });
    }

    pub(super) fn send_direct_state(&mut self, token: Token) {
        let enabled = self
            .sessions
            .get(&token)
            .is_some_and(|s| s.player_challenges_enabled);
        let notice = |r: &DirectedInvite, peer: Token| {
            self.sessions.get(&peer).map(|s| PlayerInviteNotice {
                id: r.id,
                peer_name: s.name.clone(),
                peer_is_guest: s.account.is_none(),
                remaining_s: r
                    .deadline
                    .saturating_duration_since(Instant::now())
                    .as_secs()
                    .min(60) as u16,
            })
        };
        let outgoing = self
            .player_challenges
            .iter()
            .find(|r| r.sender == token)
            .and_then(|r| notice(r, r.recipient));
        let incoming = self
            .player_challenges
            .iter()
            .filter(|r| r.recipient == token)
            .take(5)
            .filter_map(|r| notice(r, r.sender))
            .collect();
        self.send_to(
            token,
            ServerMsg::PlayerChallengeState {
                enabled,
                outgoing,
                incoming,
            },
        );
    }
}

fn pair_key(a: Token, b: Token) -> (Token, Token) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}

#[cfg(test)]
mod tests;

fn rate_log(log: &mut Vec<Instant>, now: Instant, max: usize) -> bool {
    log.retain(|at| now.duration_since(*at) < Duration::from_secs(60));
    if log.len() >= max {
        return false;
    }
    log.push(now);
    true
}
