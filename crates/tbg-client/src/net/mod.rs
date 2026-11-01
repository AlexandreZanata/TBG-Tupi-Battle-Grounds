//! Online match link over WebSocket (`ewebsock`): handshake with session
//! resume, mode request (quick match / room), ping replies, automatic
//! reconnect, and the `?lag=MS` simulator (MS/2 added each way).

mod ack;
pub mod admission;
mod auth;
mod login;
mod socket;

use std::collections::VecDeque;

pub(crate) use socket::GIVE_UP;
use tbg_core::protocol::{
    AccessToken, ClientMsg, InviteToken, LiveId, RoomCode, ServerMsg, Token, PROTOCOL_VERSION,
};

use crate::config;

use ack::AckTracker;

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
    Failed(NetFailure),
}

/// Why a link gave up; the UI maps it to a catalog message (P64).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NetFailure {
    ConnectionLost,
    VersionMismatch,
}

pub struct NetLink {
    restore_pending: bool,
    mode: Mode,
    socket: Option<(ewebsock::WsSender, ewebsock::WsReceiver)>,
    opened: bool,
    /// When the open socket last delivered a frame (P71 silence watchdog).
    heard_at: Option<f64>,
    /// P71: the last server clock, when it last moved, and the last view.
    clock_moved: Option<(u64, f64)>,
    view_at: f64,
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
    auth_epoch: u64,
    auth_access: Option<AccessToken>,
    auth_family: Option<String>,
    held_request: bool,
    login_deadline: Option<f64>,
    outq: VecDeque<(f64, ClientMsg)>,
    inq: VecDeque<(f64, ServerMsg)>,
    /// P47: a `SpectateStart` is active; this link holds no seat.
    watching: bool,
    /// P66: the slot path this link uses, or the admission still pending.
    route: admission::Route,
}

impl NetLink {
    pub fn new(mode: Mode, now: f64) -> NetLink {
        let lag_ms: f64 = config::url_param("lag")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.0);
        let mut link = NetLink {
            restore_pending: !config::resume_ready(),
            mode,
            socket: None,
            opened: false,
            heard_at: None,
            clock_moved: None,
            view_at: 0.0,
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
            auth_epoch: crate::auth::epoch(),
            auth_access: None,
            auth_family: None,
            held_request: false,
            login_deadline: None,
            outq: VecDeque::new(),
            inq: VecDeque::new(),
            watching: false,
            // P66: a probe resumes on the endpoint its session was admitted
            // to; every other mode asks for admission first.
            route: admission::Route::start(
                matches!(mode, Mode::Probe)
                    .then(config::load_resume_endpoint)
                    .flatten(),
                now,
            ),
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

    /// No pending join/login/reconnect may race a launcher update (P66 U09/U11).
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn safe_for_update(&self) -> bool {
        self.is_quiet_probe()
            && !self.restore_pending
            && !self.login_in_flight
            && !self.held_request
            && matches!(self.status, NetStatus::Ready | NetStatus::Failed(_))
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
        // P66: new play only on a slot admission chose; a resumed link may
        // sit on a draining generation.
        if !matches!(self.route, admission::Route::Ready { fresh: true, .. }) {
            return false;
        }
        self.mode = mode;
        self.watching = false;
        if !matches!(mode, Mode::Probe) {
            if self.login_in_flight
                || crate::auth::snapshot().status != crate::auth::Status::Guest
                    && self.auth_access.is_none()
            {
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
        if self.restore_pending {
            if !config::resume_ready() {
                return Vec::new();
            }
            self.restore_pending = false;
            self.token = config::load_token();
            self.route = admission::Route::start(
                matches!(self.mode, Mode::Probe)
                    .then(config::load_resume_endpoint)
                    .flatten(),
                now,
            );
            self.connect(now);
        }
        if self.route.advance(now) {
            self.connect(now);
        }
        if matches!(self.route, admission::Route::UpdateRequired)
            && !matches!(self.status, NetStatus::Failed(_))
        {
            self.status = NetStatus::Failed(NetFailure::VersionMismatch);
            crate::platform::storage::set(
                crate::platform::storage::Scope::Session,
                "update.required",
                "1",
            );
        }
        self.sync_auth(now);
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

    /// Send a mode request that waited for the automatic login.
    fn release_request(&mut self, now: f64) {
        if std::mem::take(&mut self.held_request) {
            self.send_request(now);
        }
    }
}
