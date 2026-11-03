//! Online play: one WebSocket to `tbg-moba-server`; the server owns every outcome.

use bevy::prelude::*;
use ewebsock::{WsEvent, WsMessage, WsReceiver, WsSender};
use tbg_moba_core::protocol::{self, ClientMsg, ServerMsg, MOBA_PROTOCOL_VERSION};

use super::{FailReason, Feed, Outbox, Status};

/// Seconds between keep-alive pings (the server closes silent sockets after 30 s).
const PING_EVERY: f64 = 2.0;

pub struct Socket {
    sender: WsSender,
    receiver: WsReceiver,
    last_ping: f64,
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
            WsEvent::Message(WsMessage::Binary(bytes)) => receive(&mut feed, &bytes, now),
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

fn receive(feed: &mut Feed, bytes: &[u8], now: f64) {
    match protocol::decode_server(bytes) {
        Ok(ServerMsg::Welcome(welcome)) if welcome.protocol == MOBA_PROTOCOL_VERSION => {
            feed.seat = Some(welcome.seat);
            feed.status = Status::Live;
        }
        Ok(ServerMsg::Welcome(_)) | Err(_) => feed.status = Status::Failed(FailReason::Lost),
        Ok(ServerMsg::World(world)) => feed.push(world, now),
        Ok(ServerMsg::Private(me)) => feed.me = Some(me),
        Ok(ServerMsg::Rejected(reject)) => feed.rejects.push(reject),
        Ok(ServerMsg::Pong(_)) => {}
        Ok(ServerMsg::Full) => feed.status = Status::Failed(FailReason::Full),
    }
}
