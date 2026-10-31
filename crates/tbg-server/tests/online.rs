//! Integration tests over real WebSockets against an in-process server.
//! Matches run at a 5 ms tick so a full match takes seconds.

use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

use tbg_core::protocol::{
    self, ClientMsg, ErrorCode, InviteToken, MatchMode, ServerMsg, PROTOCOL_VERSION,
};
use tbg_core::{EndReason, PlayerIntention, Rejection, Seat};
use tbg_server::botclient::{play_one, BotOpts};
use tbg_server::Config;

type Ws = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

async fn server_with(config: Config) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(tbg_server::run(config, listener));
    format!("ws://{addr}/ws")
}

async fn server(reconnect_secs: u64) -> String {
    server_with(Config {
        tick_ms: 5,
        reconnect_secs,
        max_per_ip: 1_000,
        ..Config::default()
    })
    .await
}

async fn connect(url: &str) -> Ws {
    connect_async(url).await.unwrap().0
}

async fn send(ws: &mut Ws, msg: &ClientMsg) {
    ws.send(Message::Binary(protocol::encode(msg).into()))
        .await
        .unwrap();
}

/// Next server message, answering pings and skipping `NetStats`;
/// `None` once the socket closes.
async fn recv(ws: &mut Ws) -> Option<ServerMsg> {
    loop {
        let msg = tokio::time::timeout(Duration::from_secs(10), ws.next())
            .await
            .ok()??
            .ok()?;
        let Message::Binary(b) = msg else { continue };
        match protocol::decode_server(&b).unwrap() {
            // Best-effort: the server may have closed already (e.g. after
            // a rate-limit error); a failed pong must not fail the test.
            ServerMsg::Ping { id } => {
                let _ = ws
                    .send(Message::Binary(
                        protocol::encode(&ClientMsg::Pong { id }).into(),
                    ))
                    .await;
            }
            // Latency housekeeping can arrive between any two replies.
            ServerMsg::NetStats { .. } => {}
            other => return Some(other),
        }
    }
}

async fn hello_token(ws: &mut Ws) -> [u8; 16] {
    send(
        ws,
        &ClientMsg::Hello {
            version: PROTOCOL_VERSION,
            resume: None,
        },
    )
    .await;
    match recv(ws).await {
        Some(ServerMsg::Welcome { token, .. }) => token,
        other => panic!("expected Welcome, got {other:?}"),
    }
}

async fn hello(ws: &mut Ws) {
    hello_token(ws).await;
}

async fn hello_resume(ws: &mut Ws, token: [u8; 16]) {
    send(
        ws,
        &ClientMsg::Hello {
            version: PROTOCOL_VERSION,
            resume: Some(token),
        },
    )
    .await;
    assert!(matches!(recv(ws).await, Some(ServerMsg::Welcome { .. })));
}

async fn create_challenge(ws: &mut Ws) -> InviteToken {
    send(ws, &ClientMsg::CreateChallenge).await;
    match recv(ws).await {
        Some(ServerMsg::ChallengeWaiting { token }) => token,
        other => panic!("expected ChallengeWaiting, got {other:?}"),
    }
}

async fn challenge_start(ws: &mut Ws) -> (Seat, MatchMode) {
    loop {
        match recv(ws).await {
            Some(ServerMsg::MatchStart { you, mode, .. }) => return (you, mode),
            Some(_) => {}
            None => panic!("socket closed before MatchStart"),
        }
    }
}

async fn challenge_result(ws: &mut Ws) -> Result<Seat, ()> {
    loop {
        match recv(ws).await {
            Some(ServerMsg::MatchStart { you, .. }) => return Ok(you),
            Some(ServerMsg::ChallengeUnavailable) => return Err(()),
            Some(_) => {}
            None => panic!("socket closed before challenge result"),
        }
    }
}

async fn rematch_open(ws: &mut Ws) -> String {
    loop {
        match recv(ws).await {
            Some(ServerMsg::RematchOpen { opponent }) => return opponent,
            Some(_) => {}
            None => panic!("socket closed before RematchOpen"),
        }
    }
}

async fn rematch_state(ws: &mut Ws) -> (bool, bool) {
    loop {
        match recv(ws).await {
            Some(ServerMsg::RematchState {
                you_requested,
                opponent_requested,
            }) => return (you_requested, opponent_requested),
            Some(_) => {}
            None => panic!("socket closed before RematchState"),
        }
    }
}

async fn no_match_start(ws: &mut Ws, wait: Duration) {
    let deadline = tokio::time::Instant::now() + wait;
    loop {
        let left = deadline.saturating_duration_since(tokio::time::Instant::now());
        if left.is_zero() {
            return;
        }
        match tokio::time::timeout(left, recv(ws)).await {
            Ok(Some(ServerMsg::MatchStart { .. })) => panic!("unexpected MatchStart"),
            Ok(Some(_)) => continue,
            Ok(None) | Err(_) => return,
        }
    }
}

async fn start_challenge(
    url: &str,
) -> (
    Ws,
    Ws,
    [u8; 16],
    [u8; 16],
    (Seat, MatchMode),
    (Seat, MatchMode),
) {
    let mut owner = connect(url).await;
    let mut guest = connect(url).await;
    let owner_token = hello_token(&mut owner).await;
    let guest_token = hello_token(&mut guest).await;
    let invite = create_challenge(&mut owner).await;
    send(&mut guest, &ClientMsg::JoinChallenge { token: invite }).await;
    let owner_start = challenge_start(&mut owner).await;
    let guest_start = challenge_start(&mut guest).await;
    (
        owner,
        guest,
        owner_token,
        guest_token,
        owner_start,
        guest_start,
    )
}

async fn end_by_surrender(ws: &mut Ws, other: &mut Ws) {
    send(ws, &ClientMsg::Surrender).await;
    let _ = await_end(ws).await;
    let _ = await_end(other).await;
}

fn fast(url: &str) -> BotOpts {
    BotOpts {
        timeout: Duration::from_secs(60),
        ..BotOpts::new(url)
    }
}

#[tokio::test]
async fn two_bots_finish_a_match() {
    let url = server(30).await;
    let (a, b) = tokio::join!(play_one(fast(&url), 1), play_one(fast(&url), 2));
    assert!(a.completed && b.completed, "{:?} / {:?}", a.error, b.error);
    assert_eq!(a.outcome, b.outcome, "both see the same result");
    assert_ne!(a.seat, b.seat);
    assert!(!a.rtt_ms.is_empty() || !b.rtt_ms.is_empty() || a.view_gaps_ms.len() > 10);
}

