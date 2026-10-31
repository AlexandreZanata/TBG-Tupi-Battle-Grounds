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
        PROTOCOL_VERSION, 22,
        "v22: CorrosiveSplash (D-70); v21: UnitView.reduction (D-68); v20: seven Factions widen the Synergy arrays (D-64); v19: UnitView.shield (D-63); v18: UnitView.carrier (D-61)"
    );
}

#[test]
fn v8_arcana_messages_are_appended_and_v11_resizes_the_preset() {
    // D-40/P40: appended after every v7 variant (client 13; server 18 and 19).
    let set = encode(&ClientMsg::SetArcanas {
        ranks: tbg_core::ArcanaPreset::DEFAULT.ranks,
    });
    assert_eq!(set, [13, 2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    assert!(decode_client(&set).is_ok());
    // A malformed or short rank list is a decode error, never a panic.
    assert!(decode_client(&[13, 2, 0, 2]).is_err());
    let selected = ServerMsg::ArcanasSelected {
        preset: tbg_core::ArcanaPreset::DEFAULT,
    };
    assert_eq!(
        encode(&selected),
        [18, 2, 2, 2, 2, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    );
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
            ranks: tbg_core::ArcanaPreset::DEFAULT.ranks,
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
fn v10_augment_intentions_are_appended() {
    let choose = ClientMsg::Intent {
        seq: 7,
        intent: PlayerIntention::ChooseAugment {
            round: 11,
            option: 3,
        },
    };
    let bytes = encode(&choose);
    assert_eq!(bytes[0], 5, "Intent keeps its tag");
    assert_eq!(decode_client(&bytes).as_ref(), Ok(&choose));
    let forgo = ClientMsg::Intent {
        seq: 8,
        intent: PlayerIntention::Forgo { round: 12 },
    };
    assert_eq!(decode_client(&encode(&forgo)).as_ref(), Ok(&forgo));
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

#[test]
fn v17_spectating_messages_are_appended() {
    // P47 (BR-LV02): tags 18–19 on the client, 27–30 on the server, after
    // every earlier variant. A `LiveId` is a varint (258 = [0x82, 0x02]).
    assert_eq!(encode(&ClientMsg::Spectate { id: 258 }), [18, 0x82, 0x02]);
    assert_eq!(encode(&ClientMsg::StopSpectating), [19]);
    assert_eq!(
        encode(&ServerMsg::SpectateUnavailable {
            reason: tbg_core::protocol::SpectateRefusal::Full,
        }),
        [29, 2]
    );
    assert_eq!(encode(&ServerMsg::SpectateEnd { outcome: None }), [30, 0]);
    let view = tbg_core::MatchState::with_arcanas(1, [tbg_core::ArcanaPreset::NEUTRAL; 2])
        .public_view(&[], 5);
    let bytes = encode(&ServerMsg::SpectatorView(Box::new(view.clone())));
    assert_eq!(bytes[0], 28, "SpectatorView is the 28th server variant");
    assert_eq!(
        tbg_core::protocol::decode_server(&bytes),
        Ok(ServerMsg::SpectatorView(Box::new(view)))
    );
}

#[test]
fn v22_corrosive_splash_is_appended_and_round_trips() {
    // D-70: existing Event tags stay fixed; only tag 23 is appended.
    let event = tbg_core::Event::CorrosiveSplash {
        from: 1,
        primary: 2,
        to: 3,
    };
    let bytes = encode(&event);
    assert_eq!(bytes, [23, 1, 2, 3]);
    assert_eq!(postcard::from_bytes::<tbg_core::Event>(&bytes), Ok(event));
}
