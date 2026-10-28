//! Online match link over WebSocket (`ewebsock`): handshake with session
//! resume, mode request (quick match / room), ping replies, automatic
//! reconnect, and the `?lag=MS` simulator (MS/2 added each way).

use std::collections::VecDeque;

use ewebsock::{WsEvent, WsMessage};
use tbg_core::protocol::{self, ClientMsg, RoomCode, ServerMsg, Token, PROTOCOL_VERSION};

use crate::config;

/// Give up reconnecting after this long (server window is 30 s).
const RECONNECT_FOR: f64 = 35.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Quick,
    Create,
    Join(RoomCode),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NetStatus {
    Connecting,
    Ready,
    Reconnecting,
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
    next_try: f64,
    lag: f64,
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
            next_try: now,
            lag: lag_ms / 2000.0,
            outq: VecDeque::new(),
            inq: VecDeque::new(),
        };
        link.connect(now);
        link
    }

    pub fn lag_ms(&self) -> u32 {
        (self.lag * 2000.0) as u32
    }

    pub fn send(&mut self, msg: ClientMsg, now: f64) {
        self.outq.push_back((now + self.lag, msg));
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

    fn connect(&mut self, now: f64) {
        self.opened = false;
        self.outq.clear();
        match ewebsock::connect(config::server_url(), ewebsock::Options::default()) {
            Ok(pair) => self.socket = Some(pair),
            Err(_) => self.lost(now),
        }
    }

    fn lost(&mut self, now: f64) {
        self.socket = None;
        self.opened = false;
        if !matches!(self.status, NetStatus::Failed(_)) {
            self.status = NetStatus::Reconnecting;
            self.lost_at.get_or_insert(now);
            self.next_try = now + 1.0;
        }
    }

    fn read_socket(&mut self, now: f64) {
        let mut events = Vec::new();
        if let Some((_, rx)) = &self.socket {
            while let Some(ev) = rx.try_recv() {
                events.push(ev);
            }
        }
        for ev in events {
            match ev {
                WsEvent::Opened => {
                    self.opened = true;
                    let hello = ClientMsg::Hello {
                        version: PROTOCOL_VERSION,
                        resume: self.token,
                    };
                    self.outq.push_front((now + self.lag, hello));
                }
                WsEvent::Message(WsMessage::Binary(bytes)) => {
                    if let Ok(msg) = protocol::decode_server(&bytes) {
                        self.inq.push_back((now + self.lag, msg));
                    }
                }
                WsEvent::Message(_) => {}
                WsEvent::Closed | WsEvent::Error(_) => self.lost(now),
            }
        }
    }

    fn flush(&mut self, now: f64) {
        let Some((tx, _)) = &mut self.socket else {
            return;
        };
        while self.opened && self.outq.front().is_some_and(|(at, _)| *at <= now) {
            if let Some((_, msg)) = self.outq.pop_front() {
                tx.send(WsMessage::Binary(protocol::encode(&msg)));
            }
        }
    }

    fn reconnect(&mut self, now: f64) {
        let Some(lost) = self.lost_at else { return };
        if self.socket.is_some() || matches!(self.status, NetStatus::Failed(_)) {
            return;
        }
        if now - lost > RECONNECT_FOR {
            self.status = NetStatus::Failed("Connection lost".into());
        } else if now >= self.next_try {
            self.next_try = now + 1.0;
            self.connect(now);
        }
    }

    /// Protocol housekeeping stays here; everything else goes to the game.
    fn handle(&mut self, msg: ServerMsg, now: f64) -> Option<ServerMsg> {
        match msg {
            ServerMsg::Welcome { token, .. } => {
                config::save_token(&token);
                self.token = Some(token);
                self.status = NetStatus::Ready;
                self.lost_at = None;
                // Outside a match (first connect, or dropped while queued, which
                // takes us out of the queue) ask for the mode again.
                if !self.in_match {
                    let req = match self.mode {
                        Mode::Quick => ClientMsg::QuickMatch,
                        Mode::Create => ClientMsg::CreateRoom,
                        Mode::Join(code) => ClientMsg::JoinRoom { code },
                    };
                    self.send(req, now);
                }
                None
            }
            ServerMsg::Ping { id } => {
                self.send(ClientMsg::Pong { id }, now);
                None
            }
            ServerMsg::VersionMismatch { .. } => {
                self.status =
                    NetStatus::Failed("A new version is available - reload the page".into());
                None
            }
            other => {
                match other {
                    ServerMsg::MatchStart { .. } => self.in_match = true,
                    ServerMsg::MatchEnd { .. } => self.in_match = false,
                    _ => {}
                }
                Some(other)
            }
        }
    }
}
