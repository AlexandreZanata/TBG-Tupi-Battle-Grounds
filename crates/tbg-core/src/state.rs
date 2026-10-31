//! Match state, units, outcomes, and the events a tick produces.

use serde::{Deserialize, Serialize};

use crate::arcana::ArcanaPreset;
use crate::augment::{AugmentCounts, AugmentState};
use crate::content::{self, Ability, Stats, BP, MIN_INTERVAL, OFFERS};
use crate::rng::Pcg32;
use crate::ruleset::Ruleset;
use crate::types::{
    Archetype, Arena, Faction, Seat, Status, StrongholdKind, Tick, UnitId, UnitKind, FACTION_COUNT,
};

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

/// One victim's share of an affliction stack list (Rotcap poison, Old Finn
/// fish, P45): fixed arrays, no allocation, deterministic replacement.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stack {
    pub left: u16,
    pub value: u16,
}

/// Per-unit ability state (ROSTER-EXPANSION §F.2): a compact `Copy` block.
/// All zeroes until the unit's wave lands; never part of views in P43.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitFx {
    pub poison: [Stack; 6],
    pub fish: [Stack; 3],
    pub confused_until: Tick,
    pub immune_until: Tick,
    pub ward_until: Tick,
    pub betray: bool,
    pub attacks: u8,
    pub essence: u16,
    pub essence_progress: u32,
    pub plates: u8,
    pub plate_hp: i16,
    pub march_move_bp: u16,
    pub march_dr_bp: u16,
    /// Drumguard B2: the bonded ally ignores slows and fish (not root).
    /// Rewritten every tick by the bonds pass, like the march fields.
    pub march_ignore_slow: bool,
    /// Cargo Crab B2 Overdrive: surge movement until this tick (re-armed
    /// 220 ticks after it lapses, i.e. a 15 s cooldown on a 4 s surge).
    pub surge_until: Tick,
    /// Puppetmaster betrayal bonus, in basis points (A1 +25 %, A2 +50 %),
    /// stored at confuse time and applied to the betrayal hit.
    pub betray_bp: u16,
    /// Puppetmaster A2 Stumble: the betrayer is slowed 30 % for 2 s.
    pub betray_stumble: bool,
    pub bond: Option<UnitId>,
    pub carrier: Option<UnitId>,
    pub passenger: Option<UnitId>,
    pub board_lock: Tick,
    pub shield: u16,
    pub shield_until: Tick,
    /// W0 (P58 §3.2): the bleed slot of R07, one per victim, not stacked.
    pub bleed: Bleed,
    /// W0 (R13): basic attacks this unit makes lose `weaken_bp` until `weaken_until`.
    pub weaken_bp: u16,
    pub weaken_until: Tick,
    /// W0 (R19): a Slow that starts when this unit's Root ends.
    pub queued_slow: Option<QueuedSlow>,
    /// W0 (R03): this caster's one mark, its target and deadline.
    pub mark_target: Option<UnitId>,
    pub mark_until: Tick,
    /// W0: a timed self-reduction; it joins the damage reduction sum and its cap.
    pub self_dr_bp: u16,
    pub self_dr_until: Tick,
    /// W0 (R03, R20): the ability heal-cut slot, beside the pair in `Unit`.
    pub ability_cut_bp: u16,
    pub ability_cut_until: Tick,
    /// Independent bounded special windup; serialized for deterministic replay.
    pub special: Option<PendingCast>,
    /// R18/P58 §3.2: one venom family, separate from bleed and Rotcap stacks.
    pub venom: Bleed,
}

/// One special locked at start; the ordinary basic windup stays independent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingCast {
    pub target: Option<UnitId>,
    pub left: u16,
}

/// The bleed slot (R07): `value` HP per pulse, `left` pulses still to come,
/// `next` the tick of the next one. An empty slot has `left == 0`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bleed {
    pub value: u16,
    pub next: Tick,
    pub left: u8,
    pub source: Option<Seat>,
}

/// A Slow queued behind a Root (R19).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueuedSlow {
    pub bp: u16,
    pub ticks: u16,
}

