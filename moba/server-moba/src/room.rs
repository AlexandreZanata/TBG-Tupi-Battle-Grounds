//! The room task: owns the match, ticks it at `TICK_HZ`, assigns seats and
//! fans out snapshots. Slow sockets lose frames instead of delaying others.

use std::time::Duration;

use axum::body::Bytes;
use tbg_moba_core::content::TICK_HZ;
use tbg_moba_core::protocol::{self, ServerMsg, Welcome, MOBA_PROTOCOL_VERSION};
use tbg_moba_core::state::Controller;
use tbg_moba_core::types::{Seat, Team};
use tbg_moba_core::{host, view, MobaIntention, MobaMatch};
use tokio::sync::{mpsc, oneshot};

/// Frames buffered per connection; a socket further behind drops frames.
pub const OUTBOX: usize = 64;
/// Ticks the final scoreboard stays up before a new match starts.
const RESTART_TICKS: u32 = 10 * TICK_HZ;

pub type ConnId = u64;

pub enum Command {
    Join {
        conn: ConnId,
        outbox: mpsc::Sender<Bytes>,
        reply: oneshot::Sender<Option<Welcome>>,
    },
    Leave {
        conn: ConnId,
    },
    Intention {
        conn: ConnId,
        intention: MobaIntention,
    },
}

#[derive(Clone)]
pub struct RoomHandle {
    pub commands: mpsc::Sender<Command>,
}

struct Client {
    conn: ConnId,
    seat: Seat,
    outbox: mpsc::Sender<Bytes>,
}

struct Room {
    seed: u64,
    game: MobaMatch,
    clients: Vec<Client>,
    inputs: Vec<(Seat, MobaIntention)>,
    /// Room ticks since start; the finished match stops counting its own.
    wall: u32,
    ended_at: Option<u32>,
}

pub fn spawn(seed: u64) -> RoomHandle {
    let (tx, rx) = mpsc::channel(1024);
    tokio::spawn(run(seed, rx));
    RoomHandle { commands: tx }
}

async fn run(seed: u64, mut rx: mpsc::Receiver<Command>) {
    let mut room = Room {
        seed,
        game: MobaMatch::new(seed),
        clients: Vec::new(),
        inputs: Vec::new(),
        wall: 0,
        ended_at: None,
    };
    let mut clock = tokio::time::interval(Duration::from_millis(1000 / TICK_HZ as u64));
    // A stalled host catches up by skipping, never by bursting ticks.
    clock.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tokio::select! {
            _ = clock.tick() => room.tick(),
            command = rx.recv() => match command {
                Some(command) => room.handle(command),
                None => return,
            },
        }
    }
}

impl Room {
    fn handle(&mut self, command: Command) {
        match command {
            Command::Join {
                conn,
                outbox,
                reply,
            } => {
                let welcome = self.free_seat().map(|seat| {
                    self.game.set_controller(seat, Controller::Human);
                    self.clients.push(Client { conn, seat, outbox });
                    tracing::info!(conn, seat = seat.0, "player joined");
                    Welcome {
                        protocol: MOBA_PROTOCOL_VERSION,
                        seat,
                        seed: self.game.seed,
                        tick_hz: TICK_HZ as u8,
                    }
                });
                let _ = reply.send(welcome);
            }
            Command::Leave { conn } => {
                if let Some(i) = self.clients.iter().position(|c| c.conn == conn) {
                    let client = self.clients.remove(i);
                    // A bot takes the seat back mid-match (M-27).
                    self.game.set_controller(client.seat, Controller::Bot);
                    tracing::info!(
                        conn,
                        seat = client.seat.0,
                        "player left; bot resumes the seat"
                    );
                }
            }
            Command::Intention { conn, intention } => {
                // The seat comes from the connection, never from the frame (M-05).
                if let Some(client) = self.clients.iter().find(|c| c.conn == conn) {
                    self.inputs.push((client.seat, intention));
                }
            }
        }
    }

    /// The lowest bot seat on the team with fewer humans.
    fn free_seat(&self) -> Option<Seat> {
        let humans = |team: Team| {
            self.clients
                .iter()
                .filter(|c| c.seat.team() == team)
                .count()
        };
        let order = if humans(Team::Red) < humans(Team::Blue) {
            [Team::Red, Team::Blue]
        } else {
            [Team::Blue, Team::Red]
        };
        order.into_iter().find_map(|team| {
            Seat::all().find(|&s| s.team() == team && !self.clients.iter().any(|c| c.seat == s))
        })
    }

    fn tick(&mut self) {
        self.wall += 1;
        if self
            .ended_at
            .is_some_and(|ended| self.wall - ended >= RESTART_TICKS)
        {
            self.restart();
        }
        for (seat, reject) in host::tick(&mut self.game, &mut self.inputs) {
            self.send_to(seat, &ServerMsg::Rejected(reject));
        }
        if self.game.outcome.is_some() && self.ended_at.is_none() {
            self.ended_at = Some(self.wall);
        }
        let world = Bytes::from(protocol::encode(&ServerMsg::World(view::world(&self.game))));
        for client in &self.clients {
            let private =
                protocol::encode(&ServerMsg::Private(view::private(&self.game, client.seat)));
            // try_send: a full outbox drops this tick for that socket only.
            let _ = client.outbox.try_send(world.clone());
            let _ = client.outbox.try_send(Bytes::from(private));
        }
    }

    /// A fresh match on a new seed; connected humans keep their seats.
    fn restart(&mut self) {
        self.seed = self.seed.wrapping_add(1);
        self.game = MobaMatch::new(self.seed);
        for client in &self.clients {
            self.game.set_controller(client.seat, Controller::Human);
        }
        self.ended_at = None;
        tracing::info!(seed = self.seed, "new match");
    }

    fn send_to(&self, seat: Seat, message: &ServerMsg) {
        if let Some(client) = self.clients.iter().find(|c| c.seat == seat) {
            let _ = client
                .outbox
                .try_send(Bytes::from(protocol::encode(message)));
        }
    }
}
