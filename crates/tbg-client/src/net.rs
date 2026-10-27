//! WebSocket ping: 1 Hz binary echo measures RTT for the overlay.
//!
//! The socket is not `Send` on the web, so it lives in a non-send
//! resource. One exclusive system owns (re)connect, ping, and receive
//! because `Commands` cannot insert non-send resources in Bevy 0.20.

use bevy::prelude::*;
use ewebsock::{WsEvent, WsMessage};

use crate::{config, overlay};

struct NetLink {
    sender: ewebsock::WsSender,
    receiver: ewebsock::WsReceiver,
}

#[derive(Resource)]
struct NetTimers {
    next_ping_ms: f64,
    next_retry_ms: f64,
}

impl Default for NetTimers {
    fn default() -> Self {
        Self {
            next_ping_ms: 0.0,
            next_retry_ms: 0.0,
        }
    }
}

pub fn plugin(app: &mut App) {
    app.init_resource::<NetTimers>()
        .add_systems(Update, net_system);
}

/// Exclusive system: connect, ping every second, drain echoes, reconnect.
fn net_system(world: &mut World) {
    let now_ms = world.resource::<Time>().elapsed_secs_f64() * 1000.0;
    if !world.contains_non_send::<NetLink>() {
        if now_ms >= world.resource::<NetTimers>().next_retry_ms {
            try_connect(world, now_ms);
        }
        return;
    }
    if now_ms >= world.resource::<NetTimers>().next_ping_ms {
        world.resource_mut::<NetTimers>().next_ping_ms = now_ms + 1000.0;
        if let Some(mut link) = world.get_non_send_mut::<NetLink>() {
            link.sender.send(WsMessage::Binary(now_ms.to_le_bytes().to_vec()));
        }
    }
    let mut dead = false;
    let mut rtt: Option<f64> = None;
    if let Some(link) = world.get_non_send::<NetLink>() {
        while let Some(event) = link.receiver.try_recv() {
            match event {
                WsEvent::Message(WsMessage::Binary(bytes)) => {
                    if let Some(v) = decode_rtt(&bytes, now_ms) {
                        rtt = Some(v);
                    }
                }
                WsEvent::Message(_) => {}
                WsEvent::Opened => {}
                WsEvent::Closed | WsEvent::Error(_) => dead = true,
            }
        }
    }
    if dead {
        world.remove_non_send::<NetLink>();
        world.resource_mut::<NetTimers>().next_retry_ms = now_ms + 2000.0;
        world.resource_mut::<overlay::OverlayStats>().rtt_ms = None;
    } else if let Some(v) = rtt {
        world.resource_mut::<overlay::OverlayStats>().rtt_ms = Some(v);
    }
}

fn try_connect(world: &mut World, now_ms: f64) {
    match ewebsock::connect(config::server_url(), ewebsock::Options::default()) {
        Ok((sender, receiver)) => {
            world.insert_non_send(NetLink { sender, receiver });
            world.resource_mut::<NetTimers>().next_ping_ms = now_ms + 1000.0;
        }
        Err(_) => {
            world.resource_mut::<NetTimers>().next_retry_ms = now_ms + 2000.0;
        }
    }
}

/// RTT = now minus the echoed send time; `None` on malformed payloads.
fn decode_rtt(bytes: &[u8], now_ms: f64) -> Option<f64> {
    if bytes.len() != 8 {
        return None;
    }
    let raw: [u8; 8] = bytes.try_into().ok()?;
    Some(now_ms - f64::from_le_bytes(raw))
}
