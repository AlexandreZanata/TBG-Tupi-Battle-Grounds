//! P73 real loopback clients: explicit consent, canonical summaries, private history and Ranked-safe rematches.

use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::time::Duration;
use tbg_core::protocol::{self, ClientMsg, HistorySaveState, MatchMode, ServerMsg};
use tbg_server::{botclient::native_auth, Config};
#[path = "common/player_history.rs"]
mod fixture;
type Ws =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

async fn send(ws: &mut Ws, msg: ClientMsg) {
    ws.send(tokio_tungstenite::tungstenite::Message::Binary(
        protocol::encode(&msg).into(),
    ))
    .await
    .unwrap();
}
async fn until(ws: &mut Ws, predicate: impl Fn(&ServerMsg) -> bool) -> ServerMsg {
    tokio::time::timeout(Duration::from_secs(12), async {
        loop {
            let Some(Ok(tokio_tungstenite::tungstenite::Message::Binary(bytes))) = ws.next().await
            else {
                continue;
            };
            let msg = protocol::decode_server(&bytes).unwrap();
            if let ServerMsg::Ping { id } = msg {
                send(ws, ClientMsg::Pong { id }).await;
                continue;
            }
            if predicate(&msg) {
                return msg;
            }
        }
    })
    .await
    .expect("fixture server did not reach the expected state")
}
async fn account(base: &str, name: &str) -> (Ws, native_auth::Grant, protocol::Token) {
    let (_,v)=native_auth::post(base,"register",json!({"name":name,"device_id":native_auth::random_hex(16).unwrap(),"persistent":false,"registration_retry_key":native_auth::random_hex(32).unwrap()}),None).await.unwrap();
    let grant = native_auth::parse(&v).unwrap();
    let mut ws = tokio_tungstenite::connect_async(format!("{base}/ws"))
        .await
        .unwrap()
        .0;
    send(
        &mut ws,
        ClientMsg::AuthenticatedHello {
            version: protocol::PROTOCOL_VERSION,
            resume: None,
            access: grant.access,
        },
    )
    .await;
    let welcome = until(&mut ws, |m| matches!(m, ServerMsg::Welcome { .. })).await;
    let ServerMsg::Welcome { token, .. } = welcome else {
        unreachable!()
    };
    send(
        &mut ws,
        ClientMsg::SetPlayerChallengesEnabled { enabled: true },
    )
    .await;
    until(&mut ws, |m| {
        matches!(m, ServerMsg::PlayerChallengeState { enabled: true, .. })
    })
    .await;
    (ws, grant, token)
}
async fn http(
    base: &str,
    path: &str,
    method: &str,
    access: Option<protocol::AccessToken>,
    body: Value,
    revision: Option<i64>,
) -> (u16, String, Value) {
    let target = format!("{}{path}", base.replace("ws://", "http://"));
    let method = method.to_owned();
    tokio::task::spawn_blocking(move || {
        let agent = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .max_redirects(0)
            .timeout_global(Some(Duration::from_secs(3)))
            .build()
            .new_agent();
        let mut req = ureq::http::Request::builder()
            .method(method.as_str())
            .uri(target)
            .header("Content-Type", "application/json")
            .header("X-TBG-Client", "native");
        if let Some(access) = access {
            req = req.header(
                "Authorization",
                format!("Bearer {}", tbg_server::ranked::hex(&access.0)),
            );
        }
        if let Some(revision) = revision {
            req = req.header("If-Match", format!("\"{revision}\""));
        }
        let bytes = if body.is_null() {
            Vec::new()
        } else {
            serde_json::to_vec(&body).unwrap()
        };
        let mut response = agent.run(req.body(bytes).unwrap()).unwrap();
        let status = response.status().as_u16();
        let cache = response
            .headers()
            .get("cache-control")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_owned();
        let bytes = response
            .body_mut()
            .with_config()
            .limit(262144)
            .read_to_vec()
            .unwrap();
        (
            status,
            cache,
            serde_json::from_slice(&bytes).unwrap_or(Value::Null),
        )
    })
    .await
    .unwrap()
}

