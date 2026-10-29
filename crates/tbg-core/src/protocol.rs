//! Client/server messages (postcard over WebSocket binary frames).
//! Peers with a different `PROTOCOL_VERSION` are rejected in the handshake.
//! Intentions never carry a seat: the server derives it from the connection.

use serde::{Deserialize, Serialize};

use crate::arcana::{self, ArcanaPreset};
use crate::intent::{PlayerIntention, Rejection};
use crate::state::Outcome;
use crate::types::Seat;
use crate::view::MatchView;

/// v11: the fifteen-node Arcana schema (12 points, prerequisites, 2-point
/// supremes); the `SetArcanas` rank vector grew, so older peers are refused.
pub const PROTOCOL_VERSION: u16 = 11;

/// P12 (BR-R01): casual and ranked play are the same simulation; only the
/// queue and the rating change differ. P39 (BR-R08): `Challenge` is a
/// direct invitation match with casual rules and no rating change.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchMode {
    Casual,
    Ranked,
    Challenge,
}
pub const MAX_CLIENT_MSG_BYTES: usize = 1024;
/// Room-code alphabet: no 0/O, 1/I/L.
pub const ROOM_ALPHABET: &[u8] = b"ABCDEFGHJKMNPQRSTUVWXYZ23456789";

pub type Token = [u8; 16];
/// P39: a single-use Quick Challenge invitation. Independent of `Token`.
pub type InviteToken = [u8; 16];
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
    // Post-P14 (BR-M06, Proposed): give up while the match is live.
    Surrender,
    // v8 (D-40, BR-AR05/BR-AR06): choose Arcanas while idle; the server
    // validates and answers `ArcanasSelected` or `ArcanasRejected`.
    // v11: fifteen ranks (P40); the seat still comes from the connection.
    SetArcanas { ranks: [u8; arcana::NODES] },
    // v9 (P39): Quick Challenge by link and both-accept rematch.
    CreateChallenge,
    JoinChallenge { token: InviteToken },
    RequestRematch,
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
        // Post-P14: the HUD puts a name next to each HP bar; ratings ride
        // along only in ranked matches.
        you_name: String,
        you_rating: Option<u16>,
        opponent: String,
        // P12: the queue this match came from; `opponent_rating` is `Some`
        // only for ranked matches.
        mode: MatchMode,
        opponent_rating: Option<u16>,
        /// v8 (D-40): both frozen presets, indexed by `Seat::index()`;
        /// public once paired (BR-AR05, BR-AR07).
        arcanas: [ArcanaPreset; 2],
    },
    /// Boxed: a view is far larger than every other message.
    View(Box<MatchView>),
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
    // v8 (D-40): the authoritative selection (also sent after login when
    // the account has a saved preset, BR-AR08).
    ArcanasSelected {
        preset: ArcanaPreset,
    },
    ArcanasRejected {
        reason: ArcanaRejection,
    },
    // v9 (P39): Quick Challenge by link and both-accept rematch.
    ChallengeWaiting {
        token: InviteToken,
    },
    ChallengeUnavailable,
    RematchOpen {
        opponent: String,
    },
    RematchState {
        you_requested: bool,
        opponent_requested: bool,
    },
    RematchClosed {
        declined: bool,
    },
}

/// v8 (D-40): why a `SetArcanas` was refused; the previous preset stays.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArcanaRejection {
    /// Any invalid tree: wrong budget, rank above its node maximum or an
    /// unmet prerequisite (BR-AR02/AR03).
    BadRanks,
    /// Queued, in a room, or in a match: the preset is frozen (BR-AR05).
    NotIdle,
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