#[tokio::test]
async fn cannot_evolve_opponent_unit() {
    // BR-A02: the seat comes from the connection, not from the message.
    let url = server(30).await;
    let bot = tokio::spawn(play_one(fast(&url), 3));
    let mut ws = connect(&url).await;
    hello(&mut ws).await;
    send(&mut ws, &ClientMsg::QuickMatch).await;
    let mut you = None;
    while let Some(msg) = recv(&mut ws).await {
        match msg {
            ServerMsg::MatchStart { you: s, .. } => you = Some(s),
            ServerMsg::View(v) => {
                if let Some(enemy) = v.units.iter().find(|u| Some(u.seat) != you) {
                    let intent = PlayerIntention::Evolve {
                        unit: enemy.id,
                        branch: 0,
                    };
                    send(&mut ws, &ClientMsg::Intent { seq: 9, intent }).await;
                }
            }
            ServerMsg::Rejected { seq: 9, reason } => {
                assert_eq!(reason, Rejection::NotYourUnit);
                bot.abort();
                return;
            }
            _ => {}
        }
    }
    panic!("never rejected");
}

#[tokio::test]
async fn malformed_frame_closes_connection() {
    let url = server(30).await;
    let mut ws = connect(&url).await;
    hello(&mut ws).await;
    ws.send(Message::Binary(vec![0xff, 0x13, 0x07].into()))
        .await
        .unwrap();
    assert_eq!(
        recv(&mut ws).await,
        Some(ServerMsg::Error {
            code: ErrorCode::BadMessage
        })
    );
    assert_eq!(recv(&mut ws).await, None);
}

#[tokio::test]
async fn version_mismatch_rejected() {
    let url = server(30).await;
    let mut ws = connect(&url).await;
    send(
        &mut ws,
        &ClientMsg::Hello {
            version: 999,
            resume: None,
        },
    )
    .await;
    assert_eq!(
        recv(&mut ws).await,
        Some(ServerMsg::VersionMismatch {
            server: PROTOCOL_VERSION
        })
    );
}

#[tokio::test]
async fn rate_limit_closes_flooder() {
    let url = server(30).await;
    let mut ws = connect(&url).await;
    hello(&mut ws).await;
    // 50 frames (> the 40-frame burst) in one flush, then read only: the
    // client must not write again. Answering the server's app `Ping` with a
    // `Pong` after the server dropped the socket made its kernel reply RST,
    // and the RST discarded the unread `RateLimited` frame on our side (the
    // old ~1-in-6 flake).
    for _ in 0..50 {
        let pong = Message::Binary(protocol::encode(&ClientMsg::Pong { id: 0 }).into());
        if ws.feed(pong).await.is_err() {
            break;
        }
    }
    let _ = ws.flush().await;
    let mut saw = false;
    while let Ok(Some(Ok(frame))) = tokio::time::timeout(Duration::from_secs(10), ws.next()).await {
        if let Message::Binary(b) = frame {
            saw |= protocol::decode_server(&b)
                == Ok(ServerMsg::Error {
                    code: ErrorCode::RateLimited,
                });
        }
    }
    assert!(saw, "flooder was not rate limited");
}

#[tokio::test]
async fn reconnect_resumes_match() {
    let url = server(30).await;
    let mut dropper = fast(&url);
    dropper.drop_after = Some(Duration::from_millis(300));
    dropper.rejoin_after = Some(Duration::from_secs(2));
    let (a, b) = tokio::join!(play_one(dropper, 4), play_one(fast(&url), 5));
    assert!(
        a.dropped && a.reconnected,
        "dropper did not reconnect: {:?}",
        a.error
    );
    assert!(a.completed && b.completed, "{:?} / {:?}", a.error, b.error);
    assert_ne!(b.outcome.map(|o| o.reason), Some(EndReason::Forfeit));
}

#[tokio::test]
async fn forfeit_after_window() {
    let url = server(1).await;
    let mut leaver = fast(&url);
    leaver.drop_after = Some(Duration::from_millis(300));
    let (a, b) = tokio::join!(play_one(leaver, 6), play_one(fast(&url), 7));
    assert!(a.dropped && !a.completed);
    let o = b.outcome.expect("stayer got a result");
    assert_eq!(o.reason, EndReason::Forfeit);
    assert_eq!(o.winner, b.seat);
    assert!(matches!(b.seat, Some(Seat::A | Seat::B)));
}

#[tokio::test]
async fn intent_is_acknowledged_next_tick() {
    // P9: the acting seat gets a view with `last_seq >= seq` (or a
    // rejection) within 2 ticks + local slack after its intention.
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}/ws", listener.local_addr().unwrap());
    let config = Config {
        tick_ms: 50,
        max_per_ip: 1_000,
        ..Config::default()
    };
    tokio::spawn(tbg_server::run(config, listener));
    let (mut a, mut b) = (connect(&url).await, connect(&url).await);
    hello(&mut a).await;
    hello(&mut b).await;
    send(&mut a, &ClientMsg::QuickMatch).await;
    send(&mut b, &ClientMsg::QuickMatch).await;
    // Wait for the first view so the shop offers exist server-side.
    let mut first_view = false;
    while !first_view {
        match recv(&mut a).await {
            Some(ServerMsg::View(_)) => first_view = true,
            Some(_) => {}
            None => panic!("socket closed before first view"),
        }
    }
    let seq = 7u32;
    let t0 = std::time::Instant::now();
    send(
        &mut a,
        &ClientMsg::Intent {
            seq,
            intent: PlayerIntention::Purchase { offer: 0 },
        },
    )
    .await;
    loop {
        match tokio::time::timeout(Duration::from_secs(10), recv(&mut a)).await {
            Ok(Some(ServerMsg::View(v))) if v.last_seq >= seq => break,
            Ok(Some(ServerMsg::Rejected { seq: s, .. })) if s == seq => break,
            Ok(Some(_)) => continue,
            _ => panic!("no ack view for seq {seq}"),
        }
    }
    let ms = t0.elapsed().as_millis();
    assert!(ms <= 500, "ack took {ms} ms (> 2 ticks + slack)");
}