#[tokio::test]
async fn directed_game_result_reconnect_history_privacy_and_latest_opponent_rematch_use_real_clients(
) {
    let path = fixture::path();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("ws://{}", listener.local_addr().unwrap());
    let (replays, mut replay_rx) = tokio::sync::mpsc::unbounded_channel();
    let clock = tbg_server::auth::Clock::controlled(1000);
    tokio::spawn(tbg_server::run(
        Config {
            db: Some(path),
            tick_ms: 5,
            pause_secs: 0,
            max_per_ip: 100,
            register_per_ip_hour: 100,
            api_per_ip_min: 1000,
            history_migrate: true,
            history_collect: true,
            history_api: true,
            history_public: true,
            replays: Some(replays),
            auth_clock: clock.clone(),
            ..Config::default()
        },
        listener,
    ));
    let (mut a, grant_a, token_a) = account(&base, "FixtureA").await;
    let (mut b, grant_b, _token_b) = account(&base, "FixtureB").await;
    send(
        &mut a,
        ClientMsg::ChallengePlayer {
            name: "FixtureB".into(),
        },
    )
    .await;
    let notice = until(
        &mut b,
        |m| matches!(m,ServerMsg::PlayerChallengeState{incoming,..} if !incoming.is_empty()),
    )
    .await;
    let ServerMsg::PlayerChallengeState { incoming, .. } = notice else {
        unreachable!()
    };
    let id = incoming[0].id;
    assert_eq!(incoming[0].peer_name, "FixtureA");
    send(
        &mut b,
        ClientMsg::ChallengePlayer {
            name: "FixtureA".into(),
        },
    )
    .await;
    send(
        &mut b,
        ClientMsg::AnswerPlayerChallenge { id, accept: true },
    )
    .await;
    let start_a = until(&mut a, |m| matches!(m, ServerMsg::MatchStart { .. })).await;
    let start_b = until(&mut b, |m| matches!(m, ServerMsg::MatchStart { .. })).await;
    let ServerMsg::MatchStart {
        result_id: first,
        mode: MatchMode::Challenge,
        ..
    } = start_a
    else {
        panic!("direct invitation must be unrated Challenge")
    };
    assert!(matches!(start_b,ServerMsg::MatchStart{result_id,..}if result_id==first));
    until(
        &mut b,
        |m| matches!(m,ServerMsg::View(v)if v.phase==tbg_core::Phase::Running),
    )
    .await;
    send(&mut b, ClientMsg::Surrender).await;
    let summary_a = until(&mut a, |m| {
        matches!(m, ServerMsg::PersonalMatchResult { .. })
    })
    .await;
    let summary_b = until(&mut b, |m| {
        matches!(m, ServerMsg::PersonalMatchResult { .. })
    })
    .await;
    let ServerMsg::PersonalMatchResult {
        summary: personal_a,
        ..
    } = summary_a
    else {
        unreachable!()
    };
    assert_eq!(personal_a.outcome, "win");
    assert_eq!(personal_a.purchases, Some(0));
    assert!(personal_a.wall_ms.is_some());
    assert!(
        matches!(summary_b,ServerMsg::PersonalMatchResult{summary,..}if summary.outcome=="loss")
    );
    until(&mut a, |m| matches!(m, ServerMsg::MatchEnd { .. })).await;
    until(&mut b, |m| matches!(m, ServerMsg::MatchEnd { .. })).await;
    until(
        &mut a,
        |m| matches!(m,ServerMsg::RematchOpen{opponent}if opponent=="FixtureB"),
    )
    .await;
    until(
        &mut b,
        |m| matches!(m,ServerMsg::RematchOpen{opponent}if opponent=="FixtureA"),
    )
    .await;
    let stored = until(&mut a, |m| {
        matches!(
            m,
            ServerMsg::MatchHistoryStatus {
                state: HistorySaveState::Stored,
                ..
            }
        )
    })
    .await;
    let ServerMsg::MatchHistoryStatus { entry_id, .. } = stored else {
        unreachable!()
    };
    let entry = tbg_server::ranked::hex(&entry_id);
    let (_, _, detail) = http(
        &base,
        &format!("/api/me/matches/{entry}"),
        "GET",
        Some(grant_a.access),
        Value::Null,
        None,
    )
    .await;
    assert_eq!(
        detail["summary"],
        serde_json::to_value(&personal_a).unwrap(),
        "terminal UI/reconnect and durable detail share one summary"
    );
    assert_eq!(detail["rating_status"], "unranked");
    assert_eq!(
        http(
            &base,
            &format!("/api/me/matches/{entry}"),
            "GET",
            Some(grant_b.access),
            Value::Null,
            None
        )
        .await
        .0,
        404
    );
    assert_eq!(
        http(
            &base,
            "/api/players/FixtureA/matches",
            "GET",
            None,
            Value::Null,
            None
        )
        .await
        .0,
        403
    );
    assert_eq!(
        http(
            &base,
            "/api/me/preferences/history",
            "PATCH",
            Some(grant_a.access),
            json!({"visibility":"public"}),
            Some(0)
        )
        .await
        .0,
        200
    );
    let (status, cache, public) = http(
        &base,
        &format!("/api/players/FixtureA/matches/{entry}"),
        "GET",
        None,
        Value::Null,
        None,
    )
    .await;
    assert_eq!(status, 200);
    assert!(cache.contains("no-store"));
    assert!(public["opponent"].is_null());
    assert_eq!(
        http(
            &base,
            "/api/me/preferences/history",
            "PATCH",
            Some(grant_a.access),
            json!({"visibility":"private"}),
            Some(1)
        )
        .await
        .0,
        200
    );
    assert_eq!(
        http(
            &base,
            "/api/me/preferences/history",
            "PATCH",
            Some(grant_a.access),
            json!({"visibility":"public"}),
            Some(0)
        )
        .await
        .0,
        409
    );
    let (_, cache, profile) = http(
        &base,
        "/api/players/FixtureA",
        "GET",
        None,
        Value::Null,
        None,
    )
    .await;
    assert!(cache.contains("no-store"));
    assert!(profile["recent"].as_array().unwrap().is_empty());
    // A detached authenticated controller recovers the exact terminal summary and Stored state.
    a.close(None).await.unwrap();
    tokio::time::sleep(Duration::from_millis(25)).await;
    let mut resumed = tokio_tungstenite::connect_async(format!("{base}/ws"))
        .await
        .unwrap()
        .0;
    send(
        &mut resumed,
        ClientMsg::AuthenticatedHello {
            version: protocol::PROTOCOL_VERSION,
            resume: Some(token_a),
            access: grant_a.access,
        },
    )
    .await;
    until(&mut resumed, |m| matches!(m, ServerMsg::Welcome { .. })).await;
    let replayed = until(&mut resumed, |m| {
        matches!(m, ServerMsg::PersonalMatchResult { .. })
    })
    .await;
    assert!(
        matches!(replayed,ServerMsg::PersonalMatchResult{result_id,summary}if result_id==first && *summary==*personal_a)
    );
    until(&mut resumed, |m| {
        matches!(
            m,
            ServerMsg::MatchHistoryStatus {
                state: HistorySaveState::Stored,
                ..
            }
        )
    })
    .await;
    until(&mut resumed, |m| matches!(m, ServerMsg::RematchOpen { .. })).await;
    a = resumed;
    // Browsing players/history does not send Cancel or consume the private ticket.
    send(&mut a, ClientMsg::RequestRematch).await;
    send(&mut b, ClientMsg::RequestRematch).await;
    let second = until(&mut b, |m| matches!(m, ServerMsg::MatchStart { .. })).await;
    assert!(
        matches!(second,ServerMsg::MatchStart{result_id,mode:MatchMode::Challenge,..}if result_id!=first)
    );
    until(&mut a, |m| matches!(m, ServerMsg::MatchStart { .. })).await;
    until(
        &mut b,
        |m| matches!(m,ServerMsg::View(v)if v.phase==tbg_core::Phase::Running),
    )
    .await;
    send(&mut b, ClientMsg::Surrender).await;
    until(&mut b, |m| matches!(m, ServerMsg::MatchEnd { .. })).await;
    let first_replay = replay_rx.recv().await.unwrap();
    let second_replay = replay_rx.recv().await.unwrap();
    assert_ne!(first_replay.seed, second_replay.seed);
    clock.set(2000);
    assert_eq!(
        http(
            &base,
            "/api/me/matches",
            "GET",
            Some(grant_a.access),
            Value::Null,
            None
        )
        .await
        .0,
        401,
        "expired match leases never authorize private history HTTP"
    );
}

