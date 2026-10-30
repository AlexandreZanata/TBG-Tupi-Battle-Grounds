//! Augments: infinite mirrored draws, compact per-id counts, and every
//! closed effect hook (GAME-DESIGN §10.6–10.8, owner revision 2026-10-29).
//!
//! The draw uses one match stream (3) that clients never see. Both seats
//! receive the same four ids; each may buy at most one per round. Counts are
//! `[u16; 40]`, so every effect is O(40) regardless of purchase history.
//!
//! An open round pauses the whole battle (BR-AG12): only the selection
//! clock (`MatchState::clock`) runs until both seats confirmed or the
//! window ends; confirmed picks stay hidden until that resolution.

use serde::{Deserialize, Serialize};

use std::ops::{Index, IndexMut};

use crate::arcana;
use crate::content::{
    augment_def, AugmentBonus, AugmentEffect, AugmentFilter, DamageAnchor, AUGMENTS,
    AUGMENT_CATALOG, AUGMENT_OPTIONS, BP,
};
use crate::intent::Rejection;
use crate::rng::Pcg32;
use crate::state::{Event, MatchState, Unit};
use crate::types::{Seat, Tick, UnitKind};

/// A full Bounty window is 1 Energy, shared by every source (BR-E08).
pub const BOUNTY_CEILING: u32 = 600;

/// Fixed-size copies per Augment id. Nested arrays keep serde support
/// without a dependency while staying exactly 80 serialized bytes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AugmentCounts([[u16; 8]; 5]);

impl AugmentCounts {
    pub fn get(self, id: usize) -> u16 {
        self.0[id / 8][id % 8]
    }

    pub fn add(&mut self, id: usize) {
        let slot = &mut self.0[id / 8][id % 8];
        *slot = slot.saturating_add(1);
    }

    pub fn iter(&self) -> impl Iterator<Item = &u16> {
        self.0.iter().flatten()
    }
}

impl Index<usize> for AugmentCounts {
    type Output = u16;

    fn index(&self, id: usize) -> &u16 {
        &self.0[id / 8][id % 8]
    }
}

impl IndexMut<usize> for AugmentCounts {
    fn index_mut(&mut self, id: usize) -> &mut u16 {
        &mut self.0[id / 8][id % 8]
    }
}

/// One shuffled catalog in fixed-size storage (serde has no `[T; 40]`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Deck([[u8; 8]; 5]);

impl Deck {
    fn ordered() -> Deck {
        let mut deck = Deck([[0; 8]; 5]);
        for i in 0..AUGMENT_CATALOG {
            deck.0[i / 8][i % 8] = i as u8;
        }
        deck
    }

    fn get(&self, i: usize) -> u8 {
        self.0[i / 8][i % 8]
    }

    fn set(&mut self, i: usize, value: u8) {
        self.0[i / 8][i % 8] = value;
    }

    fn four(&self, start: usize) -> [u8; AUGMENT_OPTIONS] {
        std::array::from_fn(|i| self.get(start + i))
    }

