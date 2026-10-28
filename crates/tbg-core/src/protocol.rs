//! Client/server messages (postcard over WebSocket binary frames).
//! Peers with a different `PROTOCOL_VERSION` are rejected in the handshake.
//! Intentions never carry a seat: the server derives it from the connection.

use serde::{Deserialize, Serialize};

use crate::intent::{PlayerIntention, Rejection};
use crate::state::Outcome;
use crate::types::Seat;
use crate::view::MatchView;

pub const PROTOCOL_VERSION: u16 = 5;

/// P12 (BR-R01): casual and ranked play are the same simulation; only the
/// queue and the rating change differ.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchMode {
    Casual,
    Ranked,
}
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
    // P11: appended at the end (v1 peers are rejected by version anyway).
    Register { name: String },
    Login { key: String },
    Logout,
    // P12: appended at the end (older peers are rejected by version).
    RankedMatch,
    // P13: one leaderboard page (`LEADERBOARD_PAGE` rows from `offset`).
    Leaderboard { offset: u16 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCode {
    BadMessage,
    RateLimited,
    TooManyConnections,
    NotInMatch,
    ServerFull,
    // P12: appended at the end (older peers are rejected by version).
    LoginRequired,
    // P14: sent to the old socket when the same account logs in elsewhere
    // (BR-P03), right before the server closes it.
    SessionTakenOver,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServerMsg {
    Welcome {
        token: Token,
        name: String,
    },
    VersionMismatch {
        server: u16,
    },
    Queued,
    RoomCreated {
        code: RoomCode,
    },
    RoomNotFound,
    MatchStart {
        you: Seat,
        opponent: String,
        // P12: the queue this match came from; `opponent_rating` is `Some`
        // only for ranked matches.
        mode: MatchMode,
        opponent_rating: Option<u16>,
    },
    View(MatchView),
    Rejected {
        seq: u32,
        reason: Rejection,
    },
    MatchEnd {
        outcome: Outcome,
    },
    Ping {
        id: u32,
    },
    NetStats {
        rtt_ms: u16,
    },
    Error {
        code: ErrorCode,
    },
    // P11: appended at the end (v1 peers are rejected by version anyway).
    Registered {
        key: String,
        profile: Profile,
    },
    LoggedIn {
        profile: Profile,
    },
    LoggedOut,
    AccountError {
        code: AccountError,
    },
    // P12: appended at the end (older peers are rejected by version).
    RatingChanged {
        before: u16,
        after: u16,
        profile: Profile,
    },
    // P13: appended at the end (older peers are rejected by version).
    // `you` is your rank when logged in with ≥ 1 ranked game.
    Leaderboard {
        total: u32,
        offset: u16,
        rows: Vec<LeaderRow>,
        you: Option<u32>,
    },
}

/// P13: rows per in-game leaderboard page.
pub const LEADERBOARD_PAGE: usize = 20;

/// P13 (BR-R07): one public leaderboard row (no account id).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeaderRow {
    pub rank: u32,
    pub name: String,
    pub rating: u16,
    pub wins: u32,
    pub losses: u32,
    pub draws: u32,
}

/// P11 (BR-P01–BR-P05): account failures. `BadKey` covers both unknown
/// name and wrong secret (no name oracle beyond `NameTaken` at register).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountError {
    NameInvalid,
    NameTaken,
    BadKey,
    RateLimited,
    Unavailable,
}

/// Public account summary. Rating columns exist from P11; ranked play (P12)
// fills them beyond the 1000/0 defaults.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub rating: u16,
    pub ranked_games: u32,
    pub wins: u32,
    pub losses: u32,
    pub draws: u32,
}

/// P12: settled ratings per seat `(before, after, profile)`, seats A then B.
pub type RatingResults = [(u16, u16, Profile); 2];

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
