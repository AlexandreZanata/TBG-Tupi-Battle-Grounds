//! Guard tests for the wire protocol (ARCHITECTURE §5). postcard encodes an
//! enum variant as its index, so the order of every variant is part of the
//! contract: new variants are only ever appended, and any change to the
//! bytes below needs a `PROTOCOL_VERSION` bump in the same commit (older
//! clients are then refused by the handshake instead of misreading frames).

use tbg_core::protocol::{
    decode_client, encode, AccountError, ClientMsg, ErrorCode, MatchMode, ServerMsg,
    PROTOCOL_VERSION,
};
use tbg_core::PlayerIntention;

#[test]
fn protocol_version() {
    assert_eq!(
        PROTOCOL_VERSION, 9,
        "v9 (D-39/P39): challenge/rematch appended after D-40 Arcanas"
    );
}

#[test]
fn v8_arcana_messages_are_appended() {
    // D-40: appended after every v7 variant (client 13; server 18 and 19).
    let set = encode(&ClientMsg::SetArcanas {
        ranks: [2, 0, 2, 0, 2, 0],
    });
    assert_eq!(set, [13, 2, 0, 2, 0, 2, 0]);
    assert!(decode_client(&set).is_ok());
    // A malformed rank list is a decode error, never a panic.
    assert!(decode_client(&[13, 2, 0, 2]).is_err());
    let selected = ServerMsg::ArcanasSelected {
        preset: tbg_core::ArcanaPreset::DEFAULT,
    };
    assert_eq!(encode(&selected), [18, 2, 0, 2, 0, 2, 0]);
    let rejected = ServerMsg::ArcanasRejected {
        reason: tbg_core::protocol::ArcanaRejection::NotIdle,
    };
    assert_eq!(encode(&rejected), [19, 1]);
}

#[test]
fn client_variant_order_is_frozen() {
    let order = [
        ClientMsg::Hello {
            version: PROTOCOL_VERSION,
            resume: None,
        },
        ClientMsg::QuickMatch,
        ClientMsg::CreateRoom,
        ClientMsg::JoinRoom { code: *b"ABCDE" },
        ClientMsg::Cancel,
        ClientMsg::Intent {
            seq: 1,
            intent: PlayerIntention::Purchase { offer: 0 },
        },
        ClientMsg::Pong { id: 1 },
        ClientMsg::Register { name: "Abc".into() },
        ClientMsg::Login {
            key: "Abc-0000-0000-0000-0000".into(),
        },
        ClientMsg::Logout,
        ClientMsg::RankedMatch,
        ClientMsg::Leaderboard { offset: 0 },
        ClientMsg::Surrender,
        ClientMsg::SetArcanas {
            ranks: [2, 0, 2, 0, 2, 0],
        },
        ClientMsg::CreateChallenge,
        ClientMsg::JoinChallenge { token: [7; 16] },
        ClientMsg::RequestRematch,
    ];
    for (tag, msg) in order.iter().enumerate() {
        let bytes = encode(msg);
        assert_eq!(usize::from(bytes[0]), tag, "{msg:?}");
        assert_eq!(
            decode_client(&bytes).as_ref(),
            Ok(msg),
            "round trip {msg:?}"
        );
    }
}

#[test]
fn client_bytes_are_frozen() {
    // Handshake and the small messages a stale client might still send.
    assert_eq!(
        encode(&ClientMsg::Hello {
            version: 6,
            resume: None
        }),
        [0, 6, 0]
    );
    assert_eq!(encode(&ClientMsg::Pong { id: 300 }), [6, 0xAC, 0x02]);
    assert_eq!(encode(&ClientMsg::Leaderboard { offset: 20 }), [11, 20]);
    assert_eq!(encode(&ClientMsg::Surrender), [12]);
    assert_eq!(encode(&ClientMsg::CreateChallenge), [14]);
    assert_eq!(
        encode(&ClientMsg::JoinChallenge { token: [7; 16] }),
        [15, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7]
    );
    assert_eq!(encode(&ClientMsg::RequestRematch), [16]);
}

#[test]
fn match_mode_order_is_frozen() {
    assert_eq!(encode(&MatchMode::Casual), [0]);
    assert_eq!(encode(&MatchMode::Ranked), [1]);
    assert_eq!(encode(&MatchMode::Challenge), [2]);
}

#[test]
fn error_code_order_is_frozen() {
    // Clients branch on these (LoginRequired, SessionTakenOver).
    let codes = [
        ErrorCode::BadMessage,
        ErrorCode::RateLimited,
        ErrorCode::TooManyConnections,
        ErrorCode::NotInMatch,
        ErrorCode::ServerFull,
        ErrorCode::LoginRequired,
        ErrorCode::SessionTakenOver,
    ];
    for (i, code) in codes.into_iter().enumerate() {
        assert_eq!(
            encode(&ServerMsg::Error { code }),
            [11, i as u8],
            "{code:?}"
        );
    }
    let account = [
        AccountError::NameInvalid,
        AccountError::NameTaken,
        AccountError::BadKey,
        AccountError::RateLimited,
        AccountError::Unavailable,
    ];
    for (i, code) in account.into_iter().enumerate() {
        assert_eq!(
            encode(&ServerMsg::AccountError { code }),
            [15, i as u8],
            "{code:?}"
        );
    }
}
