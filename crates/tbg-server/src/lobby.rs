//! One task owning sessions, the quick-match queue, and private rooms.
//! Pairs players and spawns match tasks. Never awaits another task.

use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::mpsc;

use tbg_core::protocol::{ClientMsg, RoomCode, Token};
use tbg_core::Seat;

use crate::chan::{ConnEvent, ConnHandle, LobbyCmd, MatchCmd};
use crate::metrics::Metrics;
use crate::names::{display_name, random_bytes};
use crate::Config;

mod matchmaking;

const ROOM_TTL: Duration = Duration::from_secs(600);

pub(super) enum SessionState {
    Idle,
    Queued,
    InRoom,
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
    pub(super) detached_at: Option<Instant>,
}

pub(super) struct Lobby {
    pub(super) sessions: Vec<Session>,
    pub(super) queue: Vec<Token>,
    pub(super) rooms: Vec<(RoomCode, Token, Instant)>,
    pub(super) self_tx: mpsc::Sender<LobbyCmd>,
    pub(super) config: Arc<Config>,
    pub(super) metrics: Arc<Metrics>,
}

pub(crate) async fn run(
    mut rx: mpsc::Receiver<LobbyCmd>,
    self_tx: mpsc::Sender<LobbyCmd>,
    config: Arc<Config>,
    metrics: Arc<Metrics>,
) {
    let mut lobby = Lobby {
        sessions: Vec::new(),
        queue: Vec::new(),
        rooms: Vec::new(),
        self_tx,
        config,
        metrics,
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
    }
}

pub(super) fn send(conn: &Option<ConnHandle>, ev: ConnEvent) {
    if let Some(c) = conn {
        let _ = c.tx.try_send(ev);
    }
}

impl Lobby {
    pub(super) fn session(&mut self, token: &Token) -> Option<&mut Session> {
        self.sessions.iter_mut().find(|s| &s.token == token)
    }

    fn handle(&mut self, cmd: LobbyCmd) {
        match cmd {
            LobbyCmd::Hello {
                resume,
                conn,
                reply,
            } => {
                let pair = self.hello(resume, conn);
                let _ = reply.send(pair);
            }
            LobbyCmd::Client { token, msg } => self.client(token, msg),
            LobbyCmd::Disconnected { token, conn_id } => self.disconnected(token, conn_id),
            LobbyCmd::MatchEnded { tokens } => {
                for t in tokens {
                    if let Some(s) = self.session(&t) {
                        s.state = SessionState::Idle;
                        send(&s.conn, ConnEvent::LeaveMatch);
                    }
                }
            }
        }
    }

    /// Resume a known session (taking over any older socket) or create one.
    fn hello(&mut self, resume: Option<Token>, conn: ConnHandle) -> (Token, String) {
        if let Some(s) = resume.and_then(|t| self.sessions.iter_mut().find(|s| s.token == t)) {
            send(&s.conn, ConnEvent::Close);
            if let SessionState::InMatch { tx, seat } = &s.state {
                let _ = tx.try_send(MatchCmd::Attach {
                    seat: *seat,
                    conn: conn.clone(),
                });
                let _ = conn.tx.try_send(ConnEvent::EnterMatch {
                    seat: *seat,
                    tx: tx.clone(),
                });
            }
            s.conn = Some(conn);
            s.detached_at = None;
            return (s.token, s.name.clone());
        }
        let session = Session {
            token: random_bytes::<16>(),
            name: display_name(),
            conn: Some(conn),
            state: SessionState::Idle,
            detached_at: None,
        };
        let pair = (session.token, session.name.clone());
        self.sessions.push(session);
        pair
    }

    fn client(&mut self, token: Token, msg: ClientMsg) {
        let Some(s) = self.session(&token) else {
            return;
        };
        if matches!(s.state, SessionState::InMatch { .. }) {
            return;
        }
        match msg {
            ClientMsg::QuickMatch => self.quick(token),
            ClientMsg::CreateRoom => self.create_room(token),
            ClientMsg::JoinRoom { code } => self.join_room(token, code),
            ClientMsg::Cancel => self.leave_waiting(token),
            _ => {}
        }
    }

    fn disconnected(&mut self, token: Token, conn_id: u64) {
        let Some(s) = self.session(&token) else {
            return;
        };
        if s.conn.as_ref().is_none_or(|c| c.id != conn_id) {
            return;
        }
        s.conn = None;
        s.detached_at = Some(Instant::now());
        self.leave_waiting(token);
    }

    /// Drop expired rooms and long-gone sessions that are not in a match.
    fn cleanup(&mut self) {
        let now = Instant::now();
        self.rooms.retain(|(.., created)| now - *created < ROOM_TTL);
        let window = Duration::from_secs(self.config.reconnect_secs);
        self.sessions.retain(|s| {
            let gone = s.detached_at.is_some_and(|t| now - t > window);
            !gone || matches!(s.state, SessionState::InMatch { .. })
        });
        let tokens: Vec<Token> = self.sessions.iter().map(|s| s.token).collect();
        self.queue.retain(|t| tokens.contains(t));
    }
}