#[tokio::test]
async fn protocol_pong_keeps_socket_alive() {
    // P9: a client that answers protocol pings but sends no app message
    // for 20 s (a hidden tab) is still connected (idle timeout is 15 s).
    let url = server(30).await;
    let (mut ws, _) = connect_async(url.as_str()).await.unwrap();
    send(
        &mut ws,
        &ClientMsg::Hello {
            version: PROTOCOL_VERSION,
            resume: None,
        },
    )
    .await;
    let deadline = std::time::Instant::now() + Duration::from_secs(20);
    loop {
        let msg = tokio::time::timeout(Duration::from_secs(25), async {
            use futures_util::StreamExt;
            ws.next().await
        })
        .await
        .expect("timed out waiting for server frames")
        .expect("stream ended")
        .expect("socket error");
        match msg {
            Message::Binary(b) => match protocol::decode_server(&b).unwrap() {
                ServerMsg::Welcome { .. } => {}
                ServerMsg::Ping { id } => {
                    send(&mut ws, &ClientMsg::Pong { id }).await;
                }
                _ => {}
            },
            Message::Ping(payload) => {
                use futures_util::SinkExt;
                ws.send(Message::Pong(payload)).await.unwrap();
            }
            Message::Close(_) => panic!("server closed the hidden-tab socket"),
            _ => {}
        }
        if std::time::Instant::now() >= deadline {
            break;
        }
    }
    // Still alive: the server answers a fresh ping round.
    send(&mut ws, &ClientMsg::QuickMatch).await;
    loop {
        let msg = tokio::time::timeout(Duration::from_secs(10), async {
            use futures_util::StreamExt;
            ws.next().await
        })
        .await
        .expect("no reply after 20 s of protocol-only activity")
        .expect("stream ended")
        .expect("socket error");
        if let Message::Binary(b) = msg {
            match protocol::decode_server(&b).unwrap() {
                ServerMsg::Queued => break,
                ServerMsg::Ping { id } => {
                    send(&mut ws, &ClientMsg::Pong { id }).await;
                }
                _ => continue,
            }
        }
    }
}

#[tokio::test]
async fn private_room_pairs_owner_and_guest() {
    let url = server(30).await;
    let (mut owner, mut guest) = (connect(&url).await, connect(&url).await);
    hello(&mut owner).await;
    hello(&mut guest).await;
    send(&mut guest, &ClientMsg::JoinRoom { code: *b"ZZZZZ" }).await;
    assert_eq!(recv(&mut guest).await, Some(ServerMsg::RoomNotFound));
    send(&mut owner, &ClientMsg::CreateRoom).await;
    let Some(ServerMsg::RoomCreated { code }) = recv(&mut owner).await else {
        panic!("no room code");
    };
    send(&mut guest, &ClientMsg::JoinRoom { code }).await;
    let starts = [recv(&mut owner).await, recv(&mut guest).await];
    let seats: Vec<Seat> = starts
        .iter()
        .filter_map(|m| match m {
            Some(ServerMsg::MatchStart { you, .. }) => Some(*you),
            _ => None,
        })
        .collect();
    assert_eq!(seats.len(), 2, "both got MatchStart: {starts:?}");
    assert_ne!(seats[0], seats[1]);
}

#[tokio::test]
async fn records_insert_and_count() {
    // P5: one row per finished match, with per-seat JSON.
    let dir = std::env::temp_dir().join(format!("tbg-records-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let db = dir.join("test.sqlite3");
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}/ws", listener.local_addr().unwrap());
    let config = Config {
        tick_ms: 5,
        max_per_ip: 1_000,
        db: Some(db.clone()),
        ..Config::default()
    };
    tokio::spawn(tbg_server::run(config, listener));
    let (a, b) = tokio::join!(play_one(fast(&url), 11), play_one(fast(&url), 12));
    assert!(a.completed && b.completed);
    tokio::time::sleep(Duration::from_millis(300)).await;
    let store = tbg_server::store::Store::open(&db).unwrap();
    assert_eq!(store.match_count().await.unwrap(), 1);
    let _ = std::fs::remove_dir_all(dir);
}

#[tokio::test]
async fn draining_server_refuses_new_matches() {
    // P5 graceful shutdown: no new matches once draining.
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}/ws", listener.local_addr().unwrap());
    let shared = tbg_server::Shared::default();
    shared
        .draining
        .store(true, std::sync::atomic::Ordering::Relaxed);
    let config = Config {
        tick_ms: 5,
        ..Config::default()
    };
    tokio::spawn(tbg_server::run_with(config, listener, shared));
    let mut ws = connect(&url).await;
    hello(&mut ws).await;
    send(&mut ws, &ClientMsg::QuickMatch).await;
    assert_eq!(
        recv(&mut ws).await,
        Some(ServerMsg::Error {
            code: ErrorCode::ServerFull
        })
    );
}

async fn await_seat(ws: &mut Ws) -> Seat {
    loop {
        match recv(ws).await {
            Some(ServerMsg::MatchStart { you, .. }) => return you,
            Some(_) => {}
            None => panic!("socket closed before MatchStart"),
        }
    }
}

async fn await_end(ws: &mut Ws) -> tbg_core::Outcome {
    loop {
        match recv(ws).await {
            Some(ServerMsg::MatchEnd { outcome, .. }) => return outcome,
            Some(_) => {}
            None => panic!("socket closed before MatchEnd"),
        }
    }
}

#[tokio::test]
async fn br_m06_surrender_forfeits_once() {
    // BR-M06/BR-M03: surrendering is a forfeit; both seats see one MatchEnd
    // with the other side as winner. Outside a match the message is refused.
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}/ws", listener.local_addr().unwrap());
    let shared = tbg_server::Shared::default();
    let metrics = shared.metrics.clone();
    let config = Config {
        tick_ms: 5,
        max_per_ip: 1_000,
        ..Config::default()
    };
    tokio::spawn(tbg_server::run_with(config, listener, shared));

    let mut idle = connect(&url).await;
    hello(&mut idle).await;
    send(&mut idle, &ClientMsg::Surrender).await;
    assert_eq!(
        recv(&mut idle).await,
        Some(ServerMsg::Error {
            code: ErrorCode::NotInMatch
        })
    );

    let (mut a, mut b) = (connect(&url).await, connect(&url).await);
    hello(&mut a).await;
    hello(&mut b).await;
    send(&mut a, &ClientMsg::QuickMatch).await;
    send(&mut b, &ClientMsg::QuickMatch).await;
    let (seat_a, seat_b) = tokio::join!(await_seat(&mut a), await_seat(&mut b));
    assert_ne!(seat_a, seat_b);

    // Two surrenders in a row still produce a single end.
    send(&mut a, &ClientMsg::Surrender).await;
    send(&mut a, &ClientMsg::Surrender).await;
    let (oa, ob) = tokio::join!(await_end(&mut a), await_end(&mut b));
    assert_eq!(oa, ob, "both seats agree on the outcome");
    assert_eq!(oa.reason, EndReason::Forfeit);
    assert_eq!(oa.winner, Some(seat_b), "the other seat wins");
    let second = tokio::time::timeout(Duration::from_millis(300), async {
        loop {
            match recv(&mut a).await {
                Some(ServerMsg::MatchEnd { .. }) => return true,
                Some(_) => {}
                None => return false,
            }
        }
    })
    .await;
    assert!(!matches!(second, Ok(true)), "duplicate MatchEnd");
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while metrics
        .matches_running
        .load(std::sync::atomic::Ordering::Relaxed)
        != 0
        && std::time::Instant::now() < deadline
    {
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert_eq!(
        metrics
            .matches_running
            .load(std::sync::atomic::Ordering::Relaxed),
        0
    );
}