    fn shuffle(&mut self, rng: &mut Pcg32) {
        for i in (1..AUGMENT_CATALOG).rev() {
            let j = rng.below((i + 1) as u32) as usize;
            let (a, b) = (self.get(i), self.get(j));
            self.set(i, b);
            self.set(j, a);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AugmentState {
    pub rng: Pcg32,
    /// One shuffled catalog; four consecutive ids per round.
    pub deck: Deck,
    pub position: u8,
    /// Current round (0 before the first draw).
    pub round: u32,
    pub options: [u8; AUGMENT_OPTIONS],
    /// Combat tick the current or last round opened at.
    pub opened_at: Tick,
    /// Selection-clock value (`MatchState::clock`) that ends the open
    /// round; `u32::MAX` while no round is open.
    pub deadline: Tick,
    /// Combat tick of the next draw: pauses never count (BR-AG01).
    pub next_open_at: Tick,
}

impl AugmentState {
    pub fn new(seed: u64, first_round: Tick) -> AugmentState {
        let mut rng = Pcg32::new(seed, 3);
        let mut deck = Deck::ordered();
        deck.shuffle(&mut rng);
        AugmentState {
            rng,
            deck,
            position: 0,
            round: 0,
            options: [0; AUGMENT_OPTIONS],
            opened_at: 0,
            deadline: u32::MAX,
            next_open_at: first_round,
        }
    }

    /// A round is open, so the battle is paused (BR-AG12).
    pub fn selecting(&self) -> bool {
        self.round > 0 && self.deadline != u32::MAX
    }
}

/// Open a round when the combat clock reaches the next draw. Called after
/// Interest; the rest of this tick still runs, the pause starts next step.
pub(crate) fn tick(state: &mut MatchState, ev: &mut Vec<Event>) {
    let rule = state.rules.augment;
    if !state.is_live() || rule.round_ticks == 0 || state.augments.selecting() {
        return;
    }
    if state.tick >= state.augments.next_open_at {
        open_round(state, ev);
    }
}

/// One paused step: resolve as soon as both seats confirmed, or when the
/// selection clock passes the deadline, whoever is still deciding
/// (BR-AG13). Nothing else moves.
pub(crate) fn selection_step(state: &mut MatchState, ev: &mut Vec<Event>) {
    let round = state.augments.round;
    let both = state.players.iter().all(|p| p.augment_resolved >= round);
    if both || state.clock >= state.augments.deadline {
        close_round(state, ev);
    }
}

/// Apply both seats' outcomes in seat order on the same step: picks become
/// public copies, undecided seats expire into a Forgo (BR-AG04, BR-AG14).
fn close_round(state: &mut MatchState, ev: &mut Vec<Event>) {
    let round = state.augments.round;
    let reward = state.rules.augment.forgo_energy;
    for seat in Seat::BOTH {
        let p = state.player_mut(seat);
        p.augment_resolved = round;
        match p.augment_pick.take() {
            Some(id) => {
                p.augments.add(usize::from(id));
                state.refresh_units(seat);
                ev.push(Event::AugmentChosen { seat, id });
            }
            None => {
                if reward > 0 {
                    p.credit(reward);
                }
                ev.push(Event::Forgone { seat, round });
            }
        }
    }
    state.augments.deadline = u32::MAX;
    state.augments.options = [0; AUGMENT_OPTIONS];
}

fn open_round(state: &mut MatchState, ev: &mut Vec<Event>) {
    let rule = state.rules.augment;
    if state.augments.position as usize + AUGMENT_OPTIONS > AUGMENT_CATALOG {
        state.augments.deck.shuffle(&mut state.augments.rng);
        state.augments.position = 0;
    }
    let start = usize::from(state.augments.position);
    let options = state.augments.deck.four(start);
    state.augments.position += AUGMENT_OPTIONS as u8;
    state.augments.round += 1;
    state.augments.options = options;
    state.augments.opened_at = state.tick;
    state.augments.deadline = state.clock.saturating_add(rule.window.max(1));
    // Combat is frozen while selecting, so the next draw is a full
    // `round_ticks` of effective battle away (BR-AG01).
    state.augments.next_open_at = state.tick.saturating_add(rule.round_ticks);
    ev.push(Event::RoundOpened {
        round: state.augments.round,
    });
}

pub fn buy(state: &mut MatchState, seat: Seat, round: u32, option: u8) -> Result<(), Rejection> {
    if !state.is_live() {
        return Err(Rejection::NotRunning);
    }
    if state.rules.augment.round_ticks == 0 {
        return Err(Rejection::Unavailable);
    }
    if !state.augments.selecting() || round != state.augments.round {
        return Err(Rejection::RoundClosed);
    }
    let slot = usize::from(option);
    if slot >= AUGMENT_OPTIONS {
        return Err(Rejection::BadOption);
    }
    let p = state.player(seat);
    if p.augment_resolved >= round {
        return Err(Rejection::RoundClosed);
    }
    let id = state.augments.options[slot];
    let Some(def) = augment_def(id) else {
        return Err(Rejection::BadOption);
    };
    let cost = state.rules.augment.costs[def.rarity.index()];
    // Energy is frozen during the pause, so paying on confirmation is the
    // same as paying on resolution, and a confirmed pick can never bounce.
    if !state.player_mut(seat).spend(cost) {
        return Err(Rejection::NotEnoughEnergy);
    }
    let p = state.player_mut(seat);
    p.augment_pick = Some(id);
    p.augment_resolved = round;
    Ok(())
}

/// Decline the open round. Like a pick, it stays hidden and pays its
/// reward when the round resolves.
pub fn forgo(state: &mut MatchState, seat: Seat, round: u32) -> Result<(), Rejection> {
    if !state.is_live() {
        return Err(Rejection::NotRunning);
    }
    if state.rules.augment.round_ticks == 0 {
        return Err(Rejection::Unavailable);
    }
    if !state.augments.selecting() || round != state.augments.round {
        return Err(Rejection::RoundClosed);
    }
    if state.player(seat).augment_resolved >= round {
        return Err(Rejection::RoundClosed);
    }
    let p = state.player_mut(seat);
    p.augment_resolved = round;
    p.augment_pick = None;
    Ok(())
}

fn filter_matches(filter: AugmentFilter, kind: UnitKind, tier: u8) -> bool {
    let def = crate::content::def(kind);
    // Only Beetlings match Beetling augments; other summons (cargo,
    // crates) take no Augments at all (BR-U08).
    if kind.is_summon() {
        return kind == UnitKind::Beetling && filter == AugmentFilter::Beetlings;
    }
    match filter {
        AugmentFilter::Conventional => true,
        AugmentFilter::Archetype(a) => def.archetype == a,
        AugmentFilter::Faction(f) => def.faction == f,
        AugmentFilter::Evolved => tier >= 1,
        AugmentFilter::Beetlings => false,
    }
}

fn add(sum: &mut i64, bp: i32, copies: u16) {
    *sum = sum.saturating_add(i64::from(bp).saturating_mul(i64::from(copies)));
}

/// Static stat sums for one unit (BR-X01). Convergence is conditional and
/// read from the same tick's Synergy levels (BR-X11).
pub fn unit_bonus(
    counts: &AugmentCounts,
    synergy: [u8; 4],
    kind: UnitKind,
    tier: u8,
) -> AugmentBonus {
    let mut b = AugmentBonus::default();
    for def in &AUGMENTS {
        let copies = counts[usize::from(def.id)];
        if copies == 0 {
            continue;
        }
        match def.effect {
            AugmentEffect::UnitStat {
                filter,
                hp_bp,
                dmg_bp,
                atk_bp,
                move_bp,
                heal_bp,
                ..
            } if filter_matches(filter, kind, tier) => {
                add(&mut b.hp_bp, hp_bp, copies);
                add(&mut b.dmg_bp, dmg_bp, copies);
                add(&mut b.atk_bp, atk_bp, copies);
                add(&mut b.move_bp, move_bp, copies);
                add(&mut b.heal_bp, heal_bp, copies);
            }
            AugmentEffect::Convergence { hp_bp, dmg_bp }
                if !kind.is_summon() && synergy.iter().filter(|&&level| level > 0).count() >= 2 =>
            {
                add(&mut b.hp_bp, hp_bp, copies);
                add(&mut b.dmg_bp, dmg_bp, copies);
            }
            _ => {}
        }
    }
    b
}

pub fn summon_limit_add(counts: &AugmentCounts) -> u16 {
    counts
        .iter()
        .zip(AUGMENTS.iter())
        .filter_map(|(&copies, def)| match def.effect {
            AugmentEffect::UnitStat { summon_add, .. } if summon_add > 0 => {
                Some(u16::from(summon_add).saturating_mul(copies))
            }
            _ => None,
        })
        .fold(0u16, u16::saturating_add)
}

pub fn evolve_discount(counts: &AugmentCounts) -> u32 {
    AUGMENTS
        .iter()
        .filter_map(|def| match def.effect {
            AugmentEffect::EvolveDiscount { amount } => Some(
                u32::try_from(u64::from(amount) * u64::from(counts[usize::from(def.id)]))
                    .unwrap_or(u32::MAX),
            ),
            _ => None,
        })
        .fold(0u32, u32::saturating_add)
}

pub fn lifesteal_bp(counts: &AugmentCounts) -> i32 {
    AUGMENTS
        .iter()
        .filter_map(|def| match def.effect {
            AugmentEffect::Lifesteal { bp } => {
                Some(i64::from(bp) * i64::from(counts[usize::from(def.id)]))
            }
            _ => None,
        })
        .sum::<i64>()
        .min(1_200) as i32
}

pub fn status_duration_bp(counts: &AugmentCounts) -> i32 {
    AUGMENTS
        .iter()
        .filter_map(|def| match def.effect {
            AugmentEffect::StatusDuration { bp } => {
                Some(i64::from(bp) * i64::from(counts[usize::from(def.id)]))
            }
            _ => None,
        })
        .sum::<i64>()
        .min(3_000) as i32
}

pub fn haste_move_bp(counts: &AugmentCounts, unit: &Unit, tick: Tick) -> i32 {
    if unit.is_summon() || tick >= unit.haste_until {
        return 0;
    }
    AUGMENTS
        .iter()
        .filter_map(|def| match def.effect {
            AugmentEffect::DeployHaste { move_bp, .. } => {
                Some(i64::from(move_bp) * i64::from(counts[usize::from(def.id)]))
            }
            _ => None,
        })
        .sum::<i64>()
        .min(5_000) as i32
}

pub fn deploy_haste_until(counts: &AugmentCounts, tick: Tick) -> Tick {
    AUGMENTS
        .iter()
        .filter_map(|def| match def.effect {
            AugmentEffect::DeployHaste { ticks, .. } if counts[usize::from(def.id)] > 0 => {
                Some(tick.saturating_add(u32::from(ticks)))
            }
            _ => None,
        })
        .max()
        .unwrap_or(0)
}

pub fn stronghold_damage_bp(
    counts: &AugmentCounts,
    tick: Tick,
    fury_until: Tick,
    fury_stacks: u8,
) -> i32 {
    let mut bp = 0i64;
    for def in &AUGMENTS {
        if let AugmentEffect::StrongholdStat { dmg_bp } = def.effect {
            add(&mut bp, dmg_bp, counts[usize::from(def.id)]);
        }
        if let AugmentEffect::TowerFury { bp: per_stack, .. } = def.effect {
            let copies = counts[usize::from(def.id)];
            if copies > 0 && tick < fury_until {
                bp = bp.saturating_add(i64::from(per_stack) * i64::from(fury_stacks));
            }
        }
    }
    bp.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

pub fn tower_fury_rule(counts: &AugmentCounts) -> Option<(u8, u16)> {
    AUGMENTS.iter().find_map(|def| match def.effect {
        AugmentEffect::TowerFury {
            max_stacks, ticks, ..
        } if counts[usize::from(def.id)] > 0 => {
            let stacks = u16::from(max_stacks)
                .saturating_mul(counts[usize::from(def.id)])
                .min(u16::from(u8::MAX)) as u8;
            Some((stacks, ticks))
        }
        _ => None,
    })
}

pub fn situational_bp(state: &MatchState, attacker: &Unit, target: crate::state::Target) -> i32 {
    let p = state.player(attacker.seat);
    let mut bp = 0i64;
    for def in &AUGMENTS {
        let copies = p.augments[usize::from(def.id)];
        if copies == 0 {
            continue;
        }
        match def.effect {
            AugmentEffect::VsStronghold { filter, bp: add_bp }
                if matches!(target, crate::state::Target::Stronghold(_))
                    && filter_matches(filter, attacker.kind, attacker.tier) =>
            {
                add(&mut bp, add_bp, copies);
            }
            AugmentEffect::VsArchetype {
                archetype,
                bp: add_bp,
            } => {
                if let crate::state::Target::Unit(id) = target {
                    if let Some(i) = state.unit_index(id) {
                        if crate::content::def(state.units[i].kind).archetype == archetype {
                            add(&mut bp, add_bp, copies);
                        }
                    }
                }
            }
            AugmentEffect::Executioner {
                below_bp,
                bp: add_bp,
            } => {
                if let crate::state::Target::Unit(id) = target {
                    if let Some(i) = state.unit_index(id) {
                        let u = &state.units[i];
                        if i64::from(u.hp) * BP < i64::from(below_bp) * i64::from(u.max_hp) {
                            add(&mut bp, add_bp, copies);
                        }
                    }
                }
            }
            AugmentEffect::Comeback { bp: add_bp }
                if matches!(target, crate::state::Target::Stronghold(_))
                    && behind(state, attacker.seat) =>
            {
                add(&mut bp, add_bp, copies);
            }
            _ => {}
        }
    }
    bp.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

fn behind(state: &MatchState, seat: Seat) -> bool {
    state.player(seat).stronghold_hp < state.player(seat.other()).stronghold_hp
}

pub fn damage_reduction_bp(state: &MatchState, target: &Unit) -> i32 {
    let p = state.player(target.seat);
    let own_x = crate::state::stronghold_x(target.seat);
    let mut bp = 0i64;
    for def in &AUGMENTS {
        let copies = p.augments[usize::from(def.id)];
        if copies == 0 {
            continue;
        }
        if let AugmentEffect::DamageTakenAura {
            anchor,
            radius,
            bp: add_bp,
            below_bp,
        } = def.effect
        {
            let condition = match anchor {
                DamageAnchor::OwnRampart => state.units.iter().any(|u| {
                    u.seat == target.seat
                        && u.hp > 0
                        && u.kind == UnitKind::Rampart
                        && (u.x - target.x).abs() <= radius
                }),
                DamageAnchor::OwnStronghold => {
                    (below_bp == 0
                        || i64::from(p.stronghold_hp) * BP
                            < i64::from(below_bp) * i64::from(state.rules.stronghold_hp))
                        && (target.x - own_x).abs() <= radius
                }
            };
            if condition {
                add(&mut bp, add_bp, copies);
            }
        }
    }
    bp.min(2_000) as i32
}

pub fn heal_bonus_bp(state: &MatchState, healer: Seat, target: &Unit) -> i32 {
    let p = state.player(healer);
    let mut bp = 0i64;
    for def in &AUGMENTS {
        let copies = p.augments[usize::from(def.id)];
        if copies == 0 {
            continue;
        }
        if let AugmentEffect::HealBonus {
            bp: add_bp,
            below_bp,
        } = def.effect
        {
            if below_bp == 0
                || i64::from(target.hp) * BP < i64::from(below_bp) * i64::from(target.max_hp)
            {
                add(&mut bp, add_bp, copies);
            }
        }
    }
    bp.clamp(0, 10_000) as i32
}

pub fn heal_cut_bp(state: &MatchState, attacker: Seat) -> i32 {
    let p = state.player(attacker);
    AUGMENTS
        .iter()
        .filter_map(|def| match def.effect {
            AugmentEffect::HealCut { bp, .. } => {
                Some(i64::from(bp) * i64::from(p.augments[usize::from(def.id)]))
            }
            _ => None,
        })
        .sum::<i64>()
        .min(10_000) as i32
}

pub fn heal_cut_ticks(state: &MatchState, attacker: Seat) -> u16 {
    let p = state.player(attacker);
    AUGMENTS
        .iter()
        .filter_map(|def| match def.effect {
            AugmentEffect::HealCut { ticks, .. } if p.augments[usize::from(def.id)] > 0 => {
                Some(ticks)
            }
            _ => None,
        })
        .max()
        .unwrap_or(0)
}

pub fn second_wind_charges(counts: &AugmentCounts) -> u8 {
    counts
        .iter()
        .zip(AUGMENTS.iter())
        .filter_map(|(&copies, def)| {
            matches!(def.effect, AugmentEffect::SecondWind).then_some(copies)
        })
        .fold(0u16, u16::saturating_add)
        .min(u16::from(u8::MAX)) as u8
}

pub fn last_stand(counts: &AugmentCounts) -> Option<(i32, i32, u16)> {
    AUGMENTS.iter().find_map(|def| match def.effect {
        AugmentEffect::LastStand {
            below_bp,
            shield_bp,
            ticks,
        } if counts[usize::from(def.id)] > 0 => Some((
            below_bp,
            shield_bp.saturating_mul(i32::from(counts[usize::from(def.id)])),
            ticks,
        )),
        _ => None,
    })
}

/// After a Stronghold loses HP: raise the one-time shield when it first
/// crosses the threshold. Called once per damage event, never by time.
pub fn after_stronghold_damage(state: &mut MatchState, seat: Seat, ev: &mut Vec<Event>) {
    let p = state.player(seat);
    if p.last_stand_used {
        return;
    }
    let Some((below_bp, shield_bp, ticks)) = last_stand(&p.augments) else {
        return;
    };
    let max = state.rules.stronghold_hp.max(1);
    if p.stronghold_hp <= 0
        || i64::from(p.stronghold_hp) * BP >= i64::from(below_bp) * i64::from(max)
    {
        return;
    }
    let shield = (i64::from(max) * i64::from(shield_bp) / BP) as i32;
    let until = state.tick.saturating_add(u32::from(ticks));
    let p = state.player_mut(seat);
    p.shield = shield.max(1);
    p.shield_until = until;
    p.last_stand_used = true;
    ev.push(Event::ShieldRaised { seat });
}

/// Eligible kill rewards: the other seat's Bounty and Stone Mend. Summons
/// never count (BR-U08); there is no time damage.
pub(crate) fn on_eligible_kill(state: &mut MatchState, dead: Seat) {
    let killer = dead.other();
    let tick = state.tick;
    let p = state.player(killer);
    let counts = p.augments;
    let mut bounty = 0u32;
    let mut mend = 0i32;
    let mut mend_window_start = p.mend_window_start;
    let mut mend_in_window = p.mend_in_window;
    for def in &AUGMENTS {
        let copies = u32::from(counts[usize::from(def.id)]);
        if copies == 0 {
            continue;
        }
        match def.effect {
            AugmentEffect::Bounty { per_kill, cap } => {
                let room = cap
                    .saturating_mul(copies)
                    .saturating_sub(p.bounty_window)
                    .min(BOUNTY_CEILING.saturating_sub(p.bounty_window));
                bounty = bounty.saturating_add(per_kill.saturating_mul(copies).min(room));
            }
            AugmentEffect::KillMend { hp, cap, window } => {
                if mend_window_start == 0
                    || tick >= mend_window_start.saturating_add(u32::from(window))
                {
                    mend_window_start = tick;
                    mend_in_window = 0;
                }
                let room = u32::try_from(cap)
                    .unwrap_or(u32::MAX)
                    .saturating_mul(copies)
                    .saturating_sub(u32::from(mend_in_window));
                let amount = (hp.max(0) as u32).saturating_mul(copies).min(room) as i32;
                if amount > 0 {
                    mend = mend.saturating_add(amount);
                    mend_in_window = mend_in_window.saturating_add(amount as u16);
                }
            }
            _ => {}
        }
    }
    let max = state.rules.stronghold_hp;
    let p = state.player_mut(killer);
    p.bounty_window = p.bounty_window.saturating_add(bounty);
    p.credit(bounty);
    p.mend_window_start = mend_window_start;
    p.mend_in_window = mend_in_window;
    if mend > 0 && p.stronghold_hp > 0 {
        p.stronghold_hp = p.stronghold_hp.saturating_add(mend).min(max);
    }
}

pub fn reset_windows(state: &mut MatchState) {
    for p in &mut state.players {
        p.bounty_window = 0;
    }
}

/// Effective heal after Triage and Grievous Wounds, capped at missing HP.
pub fn heal_amount(
    state: &MatchState,
    healer: Seat,
    target: &Unit,
    base: i32,
    direct: bool,
) -> i32 {
    if base <= 0 || target.hp <= 0 || target.hp >= target.max_hp {
        return 0;
    }
    let mut bonus = if direct {
        heal_bonus_bp(state, healer, target)
    } else {
        0
    };
    if direct && !target.is_summon() {
        bonus += arcana::emergency_heal_bp(
            &state.player(healer).arcanas,
            &state.rules().arcana,
            target.hp,
            target.max_hp,
        );
    }
    let bonus = bonus.clamp(0, 10_000);
    let cut = if state.tick < target.heal_cut_until {
        target.heal_cut_bp
    } else {
        0
    };
    let scaled = i64::from(base) * i64::from(10_000 + bonus) / BP;
    let after_cut = scaled * i64::from(10_000 - cut).max(0) / 10_000;
    (target.max_hp - target.hp).min(after_cut.max(0).min(i64::from(i32::MAX)) as i32)
}
