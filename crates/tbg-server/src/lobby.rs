//! One task owning sessions, the quick-match queue, and private rooms.
//! Pairs players and spawns match tasks. Never awaits another task.

use std::collections::{HashMap, VecDeque};
use std::net::IpAddr;
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::mpsc;

use tbg_core::protocol::{ClientMsg, ErrorCode, InviteToken, Profile, RoomCode, ServerMsg, Token};
use tbg_core::Seat;

use crate::chan::{ConnEvent, ConnHandle, LobbyCmd, MatchCmd, LOBBY_QUEUE};
use crate::limits::Limits;
use crate::metrics::Metrics;
use crate::store::{AccountId, Store};
use crate::Config;

mod account_done;
mod accounts;
mod leaderboard;
mod matchmaking;
mod ranked;
mod sessions;
#[cfg(test)]
mod tests;

const ROOM_TTL: Duration = Duration::from_secs(600);

pub(super) enum SessionState {
    Idle,
    Queued,
    InRoom,
    InChallenge {
        token: InviteToken,
    },
    InMatch {
        tx: mpsc::Sender<MatchCmd>,
        seat: Seat,
    },
}

pub(super) struct Session {
    pub(super) token: Token,
    pub(super) name: String,
    pub(super) conn: Option<ConnHandle>,
    pub(super) state: SessionState,
    /// P11: logged-in account, if any. Memory only.
    pub(super) account: Option<(AccountId, Profile)>,
    /// P11: peer address for rate limits. Memory only, never stored/logged.
    pub(super) ip: IpAddr,
    pub(super) detached_at: Option<Instant>,
}

pub(super) struct Lobby {
    /// P10: O(1) by token (the lobby is not simulation; `HashMap` is allowed).
    pub(super) sessions: HashMap<Token, Session>,
    pub(super) queue: VecDeque<Token>,
    /// P12: ranked FIFO; never mixed with the casual queue (BR-R02).
    pub(super) ranked_queue: VecDeque<Token>,
    /// P9: when each queued session sent `QuickMatch` (for `queue_wait`).
    pub(super) queued_at: HashMap<Token, Instant>,
    pub(super) rooms: Vec<(RoomCode, Token, Instant)>,
    /// P39: single-use Quick Challenge invitations.
    pub(super) challenges: Vec<(InviteToken, Token, Instant)>,
    /// P39: post-match rematch offers for challenge pairs.
    pub(super) rematches: Vec<RematchTicket>,
    pub(super) self_tx: mpsc::Sender<LobbyCmd>,
    pub(super) config: Arc<Config>,
    pub(super) metrics: Arc<Metrics>,
    pub(super) store: Option<Arc<Store>>,
    /// P11: per-IP account rate limits.
    pub(super) limits: Limits,
    pub(super) draining: Arc<AtomicBool>,
}

/// P39: a rematch offer between the two sessions of a challenge match.
pub(super) struct RematchTicket {
    pub(super) players: [Token; 2],
    pub(super) requested: [bool; 2],
    /// `None` while the challenge match is live; `Some(deadline)` after
    /// `MatchEnded`. An open ticket is a ticket with `expires.is_some()`.
    pub(super) expires: Option<Instant>,
}

pub(crate) async fn run(
    mut rx: mpsc::Receiver<LobbyCmd>,
    self_tx: mpsc::Sender<LobbyCmd>,
    config: Arc<Config>,
    metrics: Arc<Metrics>,
    store: Option<Arc<Store>>,
    draining: Arc<AtomicBool>,
) {
    let mut lobby = Lobby {
        sessions: HashMap::new(),
        queue: VecDeque::new(),
        ranked_queue: VecDeque::new(),
        queued_at: HashMap::new(),
        rooms: Vec::new(),
        challenges: Vec::new(),
        rematches: Vec::new(),
        self_tx,
        config,
        metrics,
        store,
        limits: Limits::default(),
        draining,
    };
    let mut cleanup = tokio::time::interval(Duration::from_secs(10));
    loop {
        tokio::select! {
            cmd = rx.recv() => match cmd {
                Some(cmd) => lobby.handle(cmd),
                None => return,
            },
            _ = cleanup.tick() => lobby.cleanup(),
        }
        lobby
            .metrics
            .queue_len
            .store(lobby.queue.len() as u64, Relaxed);
        // P10: worst lobby inbox backlog seen (join bursts must not pile up).
        let inbox = LOBBY_QUEUE.saturating_sub(lobby.self_tx.capacity()) as u64;
        lobby.metrics.lobby_inbox_max.fetch_max(inbox, Relaxed);
    }
}