#[tokio::test]
async fn br_u07_online_army_stays_in_sync() {
    // D-32 test 10: a real client buys the cheapest affordable offer all
    // match long. Its army rebuilt from Spawn/Death events equals the
    // server's view on every frame, and only Energy ever refuses a purchase.
    let url = server(30).await;
    let bot = tokio::spawn(play_one(fast(&url), 21));
    let mut ws = connect(&url).await;
    hello(&mut ws).await;
    send(&mut ws, &ClientMsg::QuickMatch).await;
    let (mut seq, mut refused) = (0u32, 0u32);
    let mut mine = std::collections::BTreeSet::new();
    while let Some(msg) = recv(&mut ws).await {
        match msg {
            ServerMsg::View(v) => {
                for e in &v.events {
                    match *e {
                        tbg_core::Event::Spawn { unit, seat, .. } if seat == v.you => {
                            mine.insert(unit);
                        }
                        tbg_core::Event::Death { unit } => {
                            mine.remove(&unit);
                        }
                        _ => {}
                    }
                }
                let seen: std::collections::BTreeSet<_> = v
                    .units
                    .iter()
                    .filter(|u| u.seat == v.you)
                    .map(|u| u.id)
                    .collect();
                assert_eq!(mine, seen, "client and server armies diverged");
                let live = v.phase == tbg_core::Phase::Running;
                let cheapest = (0..v.offers.len())
                    .min_by_key(|&i| tbg_core::content::def(v.offers[i]).cost)
                    .unwrap_or(0);
                if live && v.energy >= tbg_core::content::def(v.offers[cheapest]).cost {
                    seq += 1;
                    let intent = PlayerIntention::Purchase {
                        offer: cheapest as u8,
                    };
                    send(&mut ws, &ClientMsg::Intent { seq, intent }).await;
                }
            }
            ServerMsg::Rejected { reason, .. } => {
                assert_eq!(reason, Rejection::NotEnoughEnergy, "only Energy gates");
                refused += 1;
            }
            ServerMsg::MatchEnd { .. } => {
                bot.abort();
                assert!(seq - refused > 8, "bought {} units", seq - refused);
                return;
            }
            _ => {}
        }
    }
    panic!("the match never ended");
}

#[tokio::test]
async fn challenge_pairs_owner_and_guest_without_code() {
    let url = server(30).await;
    let (_, _, _, _, owner_start, guest_start) = start_challenge(&url).await;
    assert_eq!(
        (owner_start.1, guest_start.1),
        (MatchMode::Challenge, MatchMode::Challenge)
    );
    assert_ne!(owner_start.0, guest_start.0);
}

#[tokio::test]
async fn challenge_only_one_guest_wins() {
    let url = server(30).await;
    let mut owner = connect(&url).await;
    let mut first = connect(&url).await;
    let mut second = connect(&url).await;
    hello(&mut owner).await;
    hello(&mut first).await;
    hello(&mut second).await;
    let invite = create_challenge(&mut owner).await;
    send(&mut first, &ClientMsg::JoinChallenge { token: invite }).await;
    send(&mut second, &ClientMsg::JoinChallenge { token: invite }).await;
    let (a, b) = tokio::join!(challenge_result(&mut first), challenge_result(&mut second));
    assert!(a.is_ok() ^ b.is_ok(), "{a:?} / {b:?}");
    let (_, mode) = challenge_start(&mut owner).await;
    assert_eq!(mode, MatchMode::Challenge);
}

#[tokio::test]
async fn challenge_rejects_third_guest_and_owner_token() {
    let url = server(30).await;
    let mut owner = connect(&url).await;
    let mut guest = connect(&url).await;
    hello(&mut owner).await;
    hello(&mut guest).await;
    let invite = create_challenge(&mut owner).await;
    send(&mut owner, &ClientMsg::JoinChallenge { token: invite }).await;
    assert_eq!(
        recv(&mut owner).await,
        Some(ServerMsg::ChallengeUnavailable)
    );
    send(&mut guest, &ClientMsg::JoinChallenge { token: invite }).await;
    let (_, mode) = challenge_start(&mut owner).await;
    assert_eq!(mode, MatchMode::Challenge);
    let _ = challenge_start(&mut guest).await;
    let mut third = connect(&url).await;
    hello(&mut third).await;
    send(&mut third, &ClientMsg::JoinChallenge { token: invite }).await;
    assert_eq!(
        recv(&mut third).await,
        Some(ServerMsg::ChallengeUnavailable)
    );
}

#[tokio::test]
async fn challenge_cancel_removes_invite() {
    let url = server(30).await;
    let mut owner = connect(&url).await;
    let mut guest = connect(&url).await;
    hello(&mut owner).await;
    hello(&mut guest).await;
    let old = create_challenge(&mut owner).await;
    send(&mut owner, &ClientMsg::Cancel).await;
    let fresh = create_challenge(&mut owner).await;
    assert_ne!(old, fresh, "cancel must remove the old invite");
    send(&mut guest, &ClientMsg::JoinChallenge { token: old }).await;
    assert_eq!(
        recv(&mut guest).await,
        Some(ServerMsg::ChallengeUnavailable)
    );
}

