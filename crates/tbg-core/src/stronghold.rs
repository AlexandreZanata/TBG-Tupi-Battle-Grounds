//! Stronghold kinds and Fortification (GAME-DESIGN §10.4, BR-SK01–SK09):
//! effective tower stats, the Fortify intention, Treasury Spoils and the
//! Command War Banner aura.

use crate::content::{KindDef, BP, STRONGHOLD_KINDS};
use crate::intent::Rejection;
use crate::ruleset::Ruleset;
use crate::state::{stronghold_x, Event, MatchState, Unit};
use crate::types::{Seat, StrongholdKind};

/// A seat's tower after its kind and Fortification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StrongholdStats {
    pub max_hp: i32,
    pub damage: i32,
    pub interval: u16,
    pub range: i32,
    /// Sub-energy per eligible kill and per window (0 = no Spoils).
    pub spoils: u32,
    pub spoils_cap: u32,
    /// War Banner damage bp and radius in milli-tiles (0 = no aura).
    pub aura_bp: i32,
    pub aura_range: i32,
}

pub fn kind_def(kind: StrongholdKind) -> &'static KindDef {
    &STRONGHOLD_KINDS[kind.index()]
}

fn scale(value: i32, bp: i32) -> i32 {
    (i64::from(value) * (BP + i64::from(bp)) / BP).clamp(0, i64::from(i32::MAX)) as i32
}

/// Stats of `kind` at `tier` (0..=2) on `branch` (0 = A, 1 = B). Fortification
/// percentages are of the kind's own base, so tiers add up (BR-SK06).
pub fn stats(rules: &Ruleset, kind: StrongholdKind, tier: u8, branch: u8) -> StrongholdStats {
    let def = kind_def(kind);
    let base_hp = scale(rules.stronghold_hp, def.hp_bp);
    let base_dmg = scale(rules.stronghold_damage, def.dmg_bp);
    let mut s = StrongholdStats {
        max_hp: base_hp,
        damage: base_dmg,
        interval: rules.stronghold_interval,
        range: rules.stronghold_range,
        spoils: def.spoils,
        spoils_cap: def.spoils_cap,
        aura_bp: def.aura_bp,
        aura_range: def.aura_range,
    };
    let (mut hp_bp, mut dmg_bp) = (0, 0);
    let owned = usize::from(tier.min(2));
    for step in def.branches[usize::from(branch.min(1))].iter().take(owned) {
        hp_bp += step.hp_bp;
        dmg_bp += step.dmg_bp;
        s.range += step.range_add;
        if step.interval > 0 {
            s.interval = step.interval;
        }
        s.spoils += step.spoils_add;
        s.spoils_cap += step.spoils_cap_add;
        s.aura_bp += step.aura_bp_add;
        s.aura_range += step.aura_range_add;
    }
    s.max_hp = scale(base_hp, hp_bp).max(1);
    s.damage = scale(base_dmg, dmg_bp);
    s
}

impl MatchState {
    /// The seat's current tower stats.
    pub fn stronghold(&self, seat: Seat) -> StrongholdStats {
        let p = self.player(seat);
        stats(self.rules(), p.stronghold, p.fort_tier, p.fort_branch)
    }
}

/// Buy the next Fortification tier on `branch` for the seat's own tower
/// (BR-SK05). Tier 1 picks the branch for good; tier 2 must repeat it.
pub(crate) fn fortify(state: &mut MatchState, seat: Seat, branch: u8) -> Result<(), Rejection> {
    let p = state.player(seat);
    if p.fort_tier >= 2 {
        return Err(Rejection::MaxTier);
    }
    let branch_ok = if p.fort_tier == 0 {
        branch <= 1
    } else {
        branch == p.fort_branch
    };
    if !branch_ok {
        return Err(Rejection::WrongBranch);
    }
    let cost = state.rules().fortify_cost[usize::from(p.fort_tier)];
    if !state.player_mut(seat).spend(cost) {
        return Err(Rejection::NotEnoughEnergy);
    }
    let old_max = state.stronghold(seat).max_hp;
    let p = state.player_mut(seat);
    p.fort_tier += 1;
    p.fort_branch = branch;
    let tier = p.fort_tier;
    let new_max = state.stronghold(seat).max_hp;
    // Keep the HP ratio, never a free heal (BR-SK07).
    let p = state.player_mut(seat);
    p.stronghold_hp = rescale_hp(p.stronghold_hp, old_max, new_max);
    state.pending.push(Event::Fortified { seat, tier, branch });
    Ok(())
}

/// `hp` out of `old_max`, expressed out of `new_max` (rounded down, alive
/// stays alive).
pub fn rescale_hp(hp: i32, old_max: i32, new_max: i32) -> i32 {
    if hp <= 0 || old_max <= 0 || old_max == new_max {
        return hp;
    }
    let scaled = i64::from(hp) * i64::from(new_max) / i64::from(old_max);
    scaled.clamp(1, i64::from(new_max)) as i32
}

/// War Banner: bonus bp for a conventional attacker standing within its
/// own Command tower's radius (BR-SK04). Read at hit time from the current
/// position, never stored on the unit, so it ends the moment the unit
/// leaves and cannot stack across ticks.
pub(crate) fn aura_bp(state: &MatchState, attacker: &Unit) -> i32 {
    if attacker.kind.is_summon() {
        return 0;
    }
    let s = state.stronghold(attacker.seat);
    let inside = (attacker.x - stronghold_x(attacker.seat)).abs() <= s.aura_range;
    if s.aura_bp > 0 && inside {
        s.aura_bp
    } else {
        0
    }
}

/// Treasury Spoils for `killer` after one eligible enemy death (BR-SK02):
/// paid from the authoritative death, capped per Bounty window.
pub(crate) fn spoils(state: &mut MatchState, killer: Seat) {
    let s = state.stronghold(killer);
    let p = state.player_mut(killer);
    let pay = s.spoils.min(s.spoils_cap.saturating_sub(p.spoils_window));
    if pay > 0 {
        p.spoils_window += pay;
        p.credit(pay);
    }
}
