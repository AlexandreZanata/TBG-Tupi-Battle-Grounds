//! `MatchLink`: the only way the client talks to a match. `Local` runs
//! practice against the bot in the browser; P4 adds `Net`.
//! Stored as a non-send resource (`NonSendMut<MatchLink>`): the web socket
//! is not `Send`.

use bevy::prelude::*;
use tbg_core::protocol::{ClientMsg, ServerMsg};

use crate::local::LocalMatch;

pub enum MatchLink {
    Idle,
    Local(Box<LocalMatch>),
}

impl MatchLink {
    pub fn send(&mut self, msg: ClientMsg) {
        if let MatchLink::Local(m) = self {
            m.send(msg);
        }
    }

    /// Messages produced since the last call.
    pub fn poll(&mut self) -> Vec<ServerMsg> {
        match self {
            MatchLink::Idle => Vec::new(),
            MatchLink::Local(m) => m.take_outbox(),
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
