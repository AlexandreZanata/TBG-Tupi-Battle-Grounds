//! BR-AP01/08: full-directory filters, current generations and honest partial coverage.

use tbg_server::{
    live::{Counts, LiveSnapshot, OnlineRow, PresenceStatus},
    presence::{merge, DirectoryQuery, LocalPresence},
    store::release::SlotRow,
};
fn account(i: usize) -> OnlineRow {
    OnlineRow {
        name: format!("Player{i:03}"),
        rating: 1000,
        status: PresenceStatus::Idle,
        invitable: i % 2 == 0,
        slot: Some("a".into()),
    }
}
fn slot(name: &str, generation: &str) -> SlotRow {
    SlotRow {
        slot: name.into(),
        generation: generation.into(),
        server_build: "fixture".into(),
        contract: "fixture".into(),
        endpoint: format!("/ws/slot-{name}"),
        state: "active".into(),
        canary_pct: 0,
        ranked_authority: false,
        matches_running: 0,
        results_pending: 0,
        ranked_inflight: 0,
        heartbeat_unix: 100,
        version: 1,
    }
}
fn local(name: &str, generation: &str, rows: Vec<OnlineRow>) -> LocalPresence {
    LocalPresence {
        slot: Some(name.into()),
        generation: Some(generation.into()),
        contract: "fixture".into(),
        admitting: true,
        updated_unix: 100,
        revision: 1,
        counts: Counts {
            players: rows.len() as u32 + 2,
            accounts: rows.len() as u32,
            ..Default::default()
        },
        accounts: rows,
    }
}
#[test]
fn one_hundred_thirty_seven_accounts_are_searchable_before_pagination() {
    let rows: Vec<_> = (0..137).map(account).collect();
    let search = DirectoryQuery::parse(Some("offset=0&limit=50&q=pLaYeR136&available=1")).unwrap();
    assert_eq!(rows.iter().filter(|r| search.matches(r)).count(), 1);
    let filter = DirectoryQuery::parse(Some("available=1&offset=50")).unwrap();
    let found: Vec<_> = rows
        .iter()
        .filter(|r| filter.matches(r))
        .skip(filter.offset)
        .take(filter.limit)
        .collect();
    assert_eq!(found.len(), 19);
    assert_eq!(found.last().unwrap().name, "Player136");
    for query in ["available=yes", "limit=wrong", "q=%3Cscript%3E", "q=a&q=b"] {
        assert!(DirectoryQuery::parse(Some(query)).is_err());
    }
}
#[test]
fn two_slots_deduplicate_accounts_and_ignore_replaced_generations() {
    let a = local("a", "new", vec![account(0)]);
    let b = local("b", "new", vec![account(0), account(1)]);
    let old = local("b", "old", vec![account(3)]);
    let source = LiveSnapshot {
        counts: a.counts,
        accounts: a.accounts.clone(),
        ..Default::default()
    };
    let rows = vec![slot("a", "new"), slot("b", "new")];
    let snapshot = merge(&source, &a, &rows, &[old, b], 100);
    assert!(snapshot.complete);
    assert_eq!(snapshot.counts.accounts, 2);
    assert_eq!(snapshot.counts.players, 7);
    assert_eq!(snapshot.accounts.len(), 2);
    assert!(snapshot.accounts.iter().all(|r| r.name != "Player003"));
    let partial = merge(&source, &a, &rows, &[], 100);
    assert!(!partial.complete);
    assert_eq!(partial.counts.players, 3);
    assert_eq!(partial.counts.accounts, 1);
}
#[test]
fn stale_draining_and_incompatible_peers_are_never_invitable() {
    let a = local("a", "new", vec![account(0)]);
    let mut b = local("b", "new", vec![account(2)]);
    let source = LiveSnapshot {
        counts: a.counts,
        accounts: a.accounts.clone(),
        ..Default::default()
    };
    let mut rows = vec![slot("a", "new"), slot("b", "new")];
    rows[1].state = "draining".into();
    assert!(
        !merge(&source, &a, &rows, &[b.clone()], 100)
            .accounts
            .iter()
            .find(|r| r.name == "Player002")
            .unwrap()
            .invitable
    );
    rows[1].state = "active".into();
    b.contract = "old".into();
    assert!(
        !merge(&source, &a, &rows, &[b.clone()], 100)
            .accounts
            .iter()
            .find(|r| r.name == "Player002")
            .unwrap()
            .invitable
    );
    b.contract = "fixture".into();
    b.updated_unix = 90;
    let partial = merge(&source, &a, &rows, &[b.clone()], 100);
    assert!(!partial.complete && partial.stale);
    b.updated_unix = 84;
    let expired = merge(&source, &a, &rows, &[b], 100);
    assert!(!expired.complete);
    assert_eq!(expired.counts.accounts, 1);
}

