//! WebSocket ping: 1 Hz binary echo measures RTT for the overlay.
//!
//! The socket is not `Send` on the web, so it lives in a non-send
//! resource. One exclusive system owns (re)connect, ping, and receive
//! because `Commands` cannot insert non-send resources in Bevy 0.20.
//! RTT uses the wall clock (`Instant`), not Bevy's frame `Time`, so it is
//! only off by the time until the next frame polls the socket.

use bevy::{platform::time::Instant, prelude::*};
use ewebsock::{WsEvent, WsMessage};

use crate::{config, overlay};

struct NetLink {
    sender: ewebsock::WsSender,
    receiver: ewebsock::WsReceiver,
    open: bool,
}

#[derive(Resource)]
struct NetTimers {
    start: Instant,
    next_ping_ms: f64,
    next_retry_ms: f64,
}

impl Default for NetTimers {
    fn default() -> Self {
        Self {
            start: Instant::now(),
            next_ping_ms: 0.0,
            next_retry_ms: 0.0,
        }
    }
}

pub fn plugin(app: &mut App) {
    app.init_resource::<NetTimers>()
        .add_systems(Update, net_system);
}

/// Exclusive system: connect, drain events, ping every second, reconnect.
fn net_system(world: &mut World) {
    let now_ms = world.resource::<NetTimers>().start.elapsed().as_secs_f64() * 1000.0;
    if !world.contains_non_send::<NetLink>() {
        if now_ms >= world.resource::<NetTimers>().next_retry_ms {
            try_connect(world, now_ms);
        }
        return;
    }
    let (dead, rtt) = drain_events(world, now_ms);
    if dead {
        world.remove_non_send::<NetLink>();
        world.resource_mut::<NetTimers>().next_retry_ms = now_ms + 2000.0;
        world.resource_mut::<overlay::OverlayStats>().rtt_ms = None;
        return;
    }
    if let Some(v) = rtt {
        world.resource_mut::<overlay::OverlayStats>().rtt_ms = Some(v);
    }
    let ping_due = now_ms >= world.resource::<NetTimers>().next_ping_ms;
    let open = world.get_non_send::<NetLink>().is_some_and(|l| l.open);
    if ping_due && open {
        world.resource_mut::<NetTimers>().next_ping_ms = now_ms + 1000.0;
        if let Some(mut link) = world.get_non_send_mut::<NetLink>() {
            link.sender
                .send(WsMessage::Binary(now_ms.to_le_bytes().to_vec()));
        }
    }
}

/// Returns (connection lost, latest RTT sample).
fn drain_events(world: &mut World, now_ms: f64) -> (bool, Option<f64>) {
    let Some(mut link) = world.get_non_send_mut::<NetLink>() else {
        return (true, None);
    };
    let (mut dead, mut rtt) = (false, None);
    while let Some(event) = link.receiver.try_recv() {
        match event {
            WsEvent::Opened => link.open = true,
            WsEvent::Message(WsMessage::Binary(bytes)) => {
                rtt = decode_rtt(&bytes, now_ms).or(rtt);
            }
            WsEvent::Message(_) => {}
            WsEvent::Closed | WsEvent::Error(_) => dead = true,
        }
    }
    (dead, rtt)
}

fn try_connect(world: &mut World, now_ms: f64) {
    match ewebsock::connect(config::server_url(), ewebsock::Options::default()) {
        Ok((sender, receiver)) => {
            world.insert_non_send(NetLink {
                sender,
                receiver,
                open: false,
            });
            world.resource_mut::<NetTimers>().next_ping_ms = now_ms;
        }
        Err(_) => {
            world.resource_mut::<NetTimers>().next_retry_ms = now_ms + 2000.0;
        }
    }
}

/// RTT = now minus the echoed send time; `None` on malformed payloads.
fn decode_rtt(bytes: &[u8], now_ms: f64) -> Option<f64> {
    let raw: [u8; 8] = bytes.try_into().ok()?;
    Some(now_ms - f64::from_le_bytes(raw))
}
