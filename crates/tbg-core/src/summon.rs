//! Nester Hatch, Smuggler cargo crates and summon lifetime (BR-U08). These
//! limits belong to the ability, never to the army: bought units stay
//! uncapped (BR-U07, D-32).

use crate::augment;
use crate::content::{Ability, BEETLES_PER_PLAYER};
use crate::state::{Event, MatchState, Unit};
use crate::types::{Seat, Status, UnitId, UnitKind};

/// Summons whose lifetime ended drop to 0 HP; `remove_dead` reports them.
pub(crate) fn expire(state: &mut MatchState) {
    let tick = state.tick;
    for idx in 0..state.units.len() {
        let u = state.units[idx];
        if u.hp > 0 && u.expires_at.is_some_and(|end| tick >= end) {
            state.units[idx].hp = 0;
            crate::training_hooks::vanished(state, &u);
        }
    }
}

/// Hard safety bound for the summon entity count per seat. Augment copies
/// raise the gameplay limit up to this fixed array size (no per-tick alloc).
const MAX_SUMMONS: usize = 64;

/// The living summons of one seat, by summoner.
struct Brood {
    by: [UnitId; MAX_SUMMONS],
    len: usize,
}

impl Default for Brood {
    fn default() -> Brood {
        Brood {
            by: [0; MAX_SUMMONS],
            len: 0,
        }
    }
}

impl Brood {
    fn of(&self, nester: UnitId) -> usize {
        self.by[..self.len]
            .iter()
            .filter(|&&id| id == nester)
            .count()
    }

    fn push(&mut self, nester: UnitId, limit: usize) {
        if self.len < limit.min(MAX_SUMMONS) {
            self.by[self.len] = nester;
            self.len += 1;
        }
    }
}

fn limit(state: &MatchState, seat: crate::types::Seat) -> usize {
    let add = augment::summon_limit_add(&state.player(seat).augments);
    (BEETLES_PER_PLAYER + usize::from(add)).min(MAX_SUMMONS)
}

/// Each living Nester, in id order, hatches when its timer is done and both
/// limits allow; a blocked Nester stays ready and hatches once space frees.
/// Smugglers drop cargo crates on the same pass; crates open on it too.
pub(crate) fn hatch(state: &mut MatchState, ev: &mut Vec<Event>) {
    let mut brood = [Brood::default(), Brood::default()];
    // Only Beetlings count toward the per-seat cap (cargo is uncapped).
    for u in state
        .units
        .iter()
        .filter(|u| u.hp > 0 && u.kind == UnitKind::Beetling)
    {
        if let Some(nester) = u.summoner {
            brood[u.seat.index()].push(nester, MAX_SUMMONS);
        }
    }
    for idx in 0..state.units.len() {
        let n = state.units[idx];
        if n.hp <= 0 {
            continue;
        }
        if n.kind == UnitKind::Smuggler {
            cargo_tick(state, idx, &n, ev);
            continue;
        }
        if n.kind == UnitKind::Crate {
            // P62: a training dummy is carried by an inert Crate; it never
            // opens. Real Smuggler crates keep their timer.
            if !crate::training_hooks::is_dummy(state, n.id) {
                crate_tick(state, idx, &n, ev);
            }
            continue;
        }
        if !state
            .rules
            .def(n.kind)
            .ability(n.tier, n.branch)
            .contains(&Ability::Hatch)
        {
            continue;
        }
        // Count down and hatch on the same tick: exactly `every` ticks apart.
        let cd = n.action_cd.saturating_sub(1);
        state.units[idx].action_cd = cd;
        if cd > 0 {
            continue;
        }
        let (every, per_nester) = state.rules.hatch_rule(n.tier, n.branch);
        let cap = limit(state, n.seat);
        let seat_brood = &mut brood[n.seat.index()];
        if seat_brood.len >= cap || seat_brood.of(n.id) >= per_nester {
            continue;
        }
        seat_brood.push(n.id, cap);
        state.units[idx].action_cd = every;
        spawn_beetling(state, &n, ev);
    }
}