#[tokio::test]
async fn challenge_owner_reload_resumes_waiting() {
    let url = server(30).await;
    let mut owner = connect(&url).await;
    let token = hello_token(&mut owner).await;
    let invite = create_challenge(&mut owner).await;
    drop(owner);
    tokio::time::sleep(Duration::from_millis(100)).await;

    let mut guest = connect(&url).await;
    hello(&mut guest).await;
    send(&mut guest, &ClientMsg::JoinChallenge { token: invite }).await;
    assert_eq!(
        recv(&mut guest).await,
        Some(ServerMsg::ChallengeUnavailable),
        "a reloading host must not start a ghost match"
    );

    let mut owner = connect(&url).await;
    hello_resume(&mut owner, token).await;
    assert_eq!(
        recv(&mut owner).await,
        Some(ServerMsg::ChallengeWaiting { token: invite })
    );
    send(&mut guest, &ClientMsg::JoinChallenge { token: invite }).await;
    let (_, mode) = challenge_start(&mut owner).await;
    assert_eq!(mode, MatchMode::Challenge);
    let _ = challenge_start(&mut guest).await;
}

#[tokio::test]
async fn challenge_guest_reload_resumes_match() {
    let url = server(30).await;
    let (mut owner, guest, _, guest_token, _, guest_start) = start_challenge(&url).await;
    let guest_seat = guest_start.0;
    drop(guest);
    tokio::time::sleep(Duration::from_millis(100)).await;

    let mut guest = connect(&url).await;
    hello_resume(&mut guest, guest_token).await;
    let (seat, mode) = challenge_start(&mut guest).await;
    assert_eq!((seat, mode), (guest_seat, MatchMode::Challenge));
    end_by_surrender(&mut guest, &mut owner).await;
}

#[tokio::test]
async fn challenge_expires() {
    let url = server_with(Config {
        tick_ms: 5,
        max_per_ip: 1_000,
        challenge_ttl_secs: 1,
        ..Config::default()
    })
    .await;
    let mut owner = connect(&url).await;
    let mut guest = connect(&url).await;
    hello(&mut owner).await;
    hello(&mut guest).await;
    let invite = create_challenge(&mut owner).await;
    tokio::time::sleep(Duration::from_millis(1_100)).await;
    send(&mut guest, &ClientMsg::JoinChallenge { token: invite }).await;
    assert_eq!(
        recv(&mut guest).await,
        Some(ServerMsg::ChallengeUnavailable)
    );
    assert_eq!(
        recv(&mut owner).await,
        Some(ServerMsg::ChallengeUnavailable)
    );
}

#[tokio::test]
async fn rematch_requires_both() {
    let url = server(30).await;
    let (mut owner, mut guest, _, _, _, _) = start_challenge(&url).await;
    end_by_surrender(&mut owner, &mut guest).await;
    let _ = rematch_open(&mut owner).await;
    let _ = rematch_open(&mut guest).await;
    send(&mut owner, &ClientMsg::RequestRematch).await;
    assert_eq!(rematch_state(&mut owner).await, (true, false));
    assert_eq!(rematch_state(&mut guest).await, (false, true));
    no_match_start(&mut owner, Duration::from_millis(100)).await;
    no_match_start(&mut guest, Duration::from_millis(100)).await;
    send(&mut guest, &ClientMsg::RequestRematch).await;
    let (a, b) = tokio::join!(challenge_start(&mut owner), challenge_start(&mut guest));
    assert_eq!((a.1, b.1), (MatchMode::Challenge, MatchMode::Challenge));
    assert_ne!(a.0, b.0);
}

#[tokio::test]
async fn rematch_duplicate_requests_idempotent() {
    let url = server(30).await;
    let (mut owner, mut guest, _, _, _, _) = start_challenge(&url).await;
    end_by_surrender(&mut owner, &mut guest).await;
    let _ = rematch_open(&mut owner).await;
    let _ = rematch_open(&mut guest).await;
    send(&mut owner, &ClientMsg::RequestRematch).await;
    assert_eq!(rematch_state(&mut owner).await, (true, false));
    assert_eq!(rematch_state(&mut guest).await, (false, true));
    send(&mut owner, &ClientMsg::RequestRematch).await;
    assert_eq!(rematch_state(&mut owner).await, (true, false));
    assert_eq!(rematch_state(&mut guest).await, (false, true));
    no_match_start(&mut owner, Duration::from_millis(100)).await;
    send(&mut guest, &ClientMsg::RequestRematch).await;
    let (a, b) = tokio::join!(challenge_start(&mut owner), challenge_start(&mut guest));
    assert_eq!((a.1, b.1), (MatchMode::Challenge, MatchMode::Challenge));
    no_match_start(&mut owner, Duration::from_millis(100)).await;
    no_match_start(&mut guest, Duration::from_millis(100)).await;
}

#[tokio::test]
async fn rematch_decline_closes() {
    let url = server(30).await;
    let (mut owner, mut guest, _, _, _, _) = start_challenge(&url).await;
    end_by_surrender(&mut owner, &mut guest).await;
    let _ = rematch_open(&mut owner).await;
    let _ = rematch_open(&mut guest).await;
    send(&mut owner, &ClientMsg::RequestRematch).await;
    let _ = rematch_state(&mut owner).await;
    let _ = rematch_state(&mut guest).await;
    send(&mut guest, &ClientMsg::Cancel).await;
    assert_eq!(
        recv(&mut owner).await,
        Some(ServerMsg::RematchClosed { declined: true })
    );
    no_match_start(&mut owner, Duration::from_millis(100)).await;
    no_match_start(&mut guest, Duration::from_millis(100)).await;
}

#[tokio::test]
async fn rematch_reload_keeps_offer() {
    let url = server(30).await;
    let (mut owner, mut guest, owner_token, _, _, _) = start_challenge(&url).await;
    end_by_surrender(&mut owner, &mut guest).await;
    let _ = rematch_open(&mut owner).await;
    let _ = rematch_open(&mut guest).await;
    send(&mut owner, &ClientMsg::RequestRematch).await;
    let _ = rematch_state(&mut owner).await;
    let _ = rematch_state(&mut guest).await;
    drop(owner);
    tokio::time::sleep(Duration::from_millis(100)).await;

    send(&mut guest, &ClientMsg::RequestRematch).await;
    assert_eq!(rematch_state(&mut guest).await, (true, true));
    no_match_start(&mut guest, Duration::from_millis(100)).await;

    let mut owner = connect(&url).await;
    hello_resume(&mut owner, owner_token).await;
    let (a, b) = tokio::join!(challenge_start(&mut owner), challenge_start(&mut guest));
    assert_eq!((a.1, b.1), (MatchMode::Challenge, MatchMode::Challenge));
    assert_ne!(a.0, b.0);
}

