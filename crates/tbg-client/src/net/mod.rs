//! Online match link over WebSocket (`ewebsock`): handshake with session
//! resume, mode request (quick match / room), ping replies, automatic
//! reconnect, and the `?lag=MS` simulator (MS/2 added each way).

mod ack;
mod socket;

use std::collections::VecDeque;

pub(crate) use socket::GIVE_UP;
use tbg_core::protocol::{ClientMsg, RoomCode, ServerMsg, Token, PROTOCOL_VERSION};

use crate::config;

use ack::AckTracker;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Quick,
    Create,
    Join(RoomCode),
    /// P9: listen-only link opened on page load; resumes a live match if
    /// the server still holds the session, asks for nothing otherwise.
    Probe,
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
    pub status: NetStatus,
    lost_at: Option<f64>,
    tries: u32,
    next_try: f64,
    lag: f64,
    ack: AckTracker,
    outq: VecDeque<(f64, ClientMsg)>,
    inq: VecDeque<(f64, ServerMsg)>,
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
            status: NetStatus::Connecting,
            lost_at: None,
            tries: 0,
            next_try: now,
            lag: lag_ms / 2000.0,
            ack: AckTracker::default(),
            outq: VecDeque::new(),
            inq: VecDeque::new(),
        };
        link.connect(now);
        link
    }

    pub fn lag_ms(&self) -> u32 {
        (self.lag * 2000.0) as u32
    }

    /// p50 tap-to-ack latency in ms (`None` before the first ack).
    pub fn ack_ms(&self) -> Option<f64> {
        self.ack.ms()
    }

    /// Reuse an idle open socket for a new mode request (probe upgrade).
    /// Returns false when there is no usable socket; the caller then
    /// builds a fresh link.
    pub fn reuse(&mut self, mode: Mode, now: f64) -> bool {
        if self.socket.is_none() || self.in_match || !matches!(self.status, NetStatus::Ready) {
            return false;
        }
        self.mode = mode;
        if !matches!(mode, Mode::Probe) {
            self.send_request(now);
        }
        true
    }

    pub fn send(&mut self, msg: ClientMsg, now: f64) {
        if let ClientMsg::Intent { seq, .. } = &msg {
            self.ack.sent(*seq, now);
        }
        self.outq.push_back((now + self.lag, msg));
    }

    fn send_request(&mut self, now: f64) {
        let req = match self.mode {
            Mode::Quick => ClientMsg::QuickMatch,
            Mode::Create => ClientMsg::CreateRoom,
            Mode::Join(code) => ClientMsg::JoinRoom { code },
            Mode::Probe => return,
        };
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
        out
    }
}
