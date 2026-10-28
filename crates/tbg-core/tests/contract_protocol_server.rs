//! Guard tests for the server half of the wire protocol (ARCHITECTURE §5):
//! every `ServerMsg` variant keeps its index, and the frames clients parse
//! most (`MatchStart`, `MatchEnd`, `Leaderboard`) keep their exact bytes.
//! Changing any of them needs a `PROTOCOL_VERSION` bump in the same commit.

use tbg_core::protocol::{
    decode_server, encode, AccountError, ErrorCode, LeaderRow, MatchMode, Profile, ServerMsg,
    PROTOCOL_VERSION,
};
use tbg_core::{EndReason, MatchState, Outcome, Rejection, Seat};

fn profile() -> Profile {
    Profile {
        name: "Ann".into(),
        rating: 1000,
        ranked_games: 0,
        wins: 0,
        losses: 0,
        draws: 0,
    }
}

#[test]
fn server_variant_order_is_frozen() {
    let view = MatchState::new(1).view_for(Seat::A, &[]);
    let outcome = Outcome {
        winner: Some(Seat::B),
        reason: EndReason::Forfeit,
    };
    let order = [
        ServerMsg::Welcome {
            token: [7; 16],
            name: "Commander 1000".into(),
        },
        ServerMsg::VersionMismatch {
            server: PROTOCOL_VERSION,
        },
        ServerMsg::Queued,
        ServerMsg::RoomCreated { code: *b"ABCDE" },
        ServerMsg::RoomNotFound,
        ServerMsg::MatchStart {
            you: Seat::A,
            you_name: "Ann".into(),
            you_rating: None,
            opponent: "Bo".into(),
            mode: MatchMode::Casual,
            opponent_rating: None,
        },
        ServerMsg::View(view),
        ServerMsg::Rejected {
            seq: 1,
            reason: Rejection::NotEnoughEnergy,
        },
        ServerMsg::MatchEnd { outcome },
        ServerMsg::Ping { id: 1 },
        ServerMsg::NetStats { rtt_ms: 40 },
        ServerMsg::Error {
            code: ErrorCode::BadMessage,
        },
        ServerMsg::Registered {
            key: "Ann-0000-0000-0000-0000".into(),
            profile: profile(),
        },
        ServerMsg::LoggedIn { profile: profile() },
        ServerMsg::LoggedOut,
        ServerMsg::AccountError {
            code: AccountError::BadKey,
        },
        ServerMsg::RatingChanged {
            before: 1000,
            after: 1020,
            profile: profile(),
        },
        ServerMsg::Leaderboard {
            total: 0,
            offset: 0,
            rows: Vec::new(),
            you: None,
        },
    ];
    for (tag, msg) in order.iter().enumerate() {
        let bytes = encode(msg);
        assert_eq!(usize::from(bytes[0]), tag, "{msg:?}");
        assert_eq!(
            decode_server(&bytes).as_ref(),
            Ok(msg),
            "round trip {msg:?}"
        );
    }
}

#[test]
fn server_bytes_are_frozen() {
    // MatchStart field order: you, you_name, you_rating, opponent, mode,
    // opponent_rating (1000 = varint E8 07).
    let start = ServerMsg::MatchStart {
        you: Seat::B,
        you_name: "Ann".into(),
        you_rating: Some(1000),
        opponent: "Bo".into(),
        mode: MatchMode::Ranked,
        opponent_rating: None,
    };
    assert_eq!(
        encode(&start),
        [5, 1, 3, b'A', b'n', b'n', 1, 0xE8, 0x07, 2, b'B', b'o', 1, 0]
    );
    let end = ServerMsg::MatchEnd {
        outcome: Outcome {
            winner: Some(Seat::B),
            reason: EndReason::Forfeit,
        },
    };
    assert_eq!(encode(&end), [8, 1, 1, 2]);
    let row = LeaderRow {
        rank: 1,
        name: "Ann".into(),
        rating: 1020,
        wins: 1,
        losses: 0,
        draws: 0,
    };
    let board = ServerMsg::Leaderboard {
        total: 1,
        offset: 0,
        rows: vec![row],
        you: Some(1),
    };
    assert_eq!(
        encode(&board),
        [17, 1, 0, 1, 1, 3, b'A', b'n', b'n', 0xFC, 0x07, 1, 0, 0, 1, 1]
    );
}