#[tokio::test]
async fn failed_history_journal_reports_delayed_without_changing_the_authoritative_result() {
    let path = fixture::path();
    std::fs::write(
        path.parent().unwrap().join("history-pending-single"),
        b"explicit journal failure fixture",
    )
    .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("ws://{}", listener.local_addr().unwrap());
    tokio::spawn(tbg_server::run(
        Config {
            db: Some(path),
            tick_ms: 5,
            pause_secs: 0,
            max_per_ip: 100,
            register_per_ip_hour: 100,
            history_migrate: true,
            history_collect: true,
            history_api: true,
            ..Config::default()
        },
        listener,
    ));
    let (mut a, grant, _) = account(&base, "FixtureA").await;
    let (mut b, _, _) = account(&base, "FixtureB").await;
    send(&mut a, ClientMsg::CreateChallenge).await;
    let invitation = until(&mut a, |m| matches!(m, ServerMsg::ChallengeWaiting { .. })).await;
    let ServerMsg::ChallengeWaiting { token } = invitation else {
        unreachable!()
    };
    send(&mut b, ClientMsg::JoinChallenge { token }).await;
    until(&mut a, |m| matches!(m, ServerMsg::MatchStart { .. })).await;
    until(
        &mut b,
        |m| matches!(m,ServerMsg::View(v)if v.phase==tbg_core::Phase::Running),
    )
    .await;
    send(&mut b, ClientMsg::Surrender).await;
    let result = until(&mut a, |m| matches!(m, ServerMsg::MatchEnd { .. })).await;
    assert!(
        matches!(result,ServerMsg::MatchEnd{outcome,..}if outcome.winner==Some(tbg_core::Seat::A))
    );
    until(&mut a, |m| {
        matches!(
            m,
            ServerMsg::MatchHistoryStatus {
                state: HistorySaveState::Delayed,
                ..
            }
        )
    })
    .await;
    let (status, _, history) = http(
        &base,
        "/api/me/matches",
        "GET",
        Some(grant.access),
        Value::Null,
        None,
    )
    .await;
    assert_eq!(status, 200);
    assert!(history["rows"].as_array().unwrap().is_empty());
}