/// A Beetling takes its Nester's tier and branch (its Venom/Blight forms)
/// and position; ids keep growing, so `units` stays sorted by id.
fn spawn_beetling(state: &mut MatchState, nester: &Unit, ev: &mut Vec<Event>) {
    let kind = UnitKind::Beetling;
    let bonus = augment::unit_bonus(
        &state.player(nester.seat).augments,
        state.player(nester.seat).synergy,
        kind,
        nester.tier,
    );
    let max_hp = state
        .rules
        .stats_with(
            kind,
            nester.tier,
            nester.branch,
            0,
            &crate::arcana::ArcanaPreset::NEUTRAL,
            &bonus,
        )
        .max_hp;
    let id = state.next_unit_id;
    state.next_unit_id += 1;
    state.units.push(Unit {
        id,
        seat: nester.seat,
        kind,
        tier: nester.tier,
        branch: nester.branch,
        x: nester.x,
        hp: max_hp,
        max_hp,
        cooldown: 0,
        windup: None,
        status: Status::None,
        summoner: Some(nester.id),
        expires_at: Some(state.tick + state.rules.beetle_lifetime),
        action_cd: 0,
        fx: crate::state::UnitFx::default(),
        haste_until: 0,
        heal_cut_until: 0,
        heal_cut_bp: 0,
        second_wind_used: 0,
    });
    state
        .participation
        .born(nester.seat, kind, nester.tier, nester.branch, true);
    ev.push(Event::Spawn {
        unit: id,
        seat: nester.seat,
        kind,
    });
}

/// Smuggler cargo (P48-B, §C.11): every `every` ticks the Smuggler drops a
/// Crate (120 HP, no attack, no movement, targetable) behind its carrier. The
/// first crate comes after a full interval.
fn cargo_tick(state: &mut MatchState, idx: usize, smuggler: &Unit, ev: &mut Vec<Event>) {
    let cd = smuggler.action_cd.saturating_sub(1);
    state.units[idx].action_cd = cd;
    if cd > 0 {
        return;
    }
    let [a, b] = state
        .rules()
        .def(smuggler.kind)
        .ability(smuggler.tier, smuggler.branch);
    let cargo = [a, b].into_iter().find_map(|ability| match ability {
        Ability::Cargo {
            every,
            open,
            table: _,
            double_every,
        } => Some((every, open, double_every)),
        _ => None,
    });
    let Some((every, open, double_every)) = cargo else {
        return;
    };
    state.units[idx].action_cd = every.max(1);
    // B2 Double Shipment: every `double_every`-th crate holds two draws.
    // The draw count rides on the crate (`fx.attacks`), decided at drop.
    let mut draws = 1u8;
    if double_every > 0 {
        let n = state.units[idx].fx.attacks.saturating_add(1);
        if n.is_multiple_of(double_every) {
            state.units[idx].fx.attacks = 0;
            draws = 2;
        } else {
            state.units[idx].fx.attacks = n;
        }
    }
    let id = state.next_unit_id;
    state.next_unit_id += 1;
    ev.push(Event::Spawn {
        unit: id,
        seat: smuggler.seat,
        kind: UnitKind::Crate,
    });
    state.units.push(Unit {
        id,
        seat: smuggler.seat,
        kind: UnitKind::Crate,
        tier: 0,
        branch: 0,
        x: (smuggler.x - smuggler.seat.dir() * crate::content::CRATE_DROP_BACK)
            .clamp(0, crate::content::LANE_LEN),
        hp: state.rules().def(UnitKind::Crate).hp,
        max_hp: state.rules().def(UnitKind::Crate).hp,
        cooldown: 0,
        windup: None,
        status: Status::None,
        summoner: smuggler.summoner.or(Some(smuggler.id)),
        expires_at: None,
        action_cd: open,
        fx: crate::state::UnitFx {
            attacks: draws,
            ..Default::default()
        },
        haste_until: 0,
        heal_cut_until: 0,
        heal_cut_bp: 0,
        second_wind_used: 0,
    });
}

