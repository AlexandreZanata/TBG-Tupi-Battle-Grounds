//! Match state, units, outcomes, and the events a tick produces.

use serde::{Deserialize, Serialize};

use crate::arcana::ArcanaPreset;
use crate::augment::{AugmentCounts, AugmentState};
use crate::content::{self, Stats, OFFERS};
use crate::rng::Pcg32;
use crate::ruleset::Ruleset;
use crate::types::{Archetype, Faction, Seat, Status, Tick, UnitId, UnitKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Countdown,
    Running,
    Ended,
}

/// What an attack, heal, or event points at.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Target {
    Unit(UnitId),
    Stronghold(Seat),
}

/// Living conventional units per combat role, refreshed once per tick and
/// decremented on death. The conditional Arcanas read this instead of
/// scanning the army for every hit (BR-AR06).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ArmyComposition {
    pub tanks: u16,
    pub dps: u16,
}

impl ArmyComposition {
    fn add(&mut self, archetype: Archetype) {
        match archetype {
            Archetype::Tank => self.tanks = self.tanks.saturating_add(1),
            Archetype::Dps => self.dps = self.dps.saturating_add(1),
            _ => {}
        }
    }

    pub(crate) fn remove(&mut self, archetype: Archetype) {
        match archetype {
            Archetype::Tank => self.tanks = self.tanks.saturating_sub(1),
            Archetype::Dps => self.dps = self.dps.saturating_sub(1),
            _ => {}
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Unit {
    pub id: UnitId,
    pub seat: Seat,
    pub kind: UnitKind,
    pub tier: u8,
    pub branch: u8,
    pub x: i32,
    pub hp: i32,
    pub max_hp: i32,
    pub cooldown: u16,
    pub windup: Option<(Target, u16)>,
    pub status: Status,
    /// The Nester that hatched this summon (BR-U08); `None` for bought units.
    pub summoner: Option<UnitId>,
    /// Tick at which a summon disappears.
    pub expires_at: Option<Tick>,
    /// Ticks until a Nester's next hatch (0 for every other kind).
    pub action_cd: u16,
    /// Forced March: move bonus while `tick < haste_until`.
    pub haste_until: Tick,
    /// Grievous Wounds: healing received is cut while `tick < heal_cut_until`.
    pub heal_cut_until: Tick,
    pub heal_cut_bp: i32,
    /// Second Wind charges already spent by this unit.
    pub second_wind_used: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerState {
    pub energy: u32,
    /// Regen left over from the Interest multiplier, in ten-thousandths of
    /// a sub-energy, so the boost is exact (BR-E06).
    pub regen_frac: u32,
    pub offers: [UnitKind; OFFERS],
    pub stronghold_hp: i32,
    pub stronghold_cooldown: u16,
    /// Synergy level (0..=2) per `Faction` index.
    pub synergy: [u8; 4],
    pub last_seq: u32,
    pub rng: Pcg32,
    /// Frozen for the whole match (BR-AR05); public once paired.
    pub arcanas: ArcanaPreset,
    /// Copies owned per Augment id (compact, fixed size; no purchase list).
    pub augments: AugmentCounts,
    /// Last round this seat resolved (purchase, Forgo or expiry).
    pub augment_resolved: u32,
    /// The confirmed id of the open round, hidden from the other seat and
    /// applied when the round resolves (BR-AG14). `None` after a Forgo.
    pub augment_pick: Option<u8>,
    /// Bounty earned in the current Interest window (sub-energy).
    pub bounty_window: u32,
    /// Stone Mend window start and HP healed inside it.
    pub mend_window_start: Tick,
    pub mend_in_window: u16,
    /// Last Bastion shield.
    pub shield: i32,
    pub shield_until: Tick,
    pub last_stand_used: bool,
    /// Tower Fury stacks and their shared expiry tick.
    pub fury_stacks: u8,
    pub fury_until: Tick,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EndReason {
    StrongholdDestroyed,
    Forfeit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Outcome {
    /// `None` is a draw.
    pub winner: Option<Seat>,
    pub reason: EndReason,
}

/// Everything that happened in a tick; clients animate from these.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    Spawn {
        unit: UnitId,
        seat: Seat,
        kind: UnitKind,
    },
    AttackStart {
        from: Target,
        to: Target,
    },
    Hit {
        to: Target,
        amount: i32,
    },
    Heal {
        to: UnitId,
        amount: i32,
    },
    StatusApplied {
        unit: UnitId,
    },
    Death {
        unit: UnitId,
    },
    Evolved {
        unit: UnitId,
        tier: u8,
        branch: u8,
    },
    SynergyChanged {
        seat: Seat,
        faction: Faction,
        level: u8,
    },
    PhaseChanged {
        phase: Phase,
    },
    RoundOpened {
        round: u32,
    },
    AugmentChosen {
        seat: Seat,
        id: u8,
    },
    Forgone {
        seat: Seat,
        round: u32,
    },
    ShieldRaised {
        seat: Seat,
    },
    SecondWind {
        unit: UnitId,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchState {
    /// The combat clock: frozen while an Augment selection pauses the
    /// battle, so every tick-based rule counts effective combat (BR-AG12).
    pub tick: Tick,
    /// Steps since creation, advancing through pauses too: the selection
    /// clock that closes a paused round, and the replay key (BR-AG13).
    pub clock: Tick,
    pub phase: Phase,
    pub players: [PlayerState; 2],
    /// Always sorted by id (new units get the next, larger id).
    pub units: Vec<Unit>,
    pub next_unit_id: UnitId,
    pub outcome: Option<Outcome>,
    /// Augment draw, cycle and open round (BR-AG01–08).
    pub augments: AugmentState,
    /// Per-seat living role counts for conditional Arcanas; derived each
    /// tick, so it is not serialized or part of the state hash.
    #[serde(skip, default)]
    pub(crate) army: [ArmyComposition; 2],
    /// Events created by intentions, returned by the next `step()`.
    #[serde(skip)]
    pub(crate) pending: Vec<Event>,
    /// Balance numbers for this match: always `CANON` online (D-33). Not
    /// serialized, so hashes and views do not depend on it.
    #[serde(skip, default = "Ruleset::canon")]
    pub(crate) rules: &'static Ruleset,
    /// Per-seat, per-kind combat counters for the balance lab (P42). Never
    /// serialized, so the state hash and views never see them.
    #[cfg(feature = "lab")]
    #[serde(skip, default)]
    pub lab: crate::lab::LabStats,
}

/// x position of a seat's Stronghold.
pub fn stronghold_x(seat: Seat) -> i32 {
    match seat {
        Seat::A => 0,
        Seat::B => content::LANE_LEN,
    }
}

impl Unit {
    /// Summons are recognized by unit, not by kind (BR-U08): a carried
    /// roster kind (Cargo Crab, P48) is still a summon.
    pub(crate) fn is_summon(&self) -> bool {
        self.summoner.is_some() || self.kind.is_summon()
    }
}

impl PlayerState {
    /// The one spend path: rejects overdrafts (BR-E03). The balance drops
    /// at once, and with it the Interest boost (BR-E06).
    pub fn spend(&mut self, cost: u32) -> bool {
        if self.energy < cost {
            return false;
        }
        self.energy -= cost;
        true
    }

    /// Credit Energy (Bounty, Forgo) and return what was added.
    pub fn credit(&mut self, amount: u32) -> u32 {
        let before = self.energy;
        self.energy = self.energy.saturating_add(amount);
        self.energy - before
    }

    pub fn copies(&self, id: u8) -> u16 {
        self.augments.get(usize::from(id))
    }
}

impl MatchState {
    pub fn rules(&self) -> &'static Ruleset {
        self.rules
    }

    pub fn player(&self, seat: Seat) -> &PlayerState {
        &self.players[seat.index()]
    }

    pub(crate) fn player_mut(&mut self, seat: Seat) -> &mut PlayerState {
        &mut self.players[seat.index()]
    }

    pub fn unit_index(&self, id: UnitId) -> Option<usize> {
        self.units.binary_search_by_key(&id, |u| u.id).ok()
    }

    pub fn is_live(&self) -> bool {
        self.phase == Phase::Running
    }

    /// A seat's living conventional units (BR-U07: informative, never a cap).
    pub fn army_size(&self, seat: Seat) -> usize {
        self.units
            .iter()
            .filter(|u| u.seat == seat && u.hp > 0 && !u.is_summon())
            .count()
    }

    pub fn stats_of(&self, u: &Unit) -> Stats {
        // Summons never receive Synergy or Arcanas (BR-U08, D-40); only
        // Augments naming Beetlings reach them (BR-AG10).
        if u.is_summon() {
            let p = self.player(u.seat);
            let bonus = crate::augment::unit_bonus(&p.augments, p.synergy, u.kind, u.tier);
            return self.rules.stats_with(
                u.kind,
                u.tier,
                u.branch,
                0,
                &ArcanaPreset::NEUTRAL,
                &bonus,
            );
        }
        self.stats_for(u.seat, u.kind, u.tier, u.branch)
    }

    /// The seat's cached living Tank and DPS counts (BR-AR06).
    pub fn army_of(&self, seat: Seat) -> ArmyComposition {
        self.army[seat.index()]
    }

    /// Rebuild the conditional-Arcana counts from the living conventional
    /// army. Called once per combat tick; `damage` decrements on death.
    pub(crate) fn refresh_army(&mut self) {
        let mut army = [ArmyComposition::default(); 2];
        for u in self
            .units
            .iter()
            .filter(|u| u.hp > 0 && !u.is_summon())
        {
            army[u.seat.index()].add(self.rules.def(u.kind).archetype);
        }
        self.army = army;
    }

    /// Stats of a seat's conventional unit: Synergy, Arcanas and Augments.
    pub fn stats_for(&self, seat: Seat, kind: UnitKind, tier: u8, branch: u8) -> Stats {
        let p = self.player(seat);
        let level = p.synergy[self.rules.def(kind).faction.index()];
        let bonus = crate::augment::unit_bonus(&p.augments, p.synergy, kind, tier);
        self.rules
            .stats_with(kind, tier, branch, level, &p.arcanas, &bonus)
    }

    /// Recompute max HP for every living conventional unit of a seat.
    pub(crate) fn refresh_units(&mut self, seat: Seat) {
        for idx in 0..self.units.len() {
            if self.units[idx].seat == seat && self.units[idx].hp > 0 {
                self.refresh_max_hp(idx);
            }
        }
    }

    /// Recompute a unit's max HP, keeping its HP ratio (GAME-DESIGN §4).
    pub(crate) fn refresh_max_hp(&mut self, idx: usize) {
        let new_max = self.stats_of(&self.units[idx]).max_hp;
        let u = &mut self.units[idx];
        if new_max != u.max_hp && u.hp > 0 {
            let hp = i64::from(u.hp) * i64::from(new_max) / i64::from(u.max_hp);
            u.hp = (hp as i32).max(1);
            u.max_hp = new_max;
        }
    }
}
