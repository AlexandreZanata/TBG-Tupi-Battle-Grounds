//! `MatchLink`: the only way the client talks to a match. `Local` runs
//! practice against the bot in the browser; `Net` talks to the server.
//! Stored as a non-send resource (`NonSendMut<MatchLink>`): the web socket
//! is not `Send`.

use bevy::prelude::*;
use tbg_core::protocol::{ClientMsg, ServerMsg};

use crate::local::LocalMatch;
use crate::net::{NetLink, NetStatus};

pub enum MatchLink {
    Idle,
    Local(Box<LocalMatch>),
    Net(Box<NetLink>),
}

impl MatchLink {
    pub fn send(&mut self, msg: ClientMsg, now: f64) {
        match self {
            MatchLink::Idle => {}
            MatchLink::Local(m) => m.send(msg),
            MatchLink::Net(n) => n.send(msg, now),
        }
    }

    /// Messages produced since the last call.
    pub fn poll(&mut self, now: f64) -> Vec<ServerMsg> {
        match self {
            MatchLink::Idle => Vec::new(),
            MatchLink::Local(m) => m.take_outbox(),
            MatchLink::Net(n) => n.poll(now),
        }
    }

    pub fn status(&self) -> Option<&NetStatus> {
        match self {
            MatchLink::Net(n) => Some(&n.status),
            _ => None,
        }
    }

    pub fn lag_ms(&self) -> u32 {
        match self {
            MatchLink::Net(n) => n.lag_ms(),
            _ => 0,
        }
    }
}

pub fn plugin(app: &mut App) {
    app.insert_non_send(MatchLink::Idle)
        .add_systems(FixedUpdate, tick_local);
}

/// Practice matches advance one simulation tick per fixed step (20 Hz).
fn tick_local(mut link: NonSendMut<MatchLink>) {
    if let MatchLink::Local(m) = &mut *link {
        m.tick();
    }
}
