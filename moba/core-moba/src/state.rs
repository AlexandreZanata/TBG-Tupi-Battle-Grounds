//! Authoritative match state. Units live in one id-ordered `Vec`, so every
//! pass over them is deterministic (M-03).

use serde::{Deserialize, Serialize};

use crate::content::{self, Stats};
use crate::geom::Pos;
use crate::grid::Grid;
use crate::map;
use crate::rng::Rng;
use crate::types::{Destination, Lane, Seat, Side, Team, TowerTier, UnitKind, SEATS};

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
    /// The player who bought this Troop; Seats have no avatar (M-65).
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
    pub kills: u16,
    pub deaths: u16,
    /// Last Destination chosen, echoed to the HUD.
    pub destination: Destination,
    /// The three Shop offers, locked-slot bits and the seat's own Shop stream.
    pub offers: [crate::types::Champion; content::OFFERS],
    pub locked: u8,
    pub shop_rng: Rng,
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
    /// Reserved P76 wire tag; no avatar exists in current matches.
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
        troop: crate::types::Champion,
        to: Destination,
    },
    /// Reserved P76 wire tag; avatar XP/levels were removed by M-65.
    LevelUp {
        seat: Seat,
        level: u8,
    },
    Upgraded {
        seat: Seat,
        tier: u8,
    },
    Ended(Outcome),
    /// Public Troop kill counts without hidden champion identity or coordinates.
    TroopKilled {
        victim: Seat,
        killer: Option<Seat>,
        by: Team,
    },
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
    /// Champion Troop kills per team (M-14/M-65).
    pub score: [u16; 2],
    pub outcome: Option<Outcome>,
    pub events: Vec<Event>,
    pub(crate) grid: Grid,
    /// Last observed HP for surviving structures; destruction is public map knowledge.
    pub(crate) structure_hp: [Vec<(u32, i32)>; 2],
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
            kills: 0,
            deaths: 0,
            destination: Destination::Lane(Lane::Mid),
            offers: [crate::types::POOL[0]; content::OFFERS],
            locked: 0,
            shop_rng: Rng::new(0),
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
            structure_hp: [Vec::new(), Vec::new()],
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
            // Each seat draws from its own stream, as in the 1v1 Shop (BR-S05).
            let p = m.player_mut(seat);
            p.shop_rng =
                Rng::new(seed ^ (u64::from(seat.0) + 1).wrapping_mul(0xA24B_AED4_963E_E407));
            p.offers = crate::shop::opening(&mut p.shop_rng, &content::slot_rules(seat.role()));
        }
        crate::vision::remember_structures(&mut m);
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
            for v in [
                p.gold as i64,
                p.essence as i64,
                p.tier as i64,
                p.locked as i64,
                p.shop_rng.state() as i64,
            ] {
                mix(v);
            }
        }
        h
    }
}
