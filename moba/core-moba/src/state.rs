//! Authoritative match state. Units live in one id-ordered `Vec`, so every
//! pass over them is deterministic (M-03).

use serde::{Deserialize, Serialize};

use crate::content::{self, Stats};
use crate::geom::Pos;
use crate::grid::Grid;
use crate::map;
use crate::rng::Rng;
use crate::types::{Destination, Lane, Seat, Side, Team, TowerTier, UnitKind, SEATS};

/// How a Commander is steered; Auto unless its player issued a command.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandMode {
    Auto,
    /// Walk to the point ignoring enemies, then Hold there.
    Move(Pos),
    /// Fight anything near the point, within [`content::HOLD_LEASH`].
    Hold(Pos),
}

/// Per-unit behaviour state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Brain {
    /// Towers and the Stronghold: shoot, never move.
    Static,
    /// Walk the lane's waypoints towards the enemy Stronghold.
    Lane { lane: Lane, waypoint: u8 },
    /// Clear camps one by one, then join the mid lane.
    Jungle { camp: Option<u8> },
    /// Passive until hit; leashed to its camp.
    Neutral { camp: u8 },
    Commander {
        seat: Seat,
        mode: CommandMode,
        retreating: bool,
        camp: Option<u8>,
    },
}

/// Who dealt the last hit, for bounties.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Credit {
    pub side: Side,
    pub seat: Option<Seat>,
}

#[derive(Clone, Debug)]
pub struct Unit {
    pub id: u32,
    pub kind: UnitKind,
    pub side: Side,
    /// The player who bought it (Troops) or controls it (Commanders).
    pub owner: Option<Seat>,
    pub pos: Pos,
    pub hp: i32,
    pub max_hp: i32,
    pub stats: Stats,
    /// Percent damage bonus from the owner's upgrade tier.
    pub power: i32,
    pub brain: Brain,
    pub target: u32,
    pub cooldown: u16,
    pub heal_cooldown: u16,
    pub slowed_until: u32,
    /// Tick of the last attack fired, so views can animate it.
    pub last_attack: u32,
    pub last_hit: Option<Credit>,
}

impl Unit {
    pub fn alive(&self) -> bool {
        self.hp > 0
    }

    pub fn mobile(&self) -> bool {
        self.stats.speed > 0
    }

    pub fn team(&self) -> Option<Team> {
        self.side.team()
    }
}

/// Who drives a seat. Hosts feed bot seats through the same intention path.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Controller {
    Human,
    Bot,
}

#[derive(Clone, Debug)]
pub struct Player {
    pub controller: Controller,
    pub gold: u32,
    /// Lifetime gold, for the team totals on the scoreboard.
    pub gold_earned: u32,
    pub essence: u32,
    pub tier: u8,
    pub level: u8,
    pub xp: u32,
    pub kills: u16,
    pub deaths: u16,
    /// Live Commander unit id, 0 while dead.
    pub commander: u32,
    pub respawn_at: u32,
    pub mode: CommandMode,
    /// Last Destination chosen, echoed to the HUD.
    pub destination: Destination,
}

#[derive(Clone, Copy, Debug)]
pub struct Camp {
    /// Tick when the camp respawns; `None` while its monsters are up.
    pub respawn_at: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EndReason {
    StrongholdDestroyed,
    TimeLimit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Outcome {
    /// `None` is a draw.
    pub winner: Option<Team>,
    pub reason: EndReason,
}

/// Something worth telling players, drained by hosts each tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    CommanderKilled {
        victim: Seat,
        killer: Option<Seat>,
        by: Team,
    },
    TowerDestroyed {
        team: Team,
        tier: TowerTier,
        lane: Option<Lane>,
    },
    CampCleared {
        camp: u8,
        by: Option<Seat>,
    },
    TroopSent {
        seat: Seat,
        troop: crate::types::TroopKind,
        to: Destination,
    },
    LevelUp {
        seat: Seat,
        level: u8,
    },
    Upgraded {
        seat: Seat,
        tier: u8,
    },
    Ended(Outcome),
}

#[derive(Clone, Debug)]
pub struct MobaMatch {
    pub seed: u64,
    pub tick: u32,
    pub rng: Rng,
    pub next_id: u32,
    pub units: Vec<Unit>,
    pub players: Vec<Player>,
    pub camps: [Camp; map::CAMP_COUNT],
    /// Commander kills per team.
    pub score: [u16; 2],
    pub outcome: Option<Outcome>,
    pub events: Vec<Event>,
    pub(crate) grid: Grid,
}

