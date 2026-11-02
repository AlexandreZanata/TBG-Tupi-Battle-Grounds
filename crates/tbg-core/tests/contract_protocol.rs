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
        PROTOCOL_VERSION, 34,
        "OG-01: append an eighth Faction; protocol 34 rejects older snapshot layouts, preserving message tags"
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

#[test]
fn v23_independent_ability_cues_are_appended() {
    // D-71: caster kind is the discriminator; primary/center/tick are public facts.
    let events = [
        tbg_core::Event::AbilityStarted {
            unit: 1,
            kind: tbg_core::UnitKind::Whisperer,
            primary: Some(2),
            center: 0,
            tick: 7,
        },
        tbg_core::Event::AbilityResolved {
            unit: 1,
            kind: tbg_core::UnitKind::Whisperer,
            primary: Some(2),
            center: 0,
            tick: 7,
        },
    ];
    for (event, tag) in events.into_iter().zip([24, 25]) {
        let bytes = encode(&event);
        assert_eq!(bytes, [tag, 1, 30, 1, 2, 0, 7]);
        assert_eq!(postcard::from_bytes::<tbg_core::Event>(&bytes), Ok(event));
    }
}

#[test]
fn v25_crossfire_is_appended() {
    // D-75: preserve all previous event tags and expose only the chosen secondary.
    let event = tbg_core::Event::Crossfire { from: 1, to: 3 };
    let bytes = encode(&event);
    assert_eq!(bytes, [26, 1, 3]);
    assert_eq!(postcard::from_bytes::<tbg_core::Event>(&bytes), Ok(event));
}

#[test]
fn v26_profane_flame_is_appended_with_exact_endpoints() {
    // D-87 appends tag27 and preserves every earlier byte contract.
    let event = tbg_core::Event::ProfaneFlame {
        unit: 1,
        start: 0,
        end: 0,
        tick: 7,
    };
    let bytes = encode(&event);
    assert_eq!(bytes, [27, 1, 0, 0, 7]);
    assert_eq!(postcard::from_bytes::<tbg_core::Event>(&bytes), Ok(event));
}

#[test]
fn v27_drill_fusion_is_appended_without_death() {
    // D-88: old tags remain unchanged; fusion has its own public cue.
    let event = tbg_core::Event::DrillFused {
        first: 1,
        second: 2,
        unit: 3,
        x: 0,
    };
    let bytes = encode(&event);
    assert_eq!(bytes, [28, 1, 2, 3, 0]);
    assert_eq!(postcard::from_bytes::<tbg_core::Event>(&bytes), Ok(event));
}

#[test]
fn d97_auth_variants_append_without_exposing_secrets_in_debug() {
    use tbg_core::protocol::{decode_server, AccessToken, AuthRefusal, AuthStatus};
    let secret = AccessToken([0xab; 32]);
    let messages = [
        ClientMsg::AuthenticatedHello {
            version: 30,
            resume: Some([7; 16]),
            access: secret,
        },
        ClientMsg::Reauthenticate { access: secret },
        ClientMsg::ClaimAccount { take_over: true },
    ];
    for (tag, message) in (20..=22).zip(messages) {
        let bytes = encode(&message);
        assert_eq!(bytes[0], tag);
        assert_eq!(decode_client(&bytes), Ok(message.clone()));
        assert!(!format!("{message:?}").contains("171"));
        assert!(decode_client(&bytes[..bytes.len() - 1]).is_err());
    }
    let states = [
        ServerMsg::AuthState {
            status: AuthStatus::Conflict,
            access_expires_at: 900,
            session_expires_at: 2_592_000,
        },
        ServerMsg::AuthRequired {
            reason: AuthRefusal::Revoked,
        },
    ];
    for (tag, message) in (31..=32).zip(states) {
        let bytes = encode(&message);
        assert_eq!(bytes[0], tag);
        assert_eq!(decode_server(&bytes), Ok(message));
    }
    for raw in [
        "ab".repeat(31),
        "AB".repeat(32),
        "zz".repeat(32),
        "ab".repeat(33),
    ] {
        assert!(AccessToken::from_hex(&raw).is_none());
    }
    assert_eq!(AccessToken::from_hex(&"ab".repeat(32)), Some(secret));
    for message in [
        ClientMsg::Login {
            key: "PRIVATE_RECOVERY_KEY".into(),
        },
        ClientMsg::Reauthenticate { access: secret },
    ] {
        assert!(!format!("{message:?}").contains("PRIVATE"));
    }
}

