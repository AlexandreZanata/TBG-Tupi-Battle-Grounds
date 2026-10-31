//! Verify a native TLS/WebSocket handshake without entering a public queue.

use ewebsock::{WsEvent, WsMessage};
use std::time::{Duration, Instant};
use tbg_core::protocol::{self, ClientMsg, ServerMsg, PROTOCOL_VERSION};

fn main() -> Result<(), String> {
    let url = std::env::args()
        .nth(1)
        .ok_or("Pass a ws:// or wss:// server URL")?;
    let (mut sender, receiver) = ewebsock::connect(&url, ewebsock::Options::default())?;
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        while let Some(event) = receiver.try_recv() {
            match event {
                WsEvent::Opened => {
                    sender.send(WsMessage::Binary(protocol::encode(&ClientMsg::Hello {
                        version: PROTOCOL_VERSION,
                        resume: None,
                    })))
                }
                WsEvent::Message(WsMessage::Binary(bytes)) => match protocol::decode_server(&bytes)
                {
                    Ok(ServerMsg::Welcome { .. }) => {
                        println!("Native handshake OK · protocol {PROTOCOL_VERSION}");
                        return Ok(());
                    }
                    Ok(ServerMsg::VersionMismatch { server }) => {
                        return Err(format!(
                            "Server protocol {server}; client protocol {PROTOCOL_VERSION}"
                        ))
                    }
                    _ => {}
                },
                WsEvent::Error(error) => return Err(error),
                WsEvent::Closed => return Err("Connection closed before Welcome".into()),
                _ => {}
            }
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    Err("Handshake timed out".into())
}
