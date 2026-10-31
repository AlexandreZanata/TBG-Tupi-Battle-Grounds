//! One task owning sessions, the quick-match queue, and private rooms.
//! Pairs players and spawns match tasks. Never awaits another task.

use std::collections::{HashMap, VecDeque};
use std::net::IpAddr;
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::{mpsc, watch};

use tbg_core::protocol::{
    ClientMsg, ErrorCode, InviteToken, LiveId, Profile, RoomCode, ServerMsg, Token,
};
use tbg_core::Seat;

use crate::chan::{ConnEvent, ConnHandle, LobbyCmd, MatchCmd, LOBBY_QUEUE};
use crate::limits::Limits;
use crate::live::LiveSnapshot;
use crate::metrics::Metrics;
use crate::store::{AccountId, Store};
use crate::Config;

mod account_done;
mod accounts;
mod arcanas;
mod challenge;
mod leaderboard;
mod live;
mod matchmaking;
mod presence;
mod ranked;
mod rematch;
mod sessions;
mod spectate;
mod strongholds;
#[cfg(test)]
mod tests;

const ROOM_TTL: Duration = Duration::from_secs(600);
/// BR-LV07 / ARCHITECTURE §5: the live snapshot is rebuilt at most this often.
const PUBLISH_EVERY: Duration = Duration::from_secs(1);

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
    /// P47 (BR-LV02): listen-only in a public match; never a seat.
    Watching {
        id: LiveId,
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
    /// D-40: the Arcanas the next match will freeze (BR-AR05).
    pub(super) arcanas: tbg_core::ArcanaPreset,
    /// D-46: the Stronghold kind the next match will freeze (BR-SK01).
    pub(super) stronghold: tbg_core::StrongholdKind,
    /// P47 (BR-LV09): when this session last sent `Spectate` (one per second).
    pub(super) last_spectate: Option<Instant>,
    pub(super) finished: Option<(Instant, Box<crate::chan::CompletedResult>)>,
}

pub(super) struct Lobby {
    /// P10: O(1) by token (the lobby is not simulation; `HashMap` is allowed).
    pub(super) sessions: HashMap<Token, Session>,
    pub(super) queue: VecDeque<Token>,
    /// P12: ranked FIFO; never mixed with the casual queue (BR-R02).
    pub(super) ranked_queue: VecDeque<Token>,
    /// P9: when each queued session sent `QuickMatch` (for `queue_wait`).
    pub(super) queued_at: HashMap<Token, Instant>,
    /// P60: when each ranked waiter sent `RankedMatch`; never read by the
    /// Casual queue or its fallbacks.
    pub(super) ranked_queued_at: HashMap<Token, Instant>,
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
    /// P47: public matches running now (BR-LV01); private ones never enter.
    pub(super) live: HashMap<LiveId, live::LiveMatch>,
    /// P47: the snapshot `/api/live` and `/api/online` read.
    pub(super) live_tx: watch::Sender<Arc<LiveSnapshot>>,
    /// P47 (BR-LV07): the featured match and when it got the spot.
    pub(super) featured: Option<(LiveId, Instant)>,
    /// P60: the ranked finalizer (only with a database).
    pub(super) ranked: Option<crate::ranked::finalizer::Handle>,
    /// P60: per account, the newest settlement sequence its cached profile
    /// already contains; an older receipt never regresses it.
    pub(super) ranked_seen: HashMap<AccountId, i64>,
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
    live_tx: watch::Sender<Arc<LiveSnapshot>>,
) {
    let mut lobby = Lobby {
        sessions: HashMap::new(),
        queue: VecDeque::new(),
        ranked_queue: VecDeque::new(),
        queued_at: HashMap::new(),
        ranked_queued_at: HashMap::new(),
        rooms: Vec::new(),
        challenges: Vec::new(),
        rematches: Vec::new(),
        self_tx,
        config,
        metrics,
        store,
        limits: Limits::default(),
        draining,
        live: HashMap::new(),
        live_tx,
        featured: None,
        ranked: None,
        ranked_seen: HashMap::new(),
    };
    // P60: one finalizer per process settles every ranked result in order;
    // its journal recovery finishes before the lobby serves any command.
    if let (Some(store), Some(db)) = (lobby.store.clone(), lobby.config.db.clone()) {
        let handle = crate::ranked::finalizer::start(
            store,
            db,
            lobby.self_tx.clone(),
            lobby.metrics.clone(),
        )
        .await;
        lobby.ranked = Some(handle);
    }
    let mut cleanup = tokio::time::interval(Duration::from_secs(10));
    let mut publish = tokio::time::interval(PUBLISH_EVERY);
    loop {
        tokio::select! {
            cmd = rx.recv() => match cmd {
                Some(cmd) => lobby.handle(cmd),
                None => return,
            },
            _ = cleanup.tick() => lobby.cleanup(),
            _ = publish.tick() => lobby.publish_live(Instant::now()),
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
    pub(super) fn send_to(&mut self, token: Token, msg: ServerMsg) {
        if let Some(s) = self.session(&token) {
            send(&s.conn, ConnEvent::Send(msg));
        }
    }

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
            LobbyCmd::LiveProgress { id, progress } => self.live_progress(id, progress),
            LobbyCmd::MatchEnded {
                tokens,
                live,
                finished,
            } => {
                if let Some(id) = live {
                    self.live_ended(id);
                }
                let names = tokens.map(|t| {
                    self.sessions
                        .get(&t)
                        .map_or_else(String::new, |s| s.name.clone())
                });
                let until = Instant::now() + Duration::from_secs(self.config.reconnect_secs);
                for (t, result) in tokens.into_iter().zip(*finished) {
                    if let Some(s) = self.session(&t) {
                        s.state = SessionState::Idle;
                        // Also covers a reconnect racing the final match-task tick.
                        if s.conn.as_ref().map(|c| c.id) != result.delivered_to {
                            send(&s.conn, ConnEvent::Send(result.start.clone()));
                            send(&s.conn, ConnEvent::Send(result.end.clone()));
                        }
                        s.finished = Some((until, Box::new(result)));
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
            LobbyCmd::RankedDone { receipt } => self.ranked_done(*receipt),
            LobbyCmd::RegisterDone {
                token,
                conn_id,
                ip,
                result,
            } => self.register_done(token, conn_id, ip, result),
            LobbyCmd::LoginDone {
                token,
                conn_id,
                result,
            } => self.login_done(token, conn_id, result),
            LobbyCmd::ArcanasLoaded {
                token,
                account,
                preset,
            } => self.arcanas_loaded(token, account, preset),
            LobbyCmd::StrongholdLoaded {
                token,
                account,
                kind,
            } => self.stronghold_loaded(token, account, kind),
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
            // D-40: answered in every state (`NotIdle` unless idle).
            ClientMsg::SetArcanas { ranks } => return self.set_arcanas(token, ranks),
            // D-46: same idle-only rule as the Arcanas (BR-SK01).
            ClientMsg::SetStronghold { kind } => return self.set_stronghold(token, kind),
            // P47: watching is listen-only and answered from any state.
            ClientMsg::Spectate { id } => return self.spectate(token, id),
            ClientMsg::StopSpectating => return self.end_watch(token),
            _ => {}
        }
        // Any other lobby request ends a watch first (BR-LV02).
        self.end_watch(token);
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
