//! One task owning sessions, the quick-match queue, and private rooms.
//! Pairs players and spawns match tasks. Never awaits another task.

use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::mpsc;

use tbg_core::protocol::{ClientMsg, ErrorCode, RoomCode, ServerMsg, Token};
use tbg_core::Seat;

use crate::chan::{ConnEvent, ConnHandle, LobbyCmd, MatchCmd};
use crate::metrics::Metrics;
use crate::records::Records;
use crate::Config;

mod matchmaking;
mod sessions;

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
    /// P9: when each queued session sent `QuickMatch` (for `queue_wait`).
    pub(super) queued_at: std::collections::HashMap<Token, Instant>,
    pub(super) rooms: Vec<(RoomCode, Token, Instant)>,
    pub(super) self_tx: mpsc::Sender<LobbyCmd>,
    pub(super) config: Arc<Config>,
    pub(super) metrics: Arc<Metrics>,
    pub(super) records: Option<Arc<Records>>,
    pub(super) draining: Arc<AtomicBool>,
}

pub(crate) async fn run(
    mut rx: mpsc::Receiver<LobbyCmd>,
    self_tx: mpsc::Sender<LobbyCmd>,
    config: Arc<Config>,
    metrics: Arc<Metrics>,
    records: Option<Arc<Records>>,
    draining: Arc<AtomicBool>,
) {
    let mut lobby = Lobby {
        sessions: Vec::new(),
        queue: Vec::new(),
        queued_at: std::collections::HashMap::new(),
        rooms: Vec::new(),
        self_tx,
        config,
        metrics,
        records,
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

    fn client(&mut self, token: Token, msg: ClientMsg) {
        let draining = self.draining.load(Relaxed);
        let Some(s) = self.session(&token) else {
            return;
        };
        if matches!(s.state, SessionState::InMatch { .. }) {
            return;
        }
        let wants_match = matches!(
            msg,
            ClientMsg::QuickMatch | ClientMsg::CreateRoom | ClientMsg::JoinRoom { .. }
        );
        if wants_match && draining {
            // Shutting down: running matches finish, no new ones start.
            let code = ErrorCode::ServerFull;
            send(&s.conn, ConnEvent::Send(ServerMsg::Error { code }));
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
}
