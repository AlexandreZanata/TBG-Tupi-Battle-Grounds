//! P71 (BR-M03, Proposed): a dropped player pauses the match for both players
//! within a per-player budget; coming back resumes it exactly; spending the
//! budget restores the ordinary reconnect window and its forfeit (BR-R04).

use std::time::{Duration, Instant};

use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

use tbg_core::protocol::{self, ClientMsg, ServerMsg, Token, PROTOCOL_VERSION};
use tbg_core::{EndReason, Phase, PlayerIntention, Rejection};
use tbg_server::Config;

type Ws = WebSocketStream<MaybeTlsStream<TcpStream>>;

async fn server(pause_secs: u64) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}/ws", listener.local_addr().unwrap());
    let config = Config {
        tick_ms: 5,
        reconnect_secs: 1,
        pause_secs,
        max_per_ip: 100,
        ..Config::default()
    };
    tokio::spawn(tbg_server::run(config, listener));
    url
}

async fn send(ws: &mut Ws, msg: &ClientMsg) {
    ws.send(Message::Binary(protocol::encode(msg).into()))
        .await
        .unwrap();
}

/// Next server message, answering pings.
async fn recv(ws: &mut Ws) -> ServerMsg {
    loop {
        let frame = tokio::time::timeout(Duration::from_secs(10), ws.next())
            .await
            .expect("server silent")
            .expect("socket closed")
            .unwrap();
        let Message::Binary(bytes) = frame else {
            continue;
        };
        match protocol::decode_server(&bytes).unwrap() {
            ServerMsg::Ping { id } => send(ws, &ClientMsg::Pong { id }).await,
            msg => return msg,
        }
    }
}

async fn until(ws: &mut Ws, want: impl Fn(&ServerMsg) -> bool) -> ServerMsg {
    loop {
        let msg = recv(ws).await;
        if want(&msg) {
            return msg;
        }
    }
}

async fn join(url: &str, resume: Option<Token>) -> (Ws, Token) {
    let mut ws = connect_async(url).await.unwrap().0;
    send(
        &mut ws,
        &ClientMsg::Hello {
            version: PROTOCOL_VERSION,
            resume,
        },
    )
    .await;
    let ServerMsg::Welcome { token, .. } =
        until(&mut ws, |m| matches!(m, ServerMsg::Welcome { .. })).await
    else {
        unreachable!()
    };
    (ws, token)
}

/// Two guests in a running casual match.
async fn running(url: &str) -> (Ws, Token, Ws) {
    let (mut a, token) = join(url, None).await;
    let (mut b, _) = join(url, None).await;
    send(&mut a, &ClientMsg::QuickMatch).await;
    send(&mut b, &ClientMsg::QuickMatch).await;
    until(&mut a, |m| matches!(m, ServerMsg::MatchStart { .. })).await;
    until(
        &mut b,
        |m| matches!(m, ServerMsg::View(v) if v.phase == Phase::Running),
    )
    .await;
    (a, token, b)
}

/// The clock of the next view.
async fn clock(ws: &mut Ws) -> u32 {
    let ServerMsg::View(view) = until(ws, |m| matches!(m, ServerMsg::View(_))).await else {
        unreachable!()
    };
    view.clock
}

#[tokio::test]
async fn p71_br_m03_a_drop_pauses_both_clocks_and_a_return_resumes() {
    let url = server(30).await;
    let (a, token, mut b) = running(&url).await;
    drop(a);
    // Views keep coming while the match stands still for both players.
    tokio::time::sleep(Duration::from_millis(300)).await;
    let mut frozen = Vec::new();
    for _ in 0..8 {
        frozen.push(clock(&mut b).await);
    }
    let last = *frozen.last().unwrap();
    assert!(
        frozen.iter().rev().take(4).all(|c| *c == last),
        "{frozen:?}"
    );
    send(
        &mut b,
        &ClientMsg::Intent {
            seq: 1,
            intent: PlayerIntention::Purchase { offer: 0 },
        },
    )
    .await;
    let rejected = until(&mut b, |m| matches!(m, ServerMsg::Rejected { .. })).await;
    assert!(matches!(
        rejected,
        ServerMsg::Rejected {
            reason: Rejection::Paused,
            ..
        }
    ));
    let (mut back, again) = join(&url, Some(token)).await;
    assert_eq!(again, token);
    until(&mut back, |m| matches!(m, ServerMsg::MatchStart { .. })).await;
    let resumed = clock(&mut b).await;
    let later = loop {
        let c = clock(&mut b).await;
        if c > resumed + 4 {
            break c;
        }
    };
    assert!(later > last, "the clock runs again after the return");
}

#[tokio::test]
async fn p71_br_m03_spending_the_budget_restores_the_window_and_its_forfeit() {
    let url = server(1).await;
    let (a, _, mut b) = running(&url).await;
    let ServerMsg::View(v) = until(&mut b, |m| matches!(m, ServerMsg::View(_))).await else {
        unreachable!()
    };
    let me = v.you;
    let dropped = Instant::now();
    drop(a);
    let ServerMsg::MatchEnd { outcome, .. } =
        until(&mut b, |m| matches!(m, ServerMsg::MatchEnd { .. })).await
    else {
        unreachable!()
    };
    assert_eq!(outcome.reason, EndReason::Forfeit);
    assert_eq!(outcome.winner, Some(me));
    // One second of pause, then the one-second window.
    assert!(
        dropped.elapsed() >= Duration::from_millis(1900),
        "{:?}",
        dropped.elapsed()
    );
}