/// A cargo crate opening (P48-B, §C.11): after `open` ticks the crate draws
/// `draws` times from the cargo table and releases the cargo as permanent
/// summons (never Bounty/Synergy/Arcana/Augment/Evolution, never Leechroot
/// food — all gated by `is_summon`). A destroyed crate releases nothing.
fn crate_tick(state: &mut MatchState, idx: usize, krate: &Unit, ev: &mut Vec<Event>) {
    let cd = krate.action_cd.saturating_sub(1);
    state.units[idx].action_cd = cd;
    if cd > 0 {
        return;
    }
    let draws = state.units[idx].fx.attacks.max(1);
    let table = cargo_table_of(state, krate);
    let summoner = krate.summoner;
    let x = krate.x;
    let mut opened = None;
    for _ in 0..draws {
        if let Some(id) = cargo_draw(state, krate.seat, summoner, x, table, ev) {
            if opened.is_none() {
                opened = Some(id);
            }
        }
    }
    ev.push(Event::CrateOpened {
        unit: krate.id,
        opened,
    });
    // The open crate is spent.
    if let Some(i) = state.unit_index(krate.id) {
        let spent = state.units[i];
        state.units[i].hp = 0;
        crate::training_hooks::vanished(state, &spent);
    }
}

/// The cargo table of the crate's Smuggler (live read: a mid-crate
/// Evolution applies); the base table if the Smuggler is gone.
fn cargo_table_of(state: &MatchState, krate: &Unit) -> [u16; 5] {
    krate
        .summoner
        .and_then(|sid| state.unit_index(sid))
        .and_then(|si| {
            let s = &state.units[si];
            if s.hp <= 0 || s.kind != UnitKind::Smuggler {
                return None;
            }
            let [a, b] = state.rules().def(s.kind).ability(s.tier, s.branch);
            [a, b].into_iter().find_map(|ability| match ability {
                Ability::Cargo { table, .. } => Some(table),
                _ => None,
            })
        })
        .unwrap_or([20, 42, 28, 8, 2])
}

/// One cargo draw: Empty returns nothing; Common/Strong draw a base form;
/// Elite/Legendary draw a random tier-1/2 form with a random branch.
/// Eligible kinds: the staged roster except Smuggler and Nester (summons
/// never summon, BR-U08); cost-3 for Common, cost 4–6 for Strong.
/// The cargo is a summon of the smuggler's line (`summoner`).
fn cargo_draw(
    state: &mut MatchState,
    seat: Seat,
    summoner: Option<UnitId>,
    x: i32,
    table: [u16; 5],
    ev: &mut Vec<Event>,
) -> Option<UnitId> {
    let rules = state.rules();
    let mut common = Vec::new();
    let mut strong = Vec::new();
    let mut any = Vec::new();
    for k in crate::types::ALL_KINDS
        .iter()
        .copied()
        .filter(|k| k.is_conventional())
    {
        let cost = rules.def(k).cost;
        if cost == crate::content::ENERGY_SCALE * 3 {
            common.push(k);
        }
        if (crate::content::ENERGY_SCALE * 4..=crate::content::ENERGY_SCALE * 6).contains(&cost)
            && k != UnitKind::Smuggler
            && k != UnitKind::Nester
        {
            strong.push(k);
        }
        if k != UnitKind::Smuggler && k != UnitKind::Nester {
            any.push(k);
        }
    }
    let roll = state.players[seat.index()].cargo_rng.below(100);
    let mut acc = 0u32;
    let mut outcome = 0usize;
    for (i, &w) in table.iter().enumerate() {
        acc += u32::from(w);
        if roll < acc {
            outcome = i;
            break;
        }
        // A malformed table (sums below 100) falls through to Legendary.
        if i == table.len() - 1 {
            outcome = table.len() - 1;
        }
    }
    let (pool, tier, branch) = match outcome {
        0 => return None,
        1 => (common, 0, 0),
        2 => (strong, 0, 0),
        3 => {
            let b = state.players[seat.index()].cargo_rng.below(2) as u8;
            (any, 1, b)
        }
        _ => {
            let b = state.players[seat.index()].cargo_rng.below(2) as u8;
            (any, 2, b)
        }
    };
    if pool.is_empty() {
        return None;
    }
    let kind = pool[state.players[seat.index()]
        .cargo_rng
        .below(pool.len() as u32) as usize];
    Some(spawn_cargo(
        state, seat, summoner, x, kind, tier, branch, ev,
    ))
}

