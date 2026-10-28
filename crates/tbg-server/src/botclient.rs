//! WebSocket bot client used by `tbg-bots` and the integration tests:
//! quick-match, play with the `tbg-core` bot, answer pings, measure RTT and
//! view gaps, optionally add one-way delay/jitter and drop/rejoin once.

use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio::time::{sleep, sleep_until, timeout, Instant};
use tokio_tungstenite::{connect_async_with_config, tungstenite::Message};

use tbg_core::bot::Bot;
use tbg_core::protocol::{self, ClientMsg, Token, PROTOCOL_VERSION};
use tbg_core::{Outcome, Pcg32, Seat};

mod handle;
use handle::handle;

#[derive(Clone, Debug)]
pub struct BotOpts {
    pub url: String,
    /// Added to every send and every receive.
    pub delay_ms: u64,
    pub jitter_ms: u64,
    /// Close the socket this long after `MatchStart`...
    pub drop_after: Option<Duration>,
    /// ...and come back with the session token after this long (`None`: never).
    pub rejoin_after: Option<Duration>,
    pub timeout: Duration,
    /// P11: register a `Tb…` account first, then play logged in.
    pub accounts: bool,
}

impl BotOpts {
    pub fn new(url: impl Into<String>) -> BotOpts {
        BotOpts {
            url: url.into(),
            delay_ms: 0,
            jitter_ms: 0,
            drop_after: None,
            rejoin_after: None,
            timeout: Duration::from_secs(360),
            accounts: false,
        }
    }
}

#[derive(Debug, Default)]
pub struct BotResult {
    pub completed: bool,
    pub seat: Option<Seat>,
    pub outcome: Option<Outcome>,
    pub rtt_ms: Vec<u16>,
    pub view_gaps_ms: Vec<u32>,
    /// P9: tap-to-acknowledgement latency per intention (ms).
    pub ack_ms: Vec<u16>,
    /// P9: lobby queue wait per match (ms).
    pub queue_wait_ms: Vec<u16>,
    pub rejections: u32,
    pub dropped: bool,
    pub reconnected: bool,
    pub error: Option<String>,
}

pub(super) struct Session {
    pub(super) bot: Bot,
    pub(super) rng: Pcg32,
    pub(super) token: Option<Token>,
    pub(super) seq: u32,
    pub(super) last_view: Option<Instant>,
    pub(super) drop_at: Option<Instant>,
    /// P9: send time per unacknowledged intention seq.
    pub(super) pending: Vec<(u32, Instant)>,
    /// P9: when `QuickMatch` was sent for the current match.
    pub(super) queue_start: Option<Instant>,
    /// P11: a `Register` was sent for this connection (awaiting the key).
    pub(super) registering: bool,
}

pub(super) enum Step {
    Done,
    Dropped,
}

pub async fn play_one(opts: BotOpts, seed: u64) -> BotResult {
    let mut r = BotResult::default();
    let deadline = Instant::now() + opts.timeout;
    let mut s = Session {
        bot: Bot::new(seed),
        rng: Pcg32::new(seed, 99),
        token: None,
        seq: 0,
        last_view: None,
        drop_at: None,
        pending: Vec::new(),
        queue_start: None,
        registering: false,
    };
    loop {
        match connect_and_play(&opts, &mut s, &mut r, deadline).await {
            Ok(Step::Done) => return r,
            Ok(Step::Dropped) => match opts.rejoin_after {
                Some(wait) => {
                    sleep(wait).await;
                    r.reconnected = true;
                }
                None => return r,
            },
            Err(e) => {
                r.error = Some(e);
                return r;
            }
        }
    }
}

fn jitter(opts: &BotOpts, rng: &mut Pcg32) -> Duration {
    let j = rng.below(2 * opts.jitter_ms as u32 + 1) as i64 - opts.jitter_ms as i64;
    Duration::from_millis((opts.delay_ms as i64 + j).max(0) as u64)
}

async fn connect_and_play(
    opts: &BotOpts,
    s: &mut Session,
    r: &mut BotResult,
    deadline: Instant,
) -> Result<Step, String> {
    // P9: disable Nagle on the bot socket (`TCP_NODELAY`), like the server.
    let (ws, _) = timeout(
        Duration::from_secs(10),
        connect_async_with_config(opts.url.as_str(), None, true),
    )
    .await
    .map_err(|_| "connect timeout".to_string())?
    .map_err(|e| format!("connect: {e}"))?;
    let (mut write, mut read) = ws.split();
    let (in_tx, mut in_rx) = mpsc::unbounded_channel::<(Instant, Vec<u8>)>();
    let (out_tx, mut out_rx) = mpsc::unbounded_channel::<(Instant, Vec<u8>)>();
    let reader = tokio::spawn(async move {
        while let Some(Ok(msg)) = read.next().await {
            if let Message::Binary(b) = msg {
                if in_tx.send((Instant::now(), b.to_vec())).is_err() {
                    break;
                }
            }
        }
    });
    let writer = tokio::spawn(async move {
        while let Some((at, bytes)) = out_rx.recv().await {
            sleep_until(at).await;
            if write.send(Message::Binary(bytes.into())).await.is_err() {
                break;
            }
        }
        let _ = write.close().await;
    });
    let mut send = |s: &mut Session, msg: ClientMsg| {
        let at = Instant::now() + jitter(opts, &mut s.rng);
        let _ = out_tx.send((at, protocol::encode(&msg)));
    };
    send(
        s,
        ClientMsg::Hello {
            version: PROTOCOL_VERSION,
            resume: s.token,
        },
    );
    let result = loop {
        let drop_at = s.drop_at.unwrap_or(deadline + Duration::from_secs(3600));
        tokio::select! {
            got = in_rx.recv() => {
                let Some((arrived, bytes)) = got else { break Err("connection closed".to_string()) };
                sleep_until(arrived + jitter(opts, &mut s.rng)).await;
                let msg = protocol::decode_server(&bytes).map_err(|_| "bad server message".to_string())?;
                if let Some(step) = handle(msg, opts, s, r, &mut send)? {
                    break Ok(step);
                }
            }
            _ = sleep_until(drop_at) => {
                s.drop_at = None;
                r.dropped = true;
                break Ok(Step::Dropped);
            }
            _ = sleep_until(deadline) => break Err("timeout".to_string()),
        }
    };
    reader.abort();
    drop(out_tx);
    let _ = timeout(Duration::from_secs(2), writer).await;
    result
}