#[tokio::test]
async fn rematch_does_not_enter_public_queue() {
    let url = server(30).await;
    let (mut owner, mut guest, _, _, _, _) = start_challenge(&url).await;
    end_by_surrender(&mut owner, &mut guest).await;
    let _ = rematch_open(&mut owner).await;
    let _ = rematch_open(&mut guest).await;
    send(&mut owner, &ClientMsg::RequestRematch).await;
    let _ = rematch_state(&mut owner).await;
    let _ = rematch_state(&mut guest).await;

    let mut third = connect(&url).await;
    hello(&mut third).await;
    send(&mut third, &ClientMsg::QuickMatch).await;
    assert_eq!(recv(&mut third).await, Some(ServerMsg::Queued));
    no_match_start(&mut owner, Duration::from_millis(100)).await;
    no_match_start(&mut guest, Duration::from_millis(100)).await;

    send(&mut guest, &ClientMsg::RequestRematch).await;
    let (a, b) = tokio::join!(challenge_start(&mut owner), challenge_start(&mut guest));
    assert_eq!((a.1, b.1), (MatchMode::Challenge, MatchMode::Challenge));
    no_match_start(&mut third, Duration::from_millis(100)).await;
}

#[tokio::test]
async fn two_guests_same_ip_join_different_challenges() {
    let url = server(30).await;
    let mut owner_a = connect(&url).await;
    let mut owner_b = connect(&url).await;
    let mut guest_a = connect(&url).await;
    let mut guest_b = connect(&url).await;
    hello(&mut owner_a).await;
    hello(&mut owner_b).await;
    hello(&mut guest_a).await;
    hello(&mut guest_b).await;
    let invite_a = create_challenge(&mut owner_a).await;
    let invite_b = create_challenge(&mut owner_b).await;
    send(&mut guest_a, &ClientMsg::JoinChallenge { token: invite_a }).await;
    send(&mut guest_b, &ClientMsg::JoinChallenge { token: invite_b }).await;
    let (a, b, c, d) = tokio::join!(
        challenge_start(&mut owner_a),
        challenge_start(&mut guest_a),
        challenge_start(&mut owner_b),
        challenge_start(&mut guest_b)
    );
    assert_eq!(
        (a.1, b.1, c.1, d.1),
        (
            MatchMode::Challenge,
            MatchMode::Challenge,
            MatchMode::Challenge,
            MatchMode::Challenge
        )
    );
}

#[tokio::test]
async fn challenge_invite_rate_limit() {
    let url = server_with(Config {
        tick_ms: 5,
        max_per_ip: 1_000,
        invite_per_ip_min: 2,
        ..Config::default()
    })
    .await;
    let mut guest = connect(&url).await;
    hello(&mut guest).await;
    for token in [[1; 16], [2; 16]] {
        send(&mut guest, &ClientMsg::JoinChallenge { token }).await;
        assert_eq!(
            recv(&mut guest).await,
            Some(ServerMsg::ChallengeUnavailable)
        );
    }
    send(&mut guest, &ClientMsg::JoinChallenge { token: [3; 16] }).await;
    assert_eq!(
        recv(&mut guest).await,
        Some(ServerMsg::Error {
            code: ErrorCode::RateLimited
        })
    );
}

#[tokio::test]
async fn challenge_freezes_arcanas_through_reconnect_and_rematch() {
    // BR-AR05 + BR-R08/R09: frozen presets survive reconnect and rematch;
    // the rematch runs a new server seed without joining a public queue.
    use tbg_core::protocol::ArcanaRejection;
    use tbg_core::ArcanaPreset;

    let (replay_tx, mut replays) = tokio::sync::mpsc::unbounded_channel();
    let url = server_with(Config {
        tick_ms: 5,
        max_per_ip: 1_000,
        replays: Some(replay_tx),
        ..Config::default()
    })
    .await;
    let (mut owner, mut guest) = (connect(&url).await, connect(&url).await);
    hello(&mut owner).await;
    let guest_token = hello_token(&mut guest).await;
    let presets = [
        ArcanaPreset::complete([2, 2, 2, 2, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0]).unwrap(),
        ArcanaPreset::complete([2, 2, 0, 0, 2, 2, 0, 0, 0, 0, 0, 0, 1, 1, 1]).unwrap(),
    ];
    for (ws, preset) in [(&mut owner, presets[0]), (&mut guest, presets[1])] {
        send(
            ws,
            &ClientMsg::SetArcanas {
                ranks: preset.ranks,
            },
        )
        .await;
        assert_eq!(recv(ws).await, Some(ServerMsg::ArcanasSelected { preset }));
    }
    let invite = create_challenge(&mut owner).await;
    send(
        &mut owner,
        &ClientMsg::SetArcanas {
            ranks: ArcanaPreset::DEFAULT.ranks,
        },
    )
    .await;
    assert_eq!(
        recv(&mut owner).await,
        Some(ServerMsg::ArcanasRejected {
            reason: ArcanaRejection::NotIdle
        })
    );
    send(&mut guest, &ClientMsg::JoinChallenge { token: invite }).await;
    for ws in [&mut owner, &mut guest] {
        assert_challenge_presets(ws, presets).await;
    }
    send(
        &mut guest,
        &ClientMsg::SetArcanas {
            ranks: ArcanaPreset::DEFAULT.ranks,
        },
    )
    .await;
    loop {
        match recv(&mut guest).await {
            Some(ServerMsg::ArcanasRejected { reason }) => {
                assert_eq!(reason, ArcanaRejection::NotIdle);
                break;
            }
            Some(_) => {}
            None => panic!("socket closed before ArcanasRejected"),
        }
    }
    drop(guest);
    let mut guest = connect(&url).await;
    hello_resume(&mut guest, guest_token).await;
    assert_challenge_presets(&mut guest, presets).await;
    end_by_surrender(&mut owner, &mut guest).await;
    rematch_open(&mut owner).await;
    rematch_open(&mut guest).await;
    send(&mut owner, &ClientMsg::RequestRematch).await;
    send(&mut guest, &ClientMsg::RequestRematch).await;
    for ws in [&mut owner, &mut guest] {
        assert_challenge_presets(ws, presets).await;
    }
    end_by_surrender(&mut guest, &mut owner).await;
    let first = tokio::time::timeout(Duration::from_secs(2), replays.recv())
        .await
        .unwrap()
        .unwrap();
    let second = tokio::time::timeout(Duration::from_secs(2), replays.recv())
        .await
        .unwrap()
        .unwrap();
    assert_ne!(first.seed, second.seed, "BR-R09: a fresh server seed");
    assert_eq!(first.arcanas, presets);
    assert_eq!(second.arcanas, presets);
}

