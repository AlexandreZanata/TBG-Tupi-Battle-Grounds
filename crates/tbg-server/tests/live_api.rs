//! P47 HTTP contract (BR-LV06, ARCHITECTURE §11): `/api/live` and
//! `/api/online` answer from the snapshot, carry their own caching headers,
//! leak no ids, IPs or tokens, and respect the per-IP API limit.

use std::time::Duration;

use futures_util::SinkExt;
use tokio::net::TcpListener;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use tbg_core::protocol::{self, ClientMsg, PROTOCOL_VERSION};
use tbg_server::Config;

async fn server_with(config: Config) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(tbg_server::run(config, listener));
    format!("http://{addr}")
}

/// Raw HTTP/1.0 GET; returns (status, head, body).
async fn get(base: &str, path: &str) -> (u16, String, Vec<u8>) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let host = base.strip_prefix("http://").unwrap();
    let mut stream = tokio::net::TcpStream::connect(host).await.unwrap();
    let req = format!("GET {path} HTTP/1.0\r\nHost: {host}\r\nConnection: close\r\n\r\n");
    stream.write_all(req.as_bytes()).await.unwrap();
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).await.unwrap();
    let head_end = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map_or(raw.len(), |i| i + 4);
    let head = String::from_utf8_lossy(&raw[..head_end]).into_owned();
    let status = head
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    (status, head, raw[head_end..].to_vec())
}

/// Hello on a socket that stays open: the snapshot then counts one player.
async fn open_session(base: &str) -> impl std::any::Any {
    let url = format!("{}/ws", base.replace("http://", "ws://"));
    let (mut ws, _) = connect_async(url).await.unwrap();
    let hello = ClientMsg::Hello {
        version: PROTOCOL_VERSION,
        resume: None,
    };
    ws.send(Message::Binary(protocol::encode(&hello).into()))
        .await
        .unwrap();
    ws
}

async fn until_players(base: &str, want: u64) -> serde_json::Value {
    for _ in 0..100 {
        let (_, _, body) = get(base, "/api/live").await;
        let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
        if v["online"]["players"] == want {
            return v;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("the snapshot never reached {want} players");
}

#[tokio::test]
async fn live_answers_counts_an_empty_featured_and_no_cache() {
    let base = server_with(Config {
        db: None,
        ..Config::default()
    })
    .await;
    let _ws = open_session(&base).await;
    let v = until_players(&base, 1).await;
    assert_eq!(v["online"]["accounts"], 0);
    assert_eq!(v["online"]["in_battle"], 0);
    assert_eq!(v["online"]["watching"], 0);
    assert_eq!(v["featured"], serde_json::Value::Null);
    assert_eq!(v["matches"], serde_json::json!([]));

    let (status, head, body) = get(&base, "/api/live").await;
    assert_eq!(status, 200);
    assert!(
        head.to_ascii_lowercase()
            .contains("cache-control: no-cache, max-age=0"),
        "nginx must not cache the live route: {head}"
    );
    assert!(body.len() <= 16 * 1024, "BR-LV06 budget: 16 KB");
    let raw = String::from_utf8_lossy(&body);
    assert!(!raw.contains("token") && !raw.contains("127.0.0.1"));
}

#[tokio::test]
async fn online_is_paged_cached_and_never_lists_guests() {
    let base = server_with(Config {
        db: None,
        ..Config::default()
    })
    .await;
    let _ws = open_session(&base).await;
    until_players(&base, 1).await;
    let (status, head, body) = get(&base, "/api/online?offset=0&limit=5").await;
    assert_eq!(status, 200);
    assert!(head
        .to_ascii_lowercase()
        .contains("cache-control: public, max-age=5"));
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["total"], 0, "a guest is counted by /api/live only");
    assert_eq!(v["rows"], serde_json::json!([]));
}

#[tokio::test]
async fn the_live_routes_share_the_per_ip_api_limit() {
    let base = server_with(Config {
        db: None,
        api_per_ip_min: 3,
        ..Config::default()
    })
    .await;
    for _ in 0..3 {
        assert_eq!(get(&base, "/api/live").await.0, 200);
    }
    assert_eq!(get(&base, "/api/online").await.0, 429);
}