/// Queued A2 Shatterback shrapnel (P46): processed in `remove_dead` before
/// removal, never recursively. Runtime-only: skipped by serde, so hashes
/// never see it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Shrapnel {
    pub x: i32,
    pub seat: Seat,
    pub amount: i32,
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
    /// Ability state (all zero until the unit's wave lands).
    pub fx: UnitFx,
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
    /// Frozen at pairing (BR-SK01); public to both seats.
    pub stronghold: StrongholdKind,
    /// Fortification tier (0..=2) and branch (0 = A, 1 = B), BR-SK05.
    pub fort_tier: u8,
    pub fort_branch: u8,
    /// Treasury Spoils earned in the current Bounty window (BR-SK02).
    pub spoils_window: u32,
    /// Synergy level (0..=2) per `Faction` index.
    pub synergy: [u8; FACTION_COUNT],
    pub last_seq: u32,
    pub rng: Pcg32,
    /// Smuggler cargo stream (P48-B): a new hidden stream per seat, never
    /// in views (BR-S05).
    pub cargo_rng: Pcg32,
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
    /// ARENAS-G: the arena's first-Evolution discount was spent (only set
    /// when the arena has one, on a successful Evolution).
    pub arena_discount_used: bool,
    /// BR-S06–S13: private Shop state, counted in new shops rather than ticks.
    pub reserve: [u8; OFFERS],
    pub held_mask: u8,
    pub reroll_streak: u8,
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
    /// A seat bought a Fortification tier for its own tower (BR-SK05).
    Fortified {
        seat: Seat,
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
    // v13 (P43, ROSTER-EXPANSION §F.4): staged affliction and carrier
    // events. Unused until their waves land; appended, never reordered.
    PoisonTick {
        unit: UnitId,
        amount: i32,
    },
    Betrayal {
        unit: UnitId,
        ally: UnitId,
    },
    Cleansed {
        unit: UnitId,
    },
    PlateBroken {
        unit: UnitId,
    },
    CrateOpened {
        /// The crate's own id (`crate` is a keyword); `opened` is the cargo.
        unit: UnitId,
        opened: Option<UnitId>,
    },
    /// Siphon: from is the enemy struck, to the ally healed, amount actual HP restored.
    Transfer {
        from: UnitId,
        to: UnitId,
        amount: i32,
    },
    Boarded {
        crab: UnitId,
        unit: UnitId,
    },
    Dismounted {
        unit: UnitId,
    },
    /// R10: cosmetic connection from the struck primary to one secondary.
    CorrosiveSplash {
        from: UnitId,
        primary: UnitId,
        to: UnitId,
    },
    /// P58: one server-authoritative special, discriminated by caster kind.
    AbilityStarted {
        unit: UnitId,
        kind: UnitKind,
        primary: Option<UnitId>,
        center: i32,
        tick: Tick,
    },
    AbilityResolved {
        unit: UnitId,
        kind: UnitKind,
        primary: Option<UnitId>,
        center: i32,
        tick: Tick,
    },
    /// D-75: the server-selected fourth-shot secondary.
    Crossfire {
        from: UnitId,
        to: UnitId,
    },
    /// D-87: exact authoritative one-dimensional friendly-fire extent.
    ProfaneFlame {
        unit: UnitId,
        start: i32,
        end: i32,
        tick: Tick,
    },
    /// D-88: fusion consumes parents without Death or rewards.
    DrillFused {
        first: UnitId,
        second: UnitId,
        unit: UnitId,
        x: i32,
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
    /// ARENAS-G: fixed at creation; its modifiers apply to both seats.
    pub arena: Arena,
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
    /// Queued A2 shrapnel, drained in `remove_dead` before removal (P46).
    /// Runtime-only, like `pending`.
    #[serde(skip)]
    pub(crate) shrapnel: Vec<Shrapnel>,
    /// Local training runtime (P62): dummy tags and attributed meters.
    /// `None` in every normal match; never serialized, so hashes, views and
    /// wire bytes are unchanged even in a build with the feature.
    #[cfg(feature = "training")]
    #[serde(skip, default)]
    pub(crate) training: Option<Box<crate::training::Runtime>>,
    /// Runtime-only historical counts; a deserialized snapshot cannot claim full history.
    #[serde(skip, default)]
    pub(crate) participation: crate::celebration::LifetimeCast,
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
        #[cfg(feature = "bench-profile")]
        crate::profiling::ID_SEARCHES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
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

    /// This match's arena modifiers.
    pub fn arena_rule(&self) -> crate::content::ArenaRule {
        self.rules.arenas[self.arena.index()]
    }

    /// The arena's troop modifiers join a unit's Augment sums, so they share
    /// the single truncation (BR-X01). Every unit on the field gets them,
    /// summons and cargo included, on both seats.
    fn arena_bonus(&self, mut bonus: crate::content::AugmentBonus) -> crate::content::AugmentBonus {
        let a = self.arena_rule();
        bonus.move_bp += i64::from(a.move_bp);
        bonus.atk_bp += i64::from(a.atk_bp);
        bonus
    }

    pub fn stats_of(&self, u: &Unit) -> Stats {
        #[cfg(feature = "bench-profile")]
        crate::profiling::STATS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        // Summons never receive Synergy or Arcanas (BR-U08, D-40); only
        // Augments naming Beetlings reach them (BR-AG10). Cargo and crates
        // take no Augments at all (P48-B, §C.11).
        if u.is_summon() {
            let p = self.player(u.seat);
            let bonus = if u.kind == UnitKind::Beetling {
                crate::augment::unit_bonus(&p.augments, p.synergy, u.kind, u.tier)
            } else {
                crate::content::AugmentBonus::default()
            };
            return self.rules.stats_with(
                u.kind,
                u.tier,
                u.branch,
                0,
                &ArcanaPreset::NEUTRAL,
                &self.arena_bonus(bonus),
            );
        }
        let mut st = self.stats_for(u.seat, u.kind, u.tier, u.branch);
        // P46: broken Shatterback plates permanently speed up the unit.
        // Caps hold exactly (BR-X02, BR-X03): never above the uncapped max.
        let [a, b] = self.rules.def(u.kind).ability(u.tier, u.branch);
        for ability in [a, b] {
            if let Ability::Plates {
                count,
                move_bp,
                atk_bp,
                ..
            } = ability
            {
                let broken = i64::from(count.saturating_sub(u.fx.plates));
                if broken > 0 {
                    let base = self.rules.def(u.kind);
                    st.speed =
                        (i64::from(st.speed) * (10_000 + i64::from(move_bp) * broken) / BP) as i32;
                    st.speed = st
                        .speed
                        .min((i64::from(base.speed) * 15_000 / 10_000) as i32);
                    let atk = (i64::from(st.atk_bp) + i64::from(atk_bp) * broken).min(10_000);
                    st.atk_bp = atk as i32;
                    st.interval = ((i64::from(base.interval) * 10_000 / (10_000 + atk))
                        .max(i64::from(MIN_INTERVAL))) as u16;
                }
            }
            // P48-A: Leechroot Essence — +damage and +attack speed per
            // stack, the attack-speed part joining the BR-X02 cap.
            if let Ability::Essence { dmg, atk_bp, .. } = ability {
                let n = u.fx.essence;
                if n > 0 {
                    let base = self.rules.def(u.kind);
                    st.damage = st.damage.saturating_add(i32::from(dmg) * i32::from(n));
                    let atk = (i64::from(st.atk_bp) + i64::from(atk_bp) * i64::from(n)).min(10_000);
                    st.atk_bp = atk as i32;
                    st.interval = ((i64::from(base.interval) * 10_000 / (10_000 + atk))
                        .max(i64::from(MIN_INTERVAL))) as u16;
                }
            }
        }
        st
    }

    /// The seat's cached living Tank and DPS counts (BR-AR06).
    pub fn army_of(&self, seat: Seat) -> ArmyComposition {
        self.army[seat.index()]
    }

    /// Rebuild the conditional-Arcana counts from the living conventional
    /// army. Called once per combat tick; `damage` decrements on death.
    pub(crate) fn refresh_army(&mut self) {
        let mut army = [ArmyComposition::default(); 2];
        for u in self.units.iter().filter(|u| u.hp > 0 && !u.is_summon()) {
            army[u.seat.index()].add(self.rules.def(u.kind).archetype);
        }
        self.army = army;
    }

    /// Stats of a seat's conventional unit: Synergy, Arcanas and Augments.
    pub fn stats_for(&self, seat: Seat, kind: UnitKind, tier: u8, branch: u8) -> Stats {
        let p = self.player(seat);
        let level = p.synergy[self.rules.def(kind).faction.index()];
        let bonus = crate::augment::unit_bonus(&p.augments, p.synergy, kind, tier);
        self.rules.stats_with(
            kind,
            tier,
            branch,
            level,
            &p.arcanas,
            &self.arena_bonus(bonus),
        )
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
        // P62: a training dummy keeps its own max HP (a Crate carrier whose
        // definition Faction would otherwise rescale it to the Crate's).
        if crate::training_hooks::is_dummy(self, self.units[idx].id) {
            return;
        }
        let new_max = self.stats_of(&self.units[idx]).max_hp;
        let before = self.units[idx];
        let u = &mut self.units[idx];
        if new_max != u.max_hp && u.hp > 0 {
            let hp = i64::from(u.hp) * i64::from(new_max) / i64::from(u.max_hp);
            u.hp = (hp as i32).max(1);
            u.max_hp = new_max;
            let delta = u.hp - before.hp;
            crate::training_hooks::rescaled(self, &before, delta);
        }
    }
}