/// A cargo unit: a permanent summon of a roster kind (P48-B, §C.11).
/// No lifetime, no Energy, no Bounty/Synergy/Arcana/Augment/Evolution;
/// never feeds Leechroot (all gated on `is_summon`).
#[allow(clippy::too_many_arguments)] // one spawn helper; reads as a table row
fn spawn_cargo(
    state: &mut MatchState,
    seat: crate::types::Seat,
    summoner: Option<UnitId>,
    x: i32,
    kind: UnitKind,
    tier: u8,
    branch: u8,
    ev: &mut Vec<Event>,
) -> UnitId {
    let max_hp = state
        .rules()
        .stats_with(
            kind,
            tier,
            branch,
            0,
            &crate::arcana::ArcanaPreset::NEUTRAL,
            &crate::content::AugmentBonus::default(),
        )
        .max_hp;
    let id = state.next_unit_id;
    state.next_unit_id += 1;
    state.units.push(Unit {
        id,
        seat,
        kind,
        tier,
        branch,
        x,
        hp: max_hp,
        max_hp,
        cooldown: 0,
        windup: None,
        status: Status::None,
        summoner,
        expires_at: None,
        action_cd: 0,
        fx: Default::default(),
        haste_until: 0,
        heal_cut_until: 0,
        heal_cut_bp: 0,
        second_wind_used: 0,
    });
    state.participation.born(seat, kind, tier, branch, true);
    ev.push(Event::Spawn {
        unit: id,
        seat,
        kind,
    });
    id
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::ENERGY_SCALE;

    /// §C.11: the cargo table holds over 100 000 crates — every outcome
    /// share within 1 percentage point and a χ² (4 dof) under the 1 %
    /// critical value; cargo is never a Smuggler, Nester or summon.
    #[test]
    fn cargo_table_holds_over_100k_crates() {
        const DRAWS: u32 = 100_000;
        let table = [20u16, 42, 28, 8, 2];
        let mut state = MatchState::new(48);
        let mut seen = [0u32; 5];
        let mut ev = Vec::new();
        for _ in 0..DRAWS {
            let Some(id) = cargo_draw(&mut state, Seat::A, None, 0, table, &mut ev) else {
                seen[0] += 1;
                continue;
            };
            let u = state.units.pop().expect("the cargo was just pushed");
            assert_eq!(u.id, id);
            assert!(u.kind.is_conventional());
            assert!(!matches!(u.kind, UnitKind::Smuggler | UnitKind::Nester));
            let cost = state.rules().def(u.kind).cost;
            let outcome = match u.tier {
                0 if cost == ENERGY_SCALE * 3 => 1,
                0 => 2,
                t => 2 + usize::from(t),
            };
            seen[outcome] += 1;
        }
        // Integer arithmetic only (C1 forbids floats in simulation sources):
        // expected = DRAWS × p / 100 = 1000 × p, so χ² × 1000 = Σ (o − e)² / p.
        let mut chi2_milli = 0u64;
        for (i, &p) in table.iter().enumerate() {
            let expected = i64::from(DRAWS) * i64::from(p) / 100;
            let diff = i64::from(seen[i]) - expected;
            // 1 percentage point of 100 000 draws is 1 000 draws.
            assert!(diff.abs() < 1_000, "outcome {i}: {} vs {expected}", seen[i]);
            chi2_milli += (diff * diff) as u64 / u64::from(p);
        }
        assert!(
            chi2_milli < 13_280,
            "χ² {chi2_milli}/1000 over 4 dof (1 % critical 13.28)"
        );
    }
}
