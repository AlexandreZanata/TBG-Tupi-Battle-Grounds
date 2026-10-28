//! Messages between tasks: one task per connection, one lobby task, one
//! task per match. All channels are bounded; lobby and match tasks only
//! `try_send`, so a slow client can never stall them.

use tokio::sync::{mpsc, oneshot};

use tbg_core::protocol::{ClientMsg, ServerMsg, Token};
use tbg_core::{PlayerIntention, Seat};

// P10: 32 measured against 64 below (kept only with no slow-client drops).
pub const CONN_QUEUE: usize = 32;
pub const LOBBY_QUEUE: usize = 1024;
pub const MATCH_QUEUE: usize = 256;

/// The only way to talk to a client.
#[derive(Clone)]
pub struct ConnHandle {
    pub id: u64,
    pub tx: mpsc::Sender<ConnEvent>,
}

pub enum ConnEvent {
    Send(ServerMsg),
    EnterMatch {
        seat: Seat,
        tx: mpsc::Sender<MatchCmd>,
    },
    LeaveMatch,
    Close,
}

pub enum MatchCmd {
    /// `seat` is stamped by the connection task, never read from the client.
    Intent {
        seat: Seat,
        seq: u32,
        intent: PlayerIntention,
    },
    /// Ignored unless `conn_id` is the seat's current connection.
    Detach {
        seat: Seat,
        conn_id: u64,
    },
    Attach {
        seat: Seat,
        conn: ConnHandle,
    },
    Rtt {
        seat: Seat,
        ms: u16,
    },
}

pub enum LobbyCmd {
    Hello {
        resume: Option<Token>,
        conn: ConnHandle,
        reply: oneshot::Sender<(Token, String)>,
    },
    Client {
        token: Token,
        msg: ClientMsg,
    },
    /// Ignored unless `conn_id` is the session's current connection.
    Disconnected {
        token: Token,
        conn_id: u64,
    },
    MatchEnded {
        tokens: [Token; 2],
    },
}
