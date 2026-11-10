//! Online play: one WebSocket to `tbg-moba-server`; the server owns every outcome.

use bevy::prelude::*;
use ewebsock::{WsEvent, WsMessage, WsReceiver, WsSender};
use std::collections::VecDeque;
use tbg_moba_core::protocol::{self, ClientMsg, ServerMsg, MOBA_PROTOCOL_VERSION};
use tbg_moba_core::{content::TICK_HZ, protocol::WorldView};

use super::{FailReason, Feed, Outbox, Status};

/// Seconds between keep-alive pings (the server closes silent sockets after 30 s).
const PING_EVERY: f64 = 2.0;

pub struct Socket {
    sender: WsSender,
    receiver: WsReceiver,
    last_ping: f64,
    playback: Playback,
}

pub fn connect(world: &mut World, url: &str) {
    let mut feed = world.resource_mut::<Feed>();
    match ewebsock::connect(url, ewebsock::Options::default()) {
        Ok((sender, receiver)) => {
            feed.status = Status::Connecting;
            world.insert_non_send(Socket {
                sender,
                receiver,
                last_ping: 0.0,
                playback: Playback::default(),
            });
        }
        Err(error) => {
            warn!("MOBA connection failed: {error}");
            feed.status = Status::Failed(FailReason::Lost);
        }
    }
}

fn send(socket: &mut Socket, message: &ClientMsg) {
    socket
        .sender
        .send(WsMessage::Binary(protocol::encode(message)));
}

pub fn run(
    socket: Option<NonSendMut<Socket>>,
    mut feed: ResMut<Feed>,
    mut outbox: ResMut<Outbox>,
    time: Res<Time>,
) {
    let Some(mut socket) = socket else { return };
    let now = time.elapsed_secs_f64();
    while let Some(event) = socket.receiver.try_recv() {
        match event {
            WsEvent::Opened => {
                let hello = ClientMsg::Hello {
                    protocol: MOBA_PROTOCOL_VERSION,
                    name: "TBG".into(),
                };
                send(&mut socket, &hello);
            }
            WsEvent::Message(WsMessage::Binary(bytes)) => {
                receive(&mut feed, &mut socket.playback, &bytes, now)
            }
            WsEvent::Message(_) => {}
            WsEvent::Error(error) => {
                warn!("MOBA socket error: {error}");
                feed.status = Status::Failed(FailReason::Lost);
            }
            WsEvent::Closed => {
                if feed.status != Status::Failed(FailReason::Full) {
                    feed.status = Status::Failed(FailReason::Lost);
                }
            }
        }
    }
    socket
        .playback
        .advance(&mut feed, now, time.delta_secs_f64());
    if feed.status != Status::Live {
        outbox.0.clear();
        return;
    }
    for intention in outbox.0.drain(..) {
        send(&mut socket, &ClientMsg::Intention(intention));
    }
    if now - socket.last_ping >= PING_EVERY {
        socket.last_ping = now;
        send(&mut socket, &ClientMsg::Ping((now * 1000.0) as u32));
    }
}

fn receive(feed: &mut Feed, playback: &mut Playback, bytes: &[u8], now: f64) {
    match protocol::decode_server(bytes) {
        Ok(ServerMsg::Welcome(welcome)) if welcome.protocol == MOBA_PROTOCOL_VERSION => {
            feed.seat = Some(welcome.seat);
            feed.status = Status::Live;
        }
        Ok(ServerMsg::Welcome(_)) | Err(_) => feed.status = Status::Failed(FailReason::Lost),
        Ok(ServerMsg::World(world)) => playback.push(feed, world, now),
        Ok(ServerMsg::Private(me)) => feed.me = Some(me),
        Ok(ServerMsg::Rejected(reject)) => feed.rejects.push(reject),
        Ok(ServerMsg::Pong(_)) => {}
        Ok(ServerMsg::Full) => feed.status = Status::Failed(FailReason::Full),
    }
}

// Buffer two 20 Hz ticks. Follow the server clock at a bounded rate, as in
// BattleLine playback, so clustered packet arrivals do not restart a stride.
#[derive(Default)]
struct Playback {
    queue: VecDeque<WorldView>,
    transit: Option<f64>,
    tick: f64,
}

impl Playback {
    fn push(&mut self, feed: &mut Feed, world: WorldView, now: f64) {
        let newest = self.queue.back().or(feed.curr.as_ref());
        if newest.is_some_and(|view| world.tick <= view.tick) {
            return;
        }
        let sample = now * f64::from(TICK_HZ) - f64::from(world.tick);
        self.transit = Some(self.transit.map_or(sample, |base| {
            if sample < base {
                sample
            } else {
                base + (sample - base) * 0.1
            }
        }));
        if feed.curr.is_none() {
            self.tick = f64::from(world.tick);
            feed.push(world);
        } else {
            self.queue.push_back(world);
        }
        // Bound memory even when the application resumes after a long stall.
        while self.queue.len() > 32 {
            if let Some(world) = self.queue.pop_front() {
                self.tick = f64::from(world.tick);
                feed.push(world);
            }
        }
    }

    fn advance(&mut self, feed: &mut Feed, now: f64, delta: f64) {
        let Some(current) = self.queue.back().or(feed.curr.as_ref()) else {
            return;
        };
        let newest = f64::from(current.tick);
        let delay = if current.outcome.is_some() { 0.0 } else { 2.0 };
        let target = now * f64::from(TICK_HZ) - self.transit.unwrap_or_default() - delay;
        let step = delta * f64::from(TICK_HZ);
        let nominal = self.tick + step;
        let correction = (target - nominal).clamp(-step * 0.1, step * 0.1);
        if target >= self.tick {
            self.tick = (nominal + correction).min(target).min(newest);
        }
        while feed
            .curr
            .as_ref()
            .is_some_and(|view| f64::from(view.tick) < self.tick)
        {
            let Some(view) = self.queue.pop_front() else {
                break;
            };
            feed.push(view);
        }
        feed.alpha = match (&feed.prev, &feed.curr) {
            (Some(from), Some(to)) if to.tick > from.tick => ((self.tick - f64::from(from.tick))
                / f64::from(to.tick - from.tick))
            .clamp(0.0, 1.0) as f32,
            _ => 1.0,
        };
    }
}