pub(super) fn send(conn: &Option<ConnHandle>, ev: ConnEvent) {
    if let Some(c) = conn {
        let _ = c.tx.try_send(ev);
    }
}

impl Lobby {
    pub(super) fn session(&mut self, token: &Token) -> Option<&mut Session> {
        self.sessions.get_mut(token)
    }

    fn handle(&mut self, cmd: LobbyCmd) {
        match cmd {
            LobbyCmd::Hello {
                resume,
                conn,
                ip,
                reply,
            } => {
                let pair = self.hello(resume, conn, ip);
                let _ = reply.send(pair);
            }
            LobbyCmd::Client { token, msg } => self.client(token, msg),
            LobbyCmd::Disconnected { token, conn_id } => self.disconnected(token, conn_id),
            LobbyCmd::MatchEnded { tokens } => {
                let names = tokens.map(|t| {
                    self.sessions
                        .get(&t)
                        .map_or_else(String::new, |s| s.name.clone())
                });
                for t in tokens {
                    if let Some(s) = self.session(&t) {
                        s.state = SessionState::Idle;
                        send(&s.conn, ConnEvent::LeaveMatch);
                    }
                }
                if let Some(i) = self
                    .rematches
                    .iter()
                    .position(|r| r.players == tokens && r.expires.is_none())
                {
                    self.rematches[i].expires =
                        Some(Instant::now() + Duration::from_secs(self.config.rematch_ttl_secs));
                    for (seat, token) in tokens.iter().enumerate() {
                        let opponent = names[1 - seat].clone();
                        if let Some(s) = self.session(token) {
                            send(
                                &s.conn,
                                ConnEvent::Send(ServerMsg::RematchOpen { opponent }),
                            );
                        }
                    }
                }
            }
            LobbyCmd::RankedDone { tokens, results } => {
                self.ranked_done(tokens, results);
            }
            LobbyCmd::RegisterDone {
                token,
                conn_id,
                result,
            } => self.register_done(token, conn_id, result),
            LobbyCmd::LoginDone {
                token,
                conn_id,
                result,
            } => self.login_done(token, conn_id, result),
        }
    }

    fn client(&mut self, token: Token, msg: ClientMsg) {
        // P11: account messages are answered from any state (`Unavailable`
        // unless `Idle`); everything else is ignored mid-match.
        match msg {
            ClientMsg::Register { name } => return self.register(token, name),
            ClientMsg::Login { key } => return self.login(token, key),
            ClientMsg::Logout => return self.logout(token),
            // P13: read-only, from any state.
            ClientMsg::Leaderboard { offset } => return self.leaderboard(token, offset),
            _ => {}
        }
        let draining = self.draining.load(Relaxed);
        let Some(s) = self.session(&token) else {
            return;
        };
        if matches!(s.state, SessionState::InMatch { .. }) {
            return;
        }
        let wants_match = matches!(
            msg,
            ClientMsg::QuickMatch
                | ClientMsg::RankedMatch
                | ClientMsg::CreateRoom
                | ClientMsg::JoinRoom { .. }
                | ClientMsg::CreateChallenge
                | ClientMsg::JoinChallenge { .. }
                | ClientMsg::RequestRematch
        );
        if wants_match && draining {
            // Shutting down: running matches finish, no new ones start.
            let code = ErrorCode::ServerFull;
            send(&s.conn, ConnEvent::Send(ServerMsg::Error { code }));
            return;
        }
        match msg {
            ClientMsg::QuickMatch => self.quick(token),
            ClientMsg::RankedMatch => self.ranked(token),
            ClientMsg::CreateRoom => self.create_room(token),
            ClientMsg::JoinRoom { code } => self.join_room(token, code),
            ClientMsg::CreateChallenge => self.create_challenge(token),
            ClientMsg::JoinChallenge { token: invite } => self.join_challenge(token, invite),
            ClientMsg::RequestRematch => self.request_rematch(token),
            ClientMsg::Cancel => self.leave_waiting(token),
            _ => {}
        }
    }
}
