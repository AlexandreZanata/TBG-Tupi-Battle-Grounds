//! Client/server messages (postcard over WebSocket binary frames).
//! Peers with a different `PROTOCOL_VERSION` are rejected in the handshake.
//! Intentions never carry a seat: the server derives it from the connection.

use serde::{Deserialize, Serialize};

use crate::intent::{PlayerIntention, Rejection};
use crate::state::Outcome;
use crate::types::Seat;
use crate::view::MatchView;

pub const PROTOCOL_VERSION: u16 = 1;
pub const MAX_CLIENT_MSG_BYTES: usize = 1024;
/// Room-code alphabet: no 0/O, 1/I/L.
pub const ROOM_ALPHABET: &[u8] = b"ABCDEFGHJKMNPQRSTUVWXYZ23456789";

pub type Token = [u8; 16];
pub type RoomCode = [u8; 5];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClientMsg {
    Hello { version: u16, resume: Option<Token> },
    QuickMatch,
    CreateRoom,
    JoinRoom { code: RoomCode },
    Cancel,
    Intent { seq: u32, intent: PlayerIntention },
    Pong { id: u32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCode {
    BadMessage,
    RateLimited,
    TooManyConnections,
    NotInMatch,
    ServerFull,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServerMsg {
    Welcome { token: Token, name: String },
    VersionMismatch { server: u16 },
    Queued,
    RoomCreated { code: RoomCode },
    RoomNotFound,
    MatchStart { you: Seat, opponent: String },
    View(MatchView),
    Rejected { seq: u32, reason: Rejection },
    MatchEnd { outcome: Outcome },
    Ping { id: u32 },
    NetStats { rtt_ms: u16 },
    Error { code: ErrorCode },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DecodeError;

pub fn encode<T: Serialize>(msg: &T) -> Vec<u8> {
    // Serializing these plain types into a Vec cannot fail.
    postcard::to_allocvec(msg).unwrap_or_default()
}

/// Decode untrusted client bytes: size-limited, no trailing bytes.
pub fn decode_client(bytes: &[u8]) -> Result<ClientMsg, DecodeError> {
    if bytes.len() > MAX_CLIENT_MSG_BYTES {
        return Err(DecodeError);
    }
    decode_exact(bytes)
}

pub fn decode_server(bytes: &[u8]) -> Result<ServerMsg, DecodeError> {
    decode_exact(bytes)
}

fn decode_exact<T: for<'de> Deserialize<'de>>(bytes: &[u8]) -> Result<T, DecodeError> {
    match postcard::take_from_bytes(bytes) {
        Ok((msg, [])) => Ok(msg),
        _ => Err(DecodeError),
    }
}

/// Room codes are 5 characters from `ROOM_ALPHABET`.
pub fn valid_room_code(code: &RoomCode) -> bool {
    code.iter().all(|c| ROOM_ALPHABET.contains(c))
}