async fn assert_challenge_presets(ws: &mut Ws, expected: [tbg_core::ArcanaPreset; 2]) {
    loop {
        match recv(ws).await {
            Some(ServerMsg::MatchStart { mode, arcanas, .. }) => {
                assert_eq!(mode, MatchMode::Challenge);
                assert_eq!(arcanas, expected, "BR-AR05: frozen presets in seat order");
                return;
            }
            Some(_) => {}
            None => panic!("socket closed before MatchStart"),
        }
    }
}

async fn wait_for_round(ws: &mut Ws) -> tbg_core::MatchView {
    loop {
        match recv(ws).await {
            Some(ServerMsg::View(view)) if view.augment_round.is_some() => return *view,
            Some(_) => {}
            None => panic!("socket closed before the first Augment round"),
        }
    }
}

async fn wait_for_view(
    ws: &mut Ws,
    predicate: impl Fn(&tbg_core::MatchView) -> bool,
) -> tbg_core::MatchView {
    loop {
        match recv(ws).await {
            Some(ServerMsg::View(view)) if predicate(&view) => return *view,
            Some(_) => {}
            None => panic!("socket closed while waiting for a view"),
        }
    }
}

#[tokio::test]
async fn br_ag_online_paused_round_hidden_pick_reconnect_and_deadline() {
    let url = server(30).await;
    let mut a = connect(&url).await;
    let mut b = connect(&url).await;
    let a_token = hello_token(&mut a).await;
    let b_token = hello_token(&mut b).await;
    send(&mut a, &ClientMsg::QuickMatch).await;
    send(&mut b, &ClientMsg::QuickMatch).await;
    let (a_seat, b_seat) = tokio::join!(
        async {
            loop {
                match recv(&mut a).await {
                    Some(ServerMsg::MatchStart { you, .. }) => return you,
                    Some(_) => continue,
                    None => panic!("A never started"),
                }
            }
        },
        async {
            loop {
                match recv(&mut b).await {
                    Some(ServerMsg::MatchStart { you, .. }) => return you,
                    Some(_) => continue,
                    None => panic!("B never started"),
                }
            }
        }
    );
    // BR-AG12: both seats are paused on the same round and combat tick.
    let (a_view, b_view) = tokio::join!(wait_for_round(&mut a), wait_for_round(&mut b));
    let round = a_view.augment_round.clone().unwrap();
    let b_round = b_view.augment_round.clone().unwrap();
    assert_eq!(
        (round.round, round.options),
        (b_round.round, b_round.options)
    );
    assert_eq!(a_view.tick, b_view.tick, "frozen on the same combat tick");
    assert_eq!(a_seat, a_view.you);
    assert_eq!(b_seat, b_view.you);
    assert!(
        a_view.regen_bp >= 10_000 && a_view.regen_bp <= 22_500,
        "Interest boost {}",
        a_view.regen_bp
    );
    let pause_tick = a_view.tick;

    // Gameplay is rejected while paused.
    let mut seq = 1;
    send(
        &mut a,
        &ClientMsg::Intent {
            seq,
            intent: PlayerIntention::Purchase { offer: 0 },
        },
    )
    .await;
    expect_rejection(&mut a, seq, Rejection::Paused).await;

    // One confirmation: A waits, B sees nothing of it (BR-AG14).
    seq += 1;
    let intent = PlayerIntention::ChooseAugment {
        round: round.round,
        option: 0,
    };
    let id = usize::from(round.options[0]);
    send(&mut a, &ClientMsg::Intent { seq, intent }).await;
    let waiting = wait_for_view(&mut a, |v| {
        v.augment_round.as_ref().is_some_and(|r| r.resolved)
    })
    .await;
    assert_eq!(waiting.augments[a_seat.index()].get(id), 0, "hidden");
    assert_eq!(waiting.tick, pause_tick, "one choice never resumes");
    let b_sees = wait_for_view(&mut b, |v| v.clock > waiting.clock).await;
    assert!(!b_sees.augment_round.as_ref().unwrap().resolved);
    assert_eq!(b_sees.augments[a_seat.index()].get(id), 0);
    seq += 1;
    send(&mut a, &ClientMsg::Intent { seq, intent }).await;
    expect_rejection(&mut a, seq, Rejection::RoundClosed).await;
    // A replay of an accepted seq is stale and cannot buy twice.
    send(&mut a, &ClientMsg::Intent { seq: 2, intent }).await;
    expect_rejection(&mut a, 2, Rejection::Stale).await;

    // Reconnecting inside the pause restores the round, the remaining
    // time and the confirmation.
    drop(a);
    let mut a2 = connect(&url).await;
    hello_resume(&mut a2, a_token).await;
    let resumed = wait_for_view(&mut a2, |v| v.augment_round.is_some()).await;
    let r = resumed.augment_round.unwrap();
    assert!(r.resolved && r.ticks_left < round.ticks_left);
    assert_eq!(resumed.tick, pause_tick);

    // B disconnects without choosing: the selection clock still runs out
    // and the battle resumes for A (BR-AG13).
    drop(b);
    let after = wait_for_view(&mut a2, |v| v.augment_round.is_none()).await;
    assert_eq!(after.augments[a_seat.index()].get(id), 1, "pick applied");
    assert!(after.events.iter().any(|e| matches!(
        e,
        tbg_core::Event::Forgone { seat, .. } if *seat == b_seat
    )));
    let running = wait_for_view(&mut a2, |v| v.tick > pause_tick).await;
    assert!(running.augment_round.is_none());

    // B comes back to the resumed battle and sees A's public copy.
    let mut b2 = connect(&url).await;
    hello_resume(&mut b2, b_token).await;
    let b_back = wait_for_view(&mut b2, |v| v.tick > pause_tick).await;
    assert_eq!(b_back.augments[a_seat.index()].get(id), 1);

    send(&mut a2, &ClientMsg::Surrender).await;
    let _ = await_end(&mut b2).await;
}

async fn expect_rejection(ws: &mut Ws, seq: u32, reason: Rejection) {
    loop {
        match recv(ws).await {
            Some(ServerMsg::Rejected { seq: s, reason: r }) => {
                assert_eq!((s, r), (seq, reason));
                return;
            }
            Some(_) => continue,
            None => panic!("socket closed before the rejection"),
        }
    }
}