#[test]
fn v30_arena_identities_append_without_renumbering_the_original_five() {
    // GAME-DESIGN §12, BR-AR01/06: identical assignment on every peer and reconnect.
    use tbg_core::Arena;
    let order = [
        Arena::Forest,
        Arena::Tundra,
        Arena::Volcanic,
        Arena::Ruins,
        Arena::Storm,
        Arena::Celestial,
        Arena::Clockwork,
        Arena::Desert,
    ];
    for (id, arena) in order.into_iter().enumerate() {
        let bytes = postcard::to_allocvec(&arena).expect("Arena wire identity");
        assert_eq!(bytes, [id as u8]);
        assert_eq!(postcard::from_bytes::<Arena>(&bytes), Ok(arena));
    }
}

#[test]
fn p73_appends_participant_bound_messages_and_keeps_legacy_match_end_bytes() {
    use tbg_core::protocol::{decode_server, HistorySaveState, PlayerInviteOutcome};
    for (tag, message) in [
        (
            23,
            ClientMsg::ChallengePlayer {
                name: "Alpha".into(),
            },
        ),
        (
            24,
            ClientMsg::AnswerPlayerChallenge {
                id: [1; 16],
                accept: true,
            },
        ),
        (25, ClientMsg::CancelPlayerChallenge { id: [1; 16] }),
        (26, ClientMsg::SetPlayerChallengesEnabled { enabled: false }),
    ] {
        let bytes = encode(&message);
        assert_eq!(bytes[0], tag);
        assert_eq!(decode_client(&bytes).unwrap(), message);
    }
    for (tag, message) in [
        (
            33,
            ServerMsg::PlayerChallengeState {
                enabled: false,
                outgoing: None,
                incoming: Vec::new(),
            },
        ),
        (
            34,
            ServerMsg::PlayerChallengeResult {
                id: Some([1; 16]),
                outcome: PlayerInviteOutcome::Cancelled,
            },
        ),
        (
            36,
            ServerMsg::MatchHistoryStatus {
                result_id: [2; 16],
                entry_id: [3; 16],
                state: HistorySaveState::Pending,
            },
        ),
    ] {
        let bytes = encode(&message);
        assert_eq!(bytes[0], tag);
        assert_eq!(decode_server(&bytes).unwrap(), message);
    }
}

#[test]
fn cc_id01_v32_completion_facts_and_short_cast_roundtrip_without_reordering_old_kind_tags() {
    use tbg_core::{completion::state::View, state::PendingCast, view::SpecialView, UnitKind};
    assert_eq!(encode(&UnitKind::Megadrill), [50]);
    assert_eq!(encode(&UnitKind::Gravecutter), [51]);
    assert_eq!(encode(&UnitKind::Stridulator), [67]);
    let facts = View {
        echo_ticks: 80,
        regrowth_pulses: 4,
        spark_ticks: 60,
        interrupt_ticks: 80,
        charged: true,
        channel_target: Some(23),
        channel_pulses: 3,
        beam_center: Some(5200),
    };
    let special = SpecialView {
        ready_in: 0,
        cast: Some(PendingCast {
            target: Some(23),
            left: 2,
        }),
        windup: 2,
    };
    let bytes = encode(&(facts, special));
    assert!(
        bytes.len() < 64,
        "bounded reconnect facts must stay compact"
    );
    assert_eq!(
        postcard::from_bytes::<(View, SpecialView)>(&bytes),
        Ok((facts, special))
    );
}

#[test]
fn og01_new_faction_and_kind_tags_are_appended() {
    assert_eq!(encode(&tbg_core::Faction::Ogres), [7]);
    assert_eq!(encode(&tbg_core::UnitKind::Stridulator), [67]);
    assert_eq!(encode(&tbg_core::UnitKind::BasaltBulwark), [68]);
    assert_eq!(encode(&tbg_core::UnitKind::RivetBallista), [76]);
}
