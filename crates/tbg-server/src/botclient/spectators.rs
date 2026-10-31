//! Anonymous spectators for the load tool (P47 step 6, `tbg-bots --spectators
//! N`): each one watches a random listed match, reads its frames, and switches
//! after a dwell or when the match ends. One shared poller reads `/api/live`
//! for all of them, so N spectators cost one HTTP request per poll, not N.

use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::sync::watch;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use tbg_core::protocol::{self, ClientMsg, ServerMsg, PROTOCOL_VERSION};
use tbg_core::Pcg32;

/// How often the shared poller reads `/api/live`.
const POLL_EVERY: Duration = Duration::from_secs(2);
/// How long one spectator stays on a match before it picks another.
const DWELL: Duration = Duration::from_secs(20);

#[derive(Clone, Copy, Debug, Default)]
pub struct SpectatorStats {
    pub watch_starts: u64,
    pub refused: u64,
    pub views: u64,
    pub bytes: u64,
    pub ended: u64,
    pub errors: u64,
}

impl SpectatorStats {
    fn add(&mut self, other: SpectatorStats) {
        self.watch_starts += other.watch_starts;
        self.refused += other.refused;
        self.views += other.views;
        self.bytes += other.bytes;
        self.ended += other.ended;
        self.errors += other.errors;
    }
}

/// Run `count` spectators until `stop` is set; returns their totals.
pub async fn run(url: String, count: u64, stop: Arc<AtomicBool>) -> SpectatorStats {
    let base = http_base(&url);
    let (ids_tx, ids_rx) = watch::channel(Vec::<u64>::new());
    let poller = tokio::spawn(poll_listed(base, ids_tx, stop.clone()));
    let mut tasks = Vec::new();
    for i in 0..count {
        let rx = ids_rx.clone();
        let stop = stop.clone();
        let url = url.clone();
        tasks.push(tokio::spawn(async move {
            let mut rng = Pcg32::new(0x5bec + i, 7);
            watch_loop(url, rx, stop, &mut rng).await
        }));
    }
    let mut total = SpectatorStats::default();
    for t in tasks {
        total.add(t.await.unwrap_or_default());
    }
    poller.abort();
    total
}

fn http_base(ws_url: &str) -> String {
    let host = ws_url
        .strip_prefix("ws://")
        .and_then(|s| s.strip_suffix("/ws"))
        .unwrap_or("127.0.0.1:8080");
    format!("http://{host}")
}

/// Publish the listed match ids every `POLL_EVERY` until `stop`.
async fn poll_listed(base: String, ids: watch::Sender<Vec<u64>>, stop: Arc<AtomicBool>) {
    while !stop.load(Relaxed) {
        if let Some(v) = http_json(&base, "/api/live").await {
            let listed: Vec<u64> = v["matches"]
                .as_array()
                .map(|rows| {
                    rows.iter()
                        .filter_map(|r| r["id"].as_str())
                        .filter_map(|hex| u64::from_str_radix(hex, 16).ok())
                        .collect()
                })
                .unwrap_or_default();
            ids.send_replace(listed);
        }
        tokio::time::sleep(POLL_EVERY).await;
    }
}

/// One spectator: connect, then watch random listed matches until `stop`.
async fn watch_loop(
    url: String,
    ids: watch::Receiver<Vec<u64>>,
    stop: Arc<AtomicBool>,
    rng: &mut Pcg32,
) -> SpectatorStats {
    let mut stats = SpectatorStats::default();
    let Ok((mut ws, _)) = connect_async(&url).await else {
        stats.errors += 1;
        return stats;
    };
    let hello = ClientMsg::Hello {
        version: PROTOCOL_VERSION,
        resume: None,
    };
    if ws
        .send(Message::Binary(protocol::encode(&hello).into()))
        .await
        .is_err()
    {
        stats.errors += 1;
        return stats;
    }
    while !stop.load(Relaxed) {
        let listed = ids.borrow().clone();
        if listed.is_empty() {
            tokio::time::sleep(Duration::from_millis(250)).await;
            continue;
        }
        let id = listed[rng.below(listed.len() as u32) as usize];
        let send = ws
            .send(Message::Binary(
                protocol::encode(&ClientMsg::Spectate { id }).into(),
            ))
            .await;
        if send.is_err() {
            stats.errors += 1;
            break;
        }
        let ok = dwell(&mut ws, &mut stats, &stop).await;
        let _ = ws
            .send(Message::Binary(
                protocol::encode(&ClientMsg::StopSpectating).into(),
            ))
            .await;
        if !ok {
            break;
        }
    }
    stats
}

/// Read frames for one watch. Returns `false` if the socket broke.
async fn dwell(
    ws: &mut tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    stats: &mut SpectatorStats,
    stop: &AtomicBool,
) -> bool {
    let until = tokio::time::Instant::now() + DWELL;
    while !stop.load(Relaxed) && tokio::time::Instant::now() < until {
        let next = tokio::time::timeout(Duration::from_millis(500), ws.next()).await;
        let Ok(next) = next else {
            continue;
        };
        let Some(Ok(msg)) = next else {
            stats.errors += 1;
            return false;
        };
        let Message::Binary(bytes) = msg else {
            continue;
        };
        stats.bytes += bytes.len() as u64;
        match protocol::decode_server(&bytes) {
            Ok(ServerMsg::SpectateStart { .. }) => stats.watch_starts += 1,
            Ok(ServerMsg::SpectatorView(_)) => stats.views += 1,
            Ok(ServerMsg::SpectateUnavailable { .. }) => {
                // The server allows one `Spectate` per second: back off first.
                stats.refused += 1;
                tokio::time::sleep(Duration::from_millis(1_100)).await;
                return true;
            }
            Ok(ServerMsg::SpectateEnd { .. }) => {
                stats.ended += 1;
                return true;
            }
            Ok(ServerMsg::Ping { id }) => {
                let pong = protocol::encode(&ClientMsg::Pong { id });
                let _ = ws.send(Message::Binary(pong.into())).await;
            }
            _ => {}
        }
    }
    true
}

/// Minimal HTTP/1.0 GET of one JSON document; `None` on any failure.
async fn http_json(base: &str, path: &str) -> Option<serde_json::Value> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let host = base.strip_prefix("http://")?;
    let mut stream = tokio::net::TcpStream::connect(host).await.ok()?;
    let req = format!("GET {path} HTTP/1.0\r\nHost: {host}\r\nConnection: close\r\n\r\n");
    stream.write_all(req.as_bytes()).await.ok()?;
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).await.ok()?;
    let body = raw.windows(4).position(|w| w == b"\r\n\r\n")? + 4;
    serde_json::from_slice(&raw[body..]).ok()
}