#[tokio::test]
async fn br_sk01_kinds_freeze_at_pairing_fortify_online_and_survive_reconnect() {
    // D-46: choose while idle (Classic refused), frozen once in a room,
    // both towers public in every view, Fortify validated by the server,
    // tier kept across a reconnect, and the replay records both kinds.
    use tbg_core::protocol::StrongholdRejection;
    use tbg_core::{Phase, StrongholdKind};

    let (replay_tx, mut replays) = tokio::sync::mpsc::unbounded_channel();
    let url = server_with(Config {
        tick_ms: 5,
        max_per_ip: 1_000,
        reconnect_secs: 30,
        replays: Some(replay_tx),
        ..Config::default()
    })
    .await;
    let (mut owner, mut guest) = (connect(&url).await, connect(&url).await);
    hello(&mut owner).await;
    let guest_token = hello_token(&mut guest).await;
    send(
        &mut owner,
        &ClientMsg::SetStronghold {
            kind: StrongholdKind::Classic,
        },
    )
    .await;
    assert_eq!(
        recv(&mut owner).await,
        Some(ServerMsg::StrongholdRejected {
            reason: StrongholdRejection::NotSelectable
        })
    );
    for (ws, kind) in [
        (&mut owner, StrongholdKind::Artillery),
        (&mut guest, StrongholdKind::Command),
    ] {
        send(ws, &ClientMsg::SetStronghold { kind }).await;
        assert_eq!(recv(ws).await, Some(ServerMsg::StrongholdSelected { kind }));
    }
    let invite = create_challenge(&mut owner).await;
    send(
        &mut owner,
        &ClientMsg::SetStronghold {
            kind: StrongholdKind::Bastion,
        },
    )
    .await;
    assert_eq!(
        recv(&mut owner).await,
        Some(ServerMsg::StrongholdRejected {
            reason: StrongholdRejection::NotIdle
        })
    );
    send(&mut guest, &ClientMsg::JoinChallenge { token: invite }).await;
    let (owner_seat, _) = challenge_start(&mut owner).await;
    challenge_start(&mut guest).await;
    let mut kinds = [StrongholdKind::Command; 2];
    kinds[owner_seat.index()] = StrongholdKind::Artillery;
    let running = |v: &tbg_core::MatchView| v.phase == Phase::Running;
    let view = wait_for_view(&mut owner, running).await;
    assert_eq!(
        view.strongholds.map(|s| s.kind),
        kinds,
        "BR-SK08: both public"
    );
    // 5 starting Energy buys tier 1 (3 Energy) on the Rapid Fire branch.
    let intent = PlayerIntention::Fortify { branch: 1 };
    send(&mut owner, &ClientMsg::Intent { seq: 1, intent }).await;
    let me = owner_seat.index();
    let fortified = |v: &tbg_core::MatchView| v.strongholds[me].tier == 1;
    let view = wait_for_view(&mut owner, fortified).await;
    assert_eq!(view.strongholds[me].branch, 1);
    // The opponent sees it too, also after a reconnect.
    drop(guest);
    let mut guest = connect(&url).await;
    hello_resume(&mut guest, guest_token).await;
    let view = wait_for_view(&mut guest, fortified).await;
    assert_eq!(view.strongholds.map(|s| s.kind), kinds);
    end_by_surrender(&mut owner, &mut guest).await;
    let replay = tokio::time::timeout(Duration::from_secs(2), replays.recv())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(replay.strongholds, kinds);
    assert_eq!(
        replay.run().players[me].fort_tier,
        1,
        "replays rebuild the tier"
    );
}

#[tokio::test]
async fn celebration_finished_result_replays_to_either_seat_without_a_live_view() {
    // BR-VC02/05: the disconnected winner receives the frozen aggregate, not a survivor reconstruction.
    for owner_wins in [true, false] {
        let url = server(3).await;
        let (owner, guest, owner_token, guest_token, owner_start, guest_start) =
            start_challenge(&url).await;
        let (mut winner, mut loser, token, seat) = if owner_wins {
            (owner, guest, owner_token, owner_start.0)
        } else {
            (guest, owner, guest_token, guest_start.0)
        };
        let offer = loop {
            if let Some(ServerMsg::View(view)) = recv(&mut winner).await {
                if view.phase == tbg_core::Phase::Running {
                    if let Some(i) = view
                        .offers
                        .iter()
                        .position(|k| tbg_core::ruleset::CANON.def(*k).cost <= view.energy)
                    {
                        break i as u8;
                    }
                }
            }
        };
        send(
            &mut winner,
            &ClientMsg::Intent {
                seq: 1,
                intent: PlayerIntention::Purchase { offer },
            },
        )
        .await;
        loop {
            if let Some(ServerMsg::View(v)) = recv(&mut winner).await {
                if v.last_seq == 1 {
                    break;
                }
            }
        }
        winner.close(None).await.unwrap();
        send(&mut loser, &ClientMsg::Surrender).await;
        loop {
            if let Some(ServerMsg::MatchEnd { outcome, cast, .. }) = recv(&mut loser).await {
                assert_eq!(outcome.winner, Some(seat));
                assert!(cast.is_none());
                break;
            }
        }
        let mut recovered = connect(&url).await;
        hello_resume(&mut recovered, token).await;
        let mut started = None;
        let frozen = loop {
            match recv(&mut recovered).await {
                Some(ServerMsg::MatchStart { result_id, you, .. }) => {
                    assert_eq!(you, seat);
                    started = Some(result_id);
                }
                Some(ServerMsg::MatchEnd {
                    result_id,
                    cast,
                    outcome,
                    ..
                }) => {
                    assert_eq!(Some(result_id), started);
                    let cast = cast.unwrap();
                    assert_eq!(cast.decode(seat).unwrap().total(seat), Some(1));
                    assert_eq!(outcome.winner, Some(seat));
                    break (result_id, cast);
                }
                Some(_) => {}
                None => panic!("completed result was lost"),
            }
        };
        recovered.close(None).await.unwrap();
        let mut again = connect(&url).await;
        hello_resume(&mut again, token).await;
        loop {
            if let Some(ServerMsg::MatchEnd {
                result_id, cast, ..
            }) = recv(&mut again).await
            {
                assert_eq!((result_id, cast.unwrap()), frozen);
                break;
            }
        }
    }
}