#[tokio::test]
async fn actual_two_slot_loopback_snapshots_share_counts_and_protect_the_internal_route() {
    use futures_util::{SinkExt, StreamExt};
    use tbg_core::protocol::{self, ClientMsg, ServerMsg};
    use tbg_server::{botclient::native_auth, store::Store, Config};
    let listener_a = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let listener_b = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let ports = [
        listener_a.local_addr().unwrap().port(),
        listener_b.local_addr().unwrap().port(),
    ];
    let dir = std::env::temp_dir().join(format!("tbg-p73-two-slot-fixture-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let db = dir.join("fixture.sqlite");
    let config = |slot: &str| Config {
        db: Some(db.clone()),
        slot: Some(slot.into()),
        generation: format!("fixture-{slot}"),
        ws_endpoint: format!("/ws/slot-{slot}"),
        presence_ports: ports,
        api_per_ip_min: 1000,
        register_per_ip_hour: 100,
        ..Config::default()
    };
    let server_a = tokio::spawn(tbg_server::run(config("a"), listener_a));
    let server_b = tokio::spawn(tbg_server::run(config("b"), listener_b));
    let mut clients = Vec::new();
    for (slot, name, port) in [
        ("a", "FixtureSlotA", ports[0]),
        ("b", "FixtureSlotB", ports[1]),
    ] {
        let base = format!("ws://127.0.0.1:{port}");
        let (status,v)=native_auth::post(&base,"register",serde_json::json!({"name":name,"email":format!("{name}@example.invalid"),"locale":"en","device_id":native_auth::random_hex(16).unwrap(),"persistent":false,"registration_retry_key":native_auth::random_hex(32).unwrap()}),None).await.unwrap();
        assert_eq!(
            status,
            200,
            "synthetic registration failed: {:?}",
            v.get("error")
        );
        let grant = native_auth::parse(&v).unwrap();
        let mut ws = tokio_tungstenite::connect_async(format!("{base}/ws/slot-{slot}"))
            .await
            .unwrap()
            .0;
        ws.send(tokio_tungstenite::tungstenite::Message::Binary(
            protocol::encode(&ClientMsg::AuthenticatedHello {
                version: protocol::PROTOCOL_VERSION,
                resume: None,
                access: grant.access,
            })
            .into(),
        ))
        .await
        .unwrap();
        loop {
            if let Some(Ok(tokio_tungstenite::tungstenite::Message::Binary(bytes))) =
                ws.next().await
            {
                if matches!(
                    protocol::decode_server(&bytes),
                    Ok(ServerMsg::Welcome { .. })
                ) {
                    break;
                }
            }
        }
        ws.send(tokio_tungstenite::tungstenite::Message::Binary(
            protocol::encode(&ClientMsg::SetPlayerChallengesEnabled { enabled: true }).into(),
        ))
        .await
        .unwrap();
        clients.push(ws);
    }
    let store = Store::open(&db).unwrap();
    store
        .release(|conn| {
            conn.execute(
                "UPDATE release_slots SET state=CASE slot WHEN 'a' THEN 'active' ELSE 'canary' END",
                [],
            )
        })
        .await
        .unwrap()
        .unwrap();
    async fn get(port: u16, path: &str, forwarded: bool) -> (u16, serde_json::Value) {
        let target = format!("http://127.0.0.1:{port}{path}");
        tokio::task::spawn_blocking(move || {
            let agent = ureq::Agent::config_builder()
                .http_status_as_error(false)
                .build()
                .new_agent();
            let mut request = agent.get(target);
            if forwarded {
                request = request.header("X-Forwarded-For", "203.0.113.1");
            }
            let mut response = request.call().unwrap();
            let status = response.status().as_u16();
            let bytes = response
                .body_mut()
                .with_config()
                .limit(2 * 1024 * 1024)
                .read_to_vec()
                .unwrap();
            (
                status,
                serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null),
            )
        })
        .await
        .unwrap()
    }
    let (snapshot, live) = tokio::time::timeout(std::time::Duration::from_secs(8), async {
        loop {
            let (_, v) = get(ports[0], "/api/online?limit=1", false).await;
            let (_, live) = get(ports[1], "/api/live", false).await;
            // Each slot refreshes its own cached snapshot on an independent timer.
            // A converged directory on A cannot synchronize the live cache on B.
            if v["total"] == 2
                && v["complete"] == true
                && live["online"]["accounts"] == 2
                && live["online"]["players"] == 2
            {
                break (v, live);
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    })
    .await
    .expect("two real slot snapshots did not aggregate");
    assert_eq!(snapshot["rows"].as_array().unwrap().len(), 1);
    assert_eq!(snapshot["next_offset"], 1);
    assert_eq!(live["online"]["accounts"], 2);
    assert_eq!(live["online"]["players"], 2);
    assert_eq!(get(ports[0], "/api/internal/presence", true).await.0, 404);
    assert_eq!(get(ports[0], "/api/internal/presence", false).await.0, 200);
    store
        .release(|conn| {
            conn.execute(
                "UPDATE release_slots SET state='draining' WHERE slot='b'",
                [],
            )
        })
        .await
        .unwrap()
        .unwrap();
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        loop {
            let (_, v) = get(ports[0], "/api/online?q=FixtureSlotB", false).await;
            if v["rows"][0]["invitable"] == false {
                assert_eq!(v["total"], 1);
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    })
    .await
    .unwrap();
    // Only this fixture listener is stopped. Last-good remote rows become explicitly partial.
    server_b.abort();
    tokio::time::timeout(std::time::Duration::from_secs(8), async {
        loop {
            let (_, v) = get(ports[0], "/api/online", false).await;
            if v["complete"] == false && v["stale"] == true {
                assert!(v["total"].as_u64().unwrap() >= 1);
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }
    })
    .await
    .unwrap();
    drop(clients);
    server_a.abort();
}
