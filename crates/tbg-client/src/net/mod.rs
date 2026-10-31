//! Online match link over WebSocket (`ewebsock`): handshake with session
//! resume, mode request (quick match / room), ping replies, automatic
//! reconnect, and the `?lag=MS` simulator (MS/2 added each way).

mod ack;
mod login;
mod socket;

use std::collections::VecDeque;

pub(crate) use socket::GIVE_UP;
use tbg_core::protocol::{
    AccountError, ClientMsg, InviteToken, LiveId, RoomCode, ServerMsg, Token, PROTOCOL_VERSION,
};

use crate::config;

use ack::AckTracker;
use login::{needs_login, LOGIN_TIMEOUT};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Quick,
    /// P12: ranked queue (accounts only; the server refuses guests).
    Ranked,
    Create,
    Join(RoomCode),
    /// P39: create a Quick Challenge invitation.
    CreateChallenge,
    /// P39: consume a Quick Challenge invitation.
    JoinChallenge(InviteToken),
    /// P9: listen-only link opened on page load; resumes a live match if
    /// the server still holds the session, asks for nothing otherwise.
    Probe,
    /// P47: watch a listed live match (`?watch=`), listen-only. Asked again
    /// after every reconnect, since a reload never resumes a watch.
    Watch(LiveId),
}

#[derive(Clone, Debug, PartialEq)]
pub enum NetStatus {
    Connecting,
    Ready,
    Reconnecting { since: f64 },
    Failed(String),
}

pub struct NetLink {
    mode: Mode,
    socket: Option<(ewebsock::WsSender, ewebsock::WsReceiver)>,
    opened: bool,
    token: Option<Token>,
    in_match: bool,
    result_id: Option<tbg_core::protocol::ResultId>,
    discarded_result: Option<tbg_core::protocol::ResultId>,
    pub status: NetStatus,
    lost_at: Option<f64>,
    tries: u32,
    next_try: f64,
    lag: f64,
    ack: AckTracker,
    /// Post-P14: an automatic `Login` is in flight; a mode request waits.
    login_in_flight: bool,
    held_request: bool,
    login_deadline: Option<f64>,
    outq: VecDeque<(f64, ClientMsg)>,
    inq: VecDeque<(f64, ServerMsg)>,
    /// P47: a `SpectateStart` is active; this link holds no seat.
    watching: bool,
}

impl NetLink {
    pub fn new(mode: Mode, now: f64) -> NetLink {
        let lag_ms: f64 = config::url_param("lag")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.0);
        let mut link = NetLink {
            mode,
            socket: None,
            opened: false,
            token: config::load_token(),
            in_match: false,
            result_id: None,
            discarded_result: config::dismissed_result(),
            status: NetStatus::Connecting,
            lost_at: None,
            tries: 0,
            next_try: now,
            lag: lag_ms / 2000.0,
            ack: AckTracker::default(),
            login_in_flight: false,
            held_request: false,
            login_deadline: None,
            outq: VecDeque::new(),
            inq: VecDeque::new(),
            watching: false,
        };
        link.connect(now);
        link
    }

    pub fn lag_ms(&self) -> u32 {
        (self.lag * 2000.0) as u32
    }

    /// A load-time probe outside a match: its connection state is not
    /// something the player asked for, so the UI stays quiet about it.
    pub fn is_quiet_probe(&self) -> bool {
        matches!(self.mode, Mode::Probe) && !self.in_match
    }

    /// p50 tap-to-ack latency in ms (`None` before the first ack).
    pub fn ack_ms(&self) -> Option<f64> {
        self.ack.ms()
    }

    /// Explicit navigation must not recover the dismissed result on a fresh connection.
    pub fn discard_completed(&mut self, id: tbg_core::protocol::ResultId) {
        self.discarded_result = Some(id);
        config::dismiss_result(id);
    }

    /// Reuse an idle open socket for a new mode request (probe upgrade).
    /// Returns false when there is no usable socket; the caller then builds a fresh link.
    pub fn reuse(&mut self, mode: Mode, now: f64) -> bool {
        if self.socket.is_none() || self.in_match || !matches!(self.status, NetStatus::Ready) {
            return false;
        }
        self.mode = mode;
        self.watching = false;
        if !matches!(mode, Mode::Probe) {
            if self.login_in_flight {
                self.held_request = true;
            } else {
                self.send_request(now);
            }
        }
        true
    }

    pub fn send(&mut self, msg: ClientMsg, now: f64) {
        // P47 (BR-LV02): a spectator has no seat, so no intention leaves the client.
        if self.watching && matches!(msg, ClientMsg::Intent { .. } | ClientMsg::Surrender) {
            return;
        }
        if let ClientMsg::Intent { seq, .. } = &msg {
            self.ack.sent(*seq, now);
        }
        self.outq.push_back((now + self.lag, msg));
    }

    fn send_request(&mut self, now: f64) {
        if let Mode::Watch(id) = self.mode {
            self.send(ClientMsg::Spectate { id }, now);
            return;
        }
        let req = match self.mode {
            Mode::Quick => ClientMsg::QuickMatch,
            Mode::Ranked => ClientMsg::RankedMatch,
            Mode::Create => ClientMsg::CreateRoom,
            Mode::Join(code) => ClientMsg::JoinRoom { code },
            Mode::CreateChallenge => ClientMsg::CreateChallenge,
            Mode::JoinChallenge(token) => ClientMsg::JoinChallenge { token },
            Mode::Probe | Mode::Watch(_) => return,
        };
        // D-40: the editor's latest preset, validated by the server while
        // idle and frozen by the request right behind it (same order).
        let ranks = config::load_arcanas().ranks;
        self.send(ClientMsg::SetArcanas { ranks }, now);
        // D-46: the picker's kind, frozen by the same request (BR-SK01).
        let kind = config::load_stronghold();
        self.send(ClientMsg::SetStronghold { kind }, now);
        self.send(req, now);
    }

    /// Pump the socket, release lagged messages, reconnect when needed.
    pub fn poll(&mut self, now: f64) -> Vec<ServerMsg> {
        self.read_socket(now);
        self.flush(now);
        self.reconnect(now);
        let mut out = Vec::new();
        while self.inq.front().is_some_and(|(at, _)| *at <= now) {
            if let Some((_, msg)) = self.inq.pop_front() {
                out.extend(self.handle(msg, now));
            }
        }
        if self.login_in_flight && self.login_deadline.is_some_and(|d| now >= d) {
            self.login_in_flight = false;
            self.login_deadline = None;
            self.release_request(now);
            out.push(ServerMsg::AccountError {
                code: AccountError::BadKey,
            });
        }
        out
    }

    /// Send a mode request that waited for the automatic login.
    fn release_request(&mut self, now: f64) {
        if std::mem::take(&mut self.held_request) {
            self.send_request(now);
        }
    }
}