impl MobaMatch {
    /// A fresh match: every seat starts as a bot until a host hands it to a human.
    pub fn new(seed: u64) -> Self {
        let player = Player {
            controller: Controller::Bot,
            gold: content::START_GOLD,
            gold_earned: content::START_GOLD,
            essence: 0,
            tier: 0,
            level: 1,
            xp: 0,
            kills: 0,
            deaths: 0,
            commander: 0,
            respawn_at: 0,
            mode: CommandMode::Auto,
            destination: Destination::Lane(Lane::Mid),
        };
        let mut m = Self {
            seed,
            tick: 0,
            rng: Rng::new(seed),
            next_id: 1,
            units: Vec::with_capacity(512),
            players: vec![player; SEATS],
            camps: [Camp {
                respawn_at: Some(content::secs(20)),
            }; map::CAMP_COUNT],
            score: [0; 2],
            outcome: None,
            events: Vec::new(),
            grid: Grid::default(),
        };
        for (team, tier, lane, pos) in map::towers() {
            m.spawn(
                UnitKind::Tower(tier, lane),
                Side::Team(team),
                None,
                pos,
                Brain::Static,
            );
        }
        for team in Team::ALL {
            m.spawn(
                UnitKind::Stronghold,
                Side::Team(team),
                None,
                map::stronghold(team),
                Brain::Static,
            );
        }
        for seat in Seat::all() {
            crate::sim::spawn::commander(&mut m, seat);
        }
        m
    }

    pub fn player(&self, seat: Seat) -> &Player {
        &self.players[seat.0 as usize]
    }

    pub fn player_mut(&mut self, seat: Seat) -> &mut Player {
        &mut self.players[seat.0 as usize]
    }

    pub fn set_controller(&mut self, seat: Seat, controller: Controller) {
        if seat.valid() {
            self.player_mut(seat).controller = controller;
            if controller == Controller::Bot {
                self.player_mut(seat).mode = CommandMode::Auto;
            }
        }
    }

    /// Adds a unit with full HP and returns its id.
    pub fn spawn(
        &mut self,
        kind: UnitKind,
        side: Side,
        owner: Option<Seat>,
        pos: Pos,
        brain: Brain,
    ) -> u32 {
        let stats = content::base_stats(kind);
        let id = self.next_id;
        self.next_id += 1;
        self.units.push(Unit {
            id,
            kind,
            side,
            owner,
            pos,
            hp: stats.hp,
            max_hp: stats.hp,
            stats,
            power: 0,
            brain,
            target: 0,
            cooldown: 0,
            heal_cooldown: 0,
            slowed_until: 0,
            last_attack: 0,
            last_hit: None,
        });
        id
    }

    /// Slot of a live unit id; units stay sorted by id.
    pub fn slot(&self, id: u32) -> Option<usize> {
        if id == 0 {
            return None;
        }
        self.units.binary_search_by_key(&id, |u| u.id).ok()
    }

    pub fn unit(&self, id: u32) -> Option<&Unit> {
        self.slot(id).map(|i| &self.units[i])
    }

    pub fn troops_alive(&self, seat: Seat) -> u8 {
        self.units
            .iter()
            .filter(|u| u.owner == Some(seat) && matches!(u.kind, UnitKind::Troop(_)))
            .count()
            .min(u8::MAX as usize) as u8
    }

    pub fn towers_left(&self, team: Team) -> u8 {
        self.units
            .iter()
            .filter(|u| u.side == Side::Team(team) && matches!(u.kind, UnitKind::Tower(..)))
            .count() as u8
    }

    pub fn team_gold(&self, team: Team) -> u32 {
        Seat::all()
            .filter(|s| s.team() == team)
            .map(|s| self.player(s).gold_earned)
            .sum()
    }

    /// FNV-1a over everything the rules read; equal hashes ⇔ equal play (M-03).
    pub fn state_hash(&self) -> u64 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        let mut mix = |v: i64| {
            for b in v.to_le_bytes() {
                h ^= b as u64;
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        };
        mix(self.tick as i64);
        mix(self.rng.state() as i64);
        for u in &self.units {
            for v in [
                u.id as i64,
                u.pos.x as i64,
                u.pos.y as i64,
                u.hp as i64,
                u.target as i64,
            ] {
                mix(v);
            }
        }
        for p in &self.players {
            for v in [p.gold as i64, p.essence as i64, p.xp as i64, p.tier as i64] {
                mix(v);
            }
        }
        h
    }
}
