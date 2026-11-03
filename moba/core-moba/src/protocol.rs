//! MOBA wire protocol v1 (immutable once merged; `docs/moba/CONTRACTS.md` §3).
//!
//! Binary postcard frames over WebSocket. The world snapshot is identical
//! for every player and encoded once per tick; private economy goes to its
//! owner only.

use serde::{Deserialize, Serialize};

use crate::content::OFFERS;
use crate::intent::{MobaIntention, Reject};
use crate::state::{Event, Outcome};
use crate::types::{Champion, Destination, Seat, Side, UnitKind};

pub const MOBA_PROTOCOL_VERSION: u16 = 1;
/// Largest client frame accepted; anything bigger closes the socket (M-40).
pub const MAX_CLIENT_FRAME: usize = 256;
/// Largest server frame a client accepts.
pub const MAX_SERVER_FRAME: usize = 256 * 1024;
pub const MAX_NAME_CHARS: usize = 24;
/// Client frames allowed per second before the server drops the connection.
pub const MAX_CLIENT_FRAMES_PER_SECOND: u32 = 30;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClientMsg {
    Hello { protocol: u16, name: String },
    Intention(MobaIntention),
    Ping(u32),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServerMsg {
    Welcome(Welcome),
    World(WorldView),
    Private(PrivateView),
    Rejected(Reject),
    Pong(u32),
    /// Every seat is taken by a human.
    Full,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Welcome {
    pub protocol: u16,
    pub seat: Seat,
    pub seed: u64,
    pub tick_hz: u8,
}

/// Public state of one tick, the same bytes for every connection.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldView {
    pub tick: u32,
    pub score: [u16; 2],
    pub towers_left: [u8; 2],
    pub team_gold: [u32; 2],
    pub seats: Vec<SeatView>,
    pub units: Vec<UnitView>,
    pub events: Vec<Event>,
    pub outcome: Option<Outcome>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeatView {
    pub human: bool,
    pub level: u8,
    pub hp_permille: u16,
    /// Ticks until the Commander respawns, 0 while alive.
    pub respawn_ticks: u16,
    pub kills: u16,
    pub deaths: u16,
    pub commander: u32,
}

/// Positions are centi-units (`x / 100` = world units).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitView {
    pub id: u32,
    pub kind: UnitKind,
    pub side: Side,
    pub owner: Option<Seat>,
    pub x: u16,
    pub y: u16,
    pub hp_permille: u16,
    pub target: u32,
    /// An attack or heal left this unit on this tick.
    pub attacked: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivateView {
    pub seat: Seat,
    pub gold: u32,
    pub essence: u32,
    pub tier: u8,
    pub troops: u8,
    pub destination: Destination,
    pub commander_auto: bool,
    /// The seat's five Shop offers and locked-slot bits.
    pub offers: [Champion; OFFERS],
    pub locked: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WireError {
    TooLarge,
    Malformed,
    BadName,
}

pub fn encode<T: Serialize>(message: &T) -> Vec<u8> {
    // Serializing owned plain data into a Vec cannot fail.
    postcard::to_allocvec(message).unwrap_or_default()
}

/// Decodes an untrusted client frame; never panics (M-40).
pub fn decode_client(bytes: &[u8]) -> Result<ClientMsg, WireError> {
    if bytes.len() > MAX_CLIENT_FRAME {
        return Err(WireError::TooLarge);
    }
    let message: ClientMsg = postcard::from_bytes(bytes).map_err(|_| WireError::Malformed)?;
    if let ClientMsg::Hello { name, .. } = &message {
        if name.chars().count() > MAX_NAME_CHARS || name.chars().any(char::is_control) {
            return Err(WireError::BadName);
        }
    }
    Ok(message)
}

pub fn decode_server(bytes: &[u8]) -> Result<ServerMsg, WireError> {
    if bytes.len() > MAX_SERVER_FRAME {
        return Err(WireError::TooLarge);
    }
    postcard::from_bytes(bytes).map_err(|_| WireError::Malformed)
}
