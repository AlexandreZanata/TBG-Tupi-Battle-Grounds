//! D-97 A09/A10: passive restore, seat leases, renewal and acknowledged security revocation.
#[path = "common/auth.rs"]
mod auth;
use futures_util::StreamExt;
use serde_json::json;
use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};
use tbg_core::protocol::{
    self, AccessToken, AuthRefusal, AuthStatus, ClientMsg, ServerMsg, PROTOCOL_VERSION,
};
use tbg_server::{auth::Clock, botclient::native_auth, Config};
static NEXT: AtomicU64 = AtomicU64::new(1);
async fn server() -> (String, Clock) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}/ws", listener.local_addr().unwrap());
    let db = std::env::temp_dir().join(format!(
        "tbg-p68-ws-{}-{}.sqlite",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let clock = Clock::controlled(1000);
    let config = Config {
        db: Some(db),
        auth_clock: clock.clone(),
        tick_ms: 20,
        max_per_ip: 100,
        ..Config::default()
    };
    tokio::spawn(tbg_server::run(config, listener));
    (url, clock)
}
async fn recv(ws: &mut auth::Ws) -> ServerMsg {
    loop {
        let event = tokio::time::timeout(Duration::from_secs(5), ws.next())
            .await
            .expect("socket response timeout")
            .expect("socket closed")
            .unwrap();
        let tokio_tungstenite::tungstenite::Message::Binary(bytes) = event else {
            continue;
        };
        let message = protocol::decode_server(&bytes).unwrap();
        match message {
            ServerMsg::Ping { id } => auth::send(ws, ClientMsg::Pong { id }).await,
            ServerMsg::NetStats { .. } => {}
            other => return other,
        }
    }
}
async fn until(ws: &mut auth::Ws, predicate: impl Fn(&ServerMsg) -> bool) -> ServerMsg {
    for _ in 0..200 {
        let message = recv(ws).await;
        if predicate(&message) {
            return message;
        }
    }
    panic!("expected socket state not reached");
}
async fn connect(
    url: &str,
    resume: Option<protocol::Token>,
    access: Option<AccessToken>,
) -> (auth::Ws, protocol::Token) {
    let mut ws = tokio_tungstenite::connect_async(url).await.unwrap().0;
    let message = match access {
        Some(access) => ClientMsg::AuthenticatedHello {
            version: PROTOCOL_VERSION,
            resume,
            access,
        },
        None => ClientMsg::Hello {
            version: PROTOCOL_VERSION,
            resume,
        },
    };
    auth::send(&mut ws, message).await;
    let ServerMsg::Welcome { token, .. } =
        until(&mut ws, |m| matches!(m, ServerMsg::Welcome { .. })).await
    else {
        unreachable!()
    };
    (ws, token)
}
#[tokio::test]
async fn a09_same_family_passive_resume_never_displaces_an_attached_controller() {
    let (url, _) = server().await;
    let (mut original, token) = connect(&url, None, None).await;
    let grant = auth::register(&original, "PassiveOwner").await.unwrap();
    auth::attach(&mut original, &grant, false).await;
    until(&mut original, |m| {
        matches!(
            m,
            ServerMsg::AuthState {
                status: AuthStatus::Active,
                ..
            }
        )
    })
    .await;
    let (mut tab, fresh) = connect(&url, Some(token), Some(grant.access)).await;
    assert_ne!(fresh, token);
    until(&mut tab, |m| {
        matches!(
            m,
            ServerMsg::AuthState {
                status: AuthStatus::Conflict,
                ..
            }
        )
    })
    .await;
    auth::send(&mut original, ClientMsg::Leaderboard { offset: 0 }).await;
    until(&mut original, |m| {
        matches!(m, ServerMsg::Leaderboard { .. })
    })
    .await;
    auth::send(&mut tab, ClientMsg::ClaimAccount { take_over: true }).await;
    until(&mut tab, |m| {
        matches!(
            m,
            ServerMsg::AuthState {
                status: AuthStatus::Active,
                ..
            }
        )
    })
    .await;
    until(&mut original, |m| {
        matches!(
            m,
            ServerMsg::Error {
                code: protocol::ErrorCode::SessionTakenOver
            }
        )
    })
    .await;
}
/// P71 (BR-M03, BR-AS06): a network drop TCP never reports leaves the controller
/// attached; the same session's own resume replaces the silent socket and keeps
/// the seat instead of opening a Conflict whose new session would forfeit it.
#[tokio::test]
async fn p71_a_silent_controller_is_replaced_by_its_own_resume() {
    let (url, _) = server().await;
    let (mut player, token) = connect(&url, None, None).await;
    let grant = auth::register(&player, "SilentSeat").await.unwrap();
    auth::attach(&mut player, &grant, false).await;
    until(&mut player, |m| {
        matches!(
            m,
            ServerMsg::AuthState {
                status: AuthStatus::Active,
                ..
            }
        )
    })
    .await;
    let (mut rival, _) = connect(&url, None, None).await;
    auth::send(&mut player, ClientMsg::QuickMatch).await;
    auth::send(&mut rival, ClientMsg::QuickMatch).await;
    let ServerMsg::MatchStart { you, .. } =
        until(&mut player, |m| matches!(m, ServerMsg::MatchStart { .. })).await
    else {
        unreachable!()
    };
    // The link dies silently: nothing is read or sent past the 3 s limit.
    tokio::time::sleep(Duration::from_millis(3500)).await;
    let (mut back, resumed) = connect(&url, Some(token), Some(grant.access)).await;
    assert_eq!(resumed, token, "the same session resumes, no new one");
    let ServerMsg::MatchStart { you: again, .. } =
        until(&mut back, |m| matches!(m, ServerMsg::MatchStart { .. })).await
    else {
        unreachable!()
    };
    assert_eq!(again, you, "the same seat");
    until(&mut back, |m| matches!(m, ServerMsg::View(_))).await;
    // The replaced socket is closed by the server, not left half-attached.
    loop {
        match tokio::time::timeout(Duration::from_secs(5), player.next()).await {
            Ok(Some(Ok(tokio_tungstenite::tungstenite::Message::Close(_))))
            | Ok(Some(Err(_)))
            | Ok(None) => break,
            Ok(Some(Ok(_))) => continue,
            Err(_) => panic!("the silent socket stayed open"),
        }
    }
    drop(rival);
}
/// P71 (BR-AS07): a drop that outlasts the Access token still takes the admitted
/// seat back with that expired token (the seat lease), then renews on the socket;
/// the same expired token cannot open anything else.
#[tokio::test]
async fn p71_br_as07_an_expired_access_takes_back_its_own_seat_only() {
    let (url, clock) = server().await;
    let (mut player, token) = connect(&url, None, None).await;
    let grant = auth::register(&player, "LeaseBack").await.unwrap();
    auth::attach(&mut player, &grant, false).await;
    until(&mut player, |m| {
        matches!(
            m,
            ServerMsg::AuthState {
                status: AuthStatus::Active,
                ..
            }
        )
    })
    .await;
    let (mut rival, _) = connect(&url, None, None).await;
    auth::send(&mut player, ClientMsg::QuickMatch).await;
    auth::send(&mut rival, ClientMsg::QuickMatch).await;
    let ServerMsg::MatchStart { you, .. } =
        until(&mut player, |m| matches!(m, ServerMsg::MatchStart { .. })).await
    else {
        unreachable!()
    };
    drop(player);
    clock.set(1000 + 901);
    let (mut back, resumed) = connect(&url, Some(token), Some(grant.access)).await;
    assert_eq!(resumed, token);
    // The renewal prompt may come before or after the seat's MatchStart.
    let (mut seat, mut renew) = (None, false);
    for _ in 0..200 {
        match recv(&mut back).await {
            ServerMsg::MatchStart { you: again, .. } => seat = Some(again),
            ServerMsg::AuthRequired {
                reason: AuthRefusal::Expired,
            } => renew = true,
            _ => {}
        }
        if seat.is_some() && renew {
            break;
        }
    }
    assert_eq!(seat, Some(you), "the same seat");
    assert!(renew, "the resumed socket asks for renewal");
    let mut stranger = tokio_tungstenite::connect_async(&url).await.unwrap().0;
    auth::send(
        &mut stranger,
        ClientMsg::AuthenticatedHello {
            version: PROTOCOL_VERSION,
            resume: None,
            access: grant.access,
        },
    )
    .await;
    until(&mut stranger, |m| {
        matches!(
            m,
            ServerMsg::AuthRequired {
                reason: AuthRefusal::Invalid
            }
        )
    })
    .await;
    drop(rival);
}
#[tokio::test]
async fn a09_expiry_keeps_the_admitted_match_and_same_socket_renewal_works() {
    let (url, clock) = server().await;
    let (mut player, token) = connect(&url, None, None).await;
    let grant = auth::register(&player, "MatchLease").await.unwrap();
    auth::attach(&mut player, &grant, false).await;
    until(&mut player, |m| {
        matches!(
            m,
            ServerMsg::AuthState {
                status: AuthStatus::Active,
                ..
            }
        )
    })
    .await;
    let (mut opponent, _) = connect(&url, None, None).await;
    auth::send(&mut player, ClientMsg::QuickMatch).await;
    auth::send(&mut opponent, ClientMsg::QuickMatch).await;
    let start = until(&mut player, |m| matches!(m, ServerMsg::MatchStart { .. })).await;
    until(&mut opponent, |m| matches!(m, ServerMsg::MatchStart { .. })).await;
    clock.set(1900);
    let (status, value) =
        native_auth::post(&url, "logout", json!({"refresh_token":grant.refresh}), None)
            .await
            .unwrap();
    assert_eq!(status, 409);
    assert_eq!(
        value["error"], "in_match",
        "ordinary logout must not revoke an admitted match"
    );
    // The expired Access cannot authorize a new account operation.
    auth::send(
        &mut player,
        ClientMsg::SetStronghold {
            kind: tbg_core::StrongholdKind::DEFAULT,
        },
    )
    .await;
    until(&mut player, |m| {
        matches!(
            m,
            ServerMsg::AuthRequired {
                reason: AuthRefusal::Expired
            }
        )
    })
    .await;
    auth::send(
        &mut player,
        ClientMsg::Intent {
            seq: 1,
            intent: tbg_core::PlayerIntention::Purchase { offer: 0 },
        },
    )
    .await;
    let view = until(
        &mut player,
        |m| matches!(m,ServerMsg::View(v) if v.last_seq==1),
    )
    .await;
    assert!(matches!(view,ServerMsg::View(v) if v.phase!=tbg_core::Phase::Ended));
    let (_, value) = native_auth::post(
        &url,
        "refresh",
        json!({"refresh_token":grant.refresh,"operation_id":"11".repeat(16)}),
        None,
    )
    .await
    .unwrap();
    let renewed = native_auth::parse(&value).unwrap();
    auth::send(
        &mut player,
        ClientMsg::Reauthenticate {
            access: renewed.access,
        },
    )
    .await;
    until(&mut player, |m| {
        matches!(
            m,
            ServerMsg::AuthState {
                status: AuthStatus::Active,
                ..
            }
        )
    })
    .await;
    // The same socket and result identity remain admitted; renewal does not create a match.
    auth::send(&mut player, ClientMsg::Surrender).await;
    let end = until(&mut player, |m| matches!(m, ServerMsg::MatchEnd { .. })).await;
    assert!(
        matches!((start,end),(ServerMsg::MatchStart{result_id:a,..},ServerMsg::MatchEnd{result_id:b,..}) if a==b)
    );
    assert_ne!(token, [0; 16]);
}
#[tokio::test]
async fn a10_logout_all_invalidates_expired_match_leases_before_acknowledgment() {
    let (url, clock) = server().await;
    let (mut player, token) = connect(&url, None, None).await;
    let grant = auth::register(&player, "RevokedLease").await.unwrap();
    auth::attach(&mut player, &grant, false).await;
    until(&mut player, |m| {
        matches!(
            m,
            ServerMsg::AuthState {
                status: AuthStatus::Active,
                ..
            }
        )
    })
    .await;
    let (mut opponent, _) = connect(&url, None, None).await;
    auth::send(&mut player, ClientMsg::QuickMatch).await;
    auth::send(&mut opponent, ClientMsg::QuickMatch).await;
    until(&mut player, |m| matches!(m, ServerMsg::MatchStart { .. })).await;
    until(&mut opponent, |m| matches!(m, ServerMsg::MatchStart { .. })).await;
    clock.set(1900);
    let (status, _) = native_auth::post(
        &url,
        "logout-all",
        json!({"account_key":grant.account_key.unwrap()}),
        None,
    )
    .await
    .unwrap();
    assert_eq!(status, 200);
    let (status, _) = native_auth::post(
        &url,
        "confirm",
        json!({"refresh_token":grant.refresh}),
        None,
    )
    .await
    .unwrap();
    assert_eq!(status, 401);
    // A stolen old Access plus the exact match handle cannot recover the revoked seat.
    let mut invalid = tokio_tungstenite::connect_async(&url).await.unwrap().0;
    auth::send(
        &mut invalid,
        ClientMsg::AuthenticatedHello {
            version: PROTOCOL_VERSION,
            resume: Some(token),
            access: grant.access,
        },
    )
    .await;
    until(&mut invalid, |m| {
        matches!(m, ServerMsg::AuthRequired { .. })
    })
    .await;
    // Revocation detaches; it does not fabricate a winner or immediate forfeit.
    let message = until(&mut opponent, |m| {
        matches!(m, ServerMsg::View(_) | ServerMsg::MatchEnd { .. })
    })
    .await;
    assert!(matches!(message,ServerMsg::View(v) if v.phase!=tbg_core::Phase::Ended));
    let _ = player.close(None).await;
}

#[tokio::test]
async fn a09_another_family_or_guest_handle_cannot_resume_the_account() {
    let (url, _) = server().await;
    let (mut owner, token) = connect(&url, None, None).await;
    let grant = auth::register(&owner, "BoundResume").await.unwrap();
    auth::attach(&mut owner, &grant, false).await;
    until(&mut owner, |m| {
        matches!(
            m,
            ServerMsg::AuthState {
                status: AuthStatus::Active,
                ..
            }
        )
    })
    .await;
    let (mut guest, fresh) = connect(&url, Some(token), None).await;
    assert_ne!(fresh, token);
    auth::send(&mut guest, ClientMsg::RankedMatch).await;
    until(&mut guest, |m| {
        matches!(
            m,
            ServerMsg::Error {
                code: protocol::ErrorCode::LoginRequired
            }
        )
    })
    .await;
    let other = auth::login(&owner, grant.account_key.as_deref().unwrap())
        .await
        .unwrap();
    let mut wrong = tokio_tungstenite::connect_async(&url).await.unwrap().0;
    auth::send(
        &mut wrong,
        ClientMsg::AuthenticatedHello {
            version: PROTOCOL_VERSION,
            resume: Some(token),
            access: other.access,
        },
    )
    .await;
    until(&mut wrong, |m| matches!(m, ServerMsg::AuthRequired { .. })).await;
    auth::send(&mut owner, ClientMsg::Leaderboard { offset: 0 }).await;
    until(&mut owner, |m| matches!(m, ServerMsg::Leaderboard { .. })).await;
}
