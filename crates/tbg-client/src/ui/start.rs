//! Startup wiring: `?room=CODE` joins that room, and a stored token or
//! account key opens the listen-only probe link (P9/P11). The probe's
//! automatic login lives in `net/socket.rs`.

use bevy::prelude::*;
use tbg_core::protocol::{valid_room_code, RoomCode};

use super::account::Account;
use super::ActionQueue;
use crate::config;
use crate::link::MatchLink;
use crate::net::{Mode, NetLink};

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, (join_from_url, probe_on_load));
}

/// `?room=CODE` joins that private room right away.
fn join_from_url(mut queue: ResMut<ActionQueue>) {
    let Some(code) = config::url_param("room") else {
        return;
    };
    let bytes = code.to_ascii_uppercase().into_bytes();
    if let Ok(code) = RoomCode::try_from(bytes.as_slice()) {
        if valid_room_code(&code) {
            queue.0.push(super::UiAction::JoinRoom(code));
        }
    }
}

/// P9: with a stored session token and no `?room=`, open a listen-only
/// link at once; a live match resumes with no tap, otherwise the socket
/// is reused by the next Quick Match / room action.
/// Post-P14: any new session logs in with the stored key before asking
/// for a mode, so the name and ranked eligibility stay correct.
fn probe_on_load(mut link: NonSendMut<MatchLink>, mut account: ResMut<Account>) {
    if config::url_param("room").is_some() {
        return;
    }
    if config::load_token().is_some() || config::load_account_key().is_some() {
        *link = MatchLink::Net(Box::new(NetLink::new(Mode::Probe, 0.0)));
        account.auto_login = config::load_account_key().is_some();
    }
}
