//! Ally-support actions (P46–P47): Bloodweaver transfers, Drumguard bonds.
//! Cargo Crab boarding joins in P48 (ROSTER-EXPANSION §F.3).

use std::cmp::Reverse;

use crate::afflictions::abilities_of;
use crate::content::{Ability, Stats, LANE_LEN};
use crate::state::{Event, MatchState, Target, Unit};
use crate::types::{Archetype, OnHit, UnitId};

/// The attacker's Transfer params, if any (absolute per form slot, like Brew).
pub(crate) fn transfer_of(state: &MatchState, attacker: &Unit) -> Option<(u16, u16, bool)> {
    abilities_of(state, attacker)
        .into_iter()
        .find_map(|ability| match ability {
            Ability::Transfer {
                amount,
                low_bp,
                second,
            } => Some((amount, low_bp, second)),
            _ => None,
        })
}

/// The transfer pair (§C.14): among allied **conventional** units in range
/// (itself included; summons excluded as donors and receivers), the donor
/// is the highest HP % and the receiver the lowest HP % below full; they
/// must be different units and the donor's % higher. Ties: nearest, then
/// lowest id. `exclude` skips two units (the Double Flask second transfer).
pub(crate) fn transfer_pair(
    state: &MatchState,
    u: &Unit,
    range: i32,
    exclude: [Option<UnitId>; 2],
) -> Option<(UnitId, UnitId, i32)> {
    let mut donor: Option<(i32, i32, UnitId, i32)> = None;
    let mut receiver: Option<(i32, i32, UnitId, i32)> = None;
    for a in state.units.iter().filter(|a| {
        a.seat == u.seat
            && a.hp > 0
            && !a.is_summon()
            && Some(a.id) != exclude[0]
            && Some(a.id) != exclude[1]
            && (a.x - u.x).abs() <= range
    }) {
        // `pct` in basis points of max HP; the receiver also tracks HP
        // missing below full.
        let pct = (i64::from(a.hp) * 10_000 / i64::from(a.max_hp.max(1))) as i32;
        let dist = (a.x - u.x).abs();
        // Donor: highest %, then nearest, then lowest id.
        let better_donor = match donor {
            None => true,
            Some((p, d, id, _)) => (pct, Reverse(dist), Reverse(id)) > (p, Reverse(d), Reverse(id)),
        };
        if better_donor {
            donor = Some((pct, dist, a.id, a.hp - 1));
        }
        // Receiver: lowest % below full, then nearest, then lowest id.
        if a.hp < a.max_hp {
            let better_receiver = match receiver {
                None => true,
                Some((p, d, id, _)) => (pct, dist, id) < (p, d, id),
            };
            if better_receiver {
                receiver = Some((pct, dist, a.id, a.max_hp - a.hp));
            }
        }
    }
    let (dpct, _, donor_id, donor_headroom) = donor?;
    let (rpct, _, receiver_id, missing) = receiver?;
    if donor_id == receiver_id || dpct <= rpct {
        return None;
    }
    // A pair that would move nothing is no pair: the weaver waits ready.
    let headroom = donor_headroom.min(missing);
    if headroom < 1 {
        return None;
    }
    Some((donor_id, receiver_id, headroom))
}

/// The Bloodweaver's cast (§C.14): the donor loses `amount` HP directly (no
/// reduction, plates or Second Wind; never below 1 HP); the receiver gains
/// it minus heal cut (Grievous Wounds, Deep Wound turn transfers into a net
/// loss). Not a heal (no Arcana/Augment bonus, no Purification) and not
/// damage (no on-hit, lifesteal, Essence).
pub(crate) fn transfer_cast(
    state: &mut MatchState,
    attacker: &Unit,
    range: i32,
    amount_param: u16,
    low_bp: u16,
    second: bool,
    ev: &mut Vec<Event>,
) {
    if let Some((donor, receiver, headroom)) = transfer_pair(state, attacker, range, [None, None]) {
        transfer_one(state, donor, receiver, amount_param, low_bp, headroom, ev);
        if second {
            if let Some((donor2, receiver2, headroom2)) =
                transfer_pair(state, attacker, range, [Some(donor), Some(receiver)])
            {
                transfer_one(
                    state,
                    donor2,
                    receiver2,
                    amount_param,
                    low_bp,
                    headroom2,
                    ev,
                );
            }
        }
    }
}

/// One donor→receiver move. `headroom` is min(donor HP − 1, receiver missing
/// HP) at selection time; the A2 double is capped by it so donors survive.
fn transfer_one(
    state: &mut MatchState,
    donor: UnitId,
    receiver: UnitId,
    amount_param: u16,
    low_bp: u16,
    headroom: i32,
    ev: &mut Vec<Event>,
) {
    let Some(di) = state.unit_index(donor) else {
        return;
    };
    let Some(ri) = state.unit_index(receiver) else {
        return;
    };
    let mut amount = i32::from(amount_param).min(headroom);
    let receiver_pct =
        i64::from(state.units[ri].hp) * 10_000 / i64::from(state.units[ri].max_hp.max(1));
    if low_bp > 0 && receiver_pct < i64::from(low_bp) {
        // Last Offering: double, still capped so the donor survives.
        amount = (amount * 2).min(state.units[di].hp - 1);
    }
    if amount <= 0 {
        return;
    }
    state.units[di].hp -= amount;
    let cut = if state.tick < state.units[ri].heal_cut_until {
        state.units[ri].heal_cut_bp
    } else {
        0
    };
    let gain = amount - amount * cut.min(10_000) / 10_000;
    let u = &mut state.units[ri];
    u.hp = (u.hp + gain).min(u.max_hp);
    ev.push(Event::Transfer {
        from: donor,
        to: receiver,
        amount,
    });
}

/// Drumguard bonds (P47, §C.6): every tick, each living Drumguard bonds to
/// the nearest allied conventional non-Drumguard within reach (ties lowest
/// id) and keeps the bond until it is dead, out of range, or another
/// candidate is at least `hysteresis` closer. The bonded ally's march
/// fields are rewritten with the best value of each bonus (no stacking).
/// Bonds are view state, not events.
pub(crate) fn bonds(state: &mut MatchState) {
    for u in state.units.iter_mut() {
        u.fx.march_move_bp = 0;
        u.fx.march_dr_bp = 0;
        u.fx.march_ignore_slow = false;
    }
    let drummers: Vec<usize> = state
        .units
        .iter()
        .enumerate()
        .filter(|(_, u)| u.hp > 0 && march_of(state, u).is_some())
        .map(|(i, _)| i)
        .collect();
    for di in &drummers {
        let id = state.units[*di].id;
        let bond = eval_bond(state, *di);
        if let Some(i) = state.unit_index(id) {
            state.units[i].fx.bond = bond;
        }
    }
    // Best value of each bonus applies when several drummers share an ally.
    let mut best: Vec<(u16, u16, bool)> = vec![(0, 0, false); state.units.len()];
    for &di in &drummers {
        let d = state.units[di];
        let Some(bond) = d.fx.bond else {
            continue;
        };
        let Some((_, move_bp, dr_bp, _, ignore)) = march_of(state, &d) else {
            continue;
        };
        if let Some(bi) = state.unit_index(bond) {
            let e = &mut best[bi];
            e.0 = e.0.max(move_bp);
            e.1 = e.1.max(dr_bp);
            e.2 = e.2 || ignore;
        }
    }
    for (i, (move_bp, dr_bp, ignore)) in best.into_iter().enumerate() {
        state.units[i].fx.march_move_bp = move_bp;
        state.units[i].fx.march_dr_bp = dr_bp;
        state.units[i].fx.march_ignore_slow = ignore;
    }
}

/// The drummer's March params, if any (absolute per form slot, like Brew).
fn march_of(state: &MatchState, u: &Unit) -> Option<(i32, u16, u16, i32, bool)> {
    abilities_of(state, u)
        .into_iter()
        .find_map(|ability| match ability {
            Ability::March {
                reach,
                move_bp,
                dr_bp,
                hysteresis,
                ignore_slow,
            } => Some((reach, move_bp, dr_bp, hysteresis, ignore_slow)),
            _ => None,
        })
}

/// The nearest eligible ally, ties lowest id. Only the minimum is used;
/// sorting and allocating every candidate adds no information.
fn bond_candidate(
    state: &MatchState,
    d: &Unit,
    reach: i32,
    exclude: Option<UnitId>,
) -> Option<(i32, UnitId)> {
    state
        .units
        .iter()
        .filter(|a| {
            a.seat == d.seat
                && a.hp > 0
                && a.id != d.id
                && Some(a.id) != exclude
                && a.kind != crate::types::UnitKind::Drumguard
                && !a.is_summon()
        })
        .filter(|a| (a.x - d.x).abs() <= reach)
        .map(|a| ((a.x - d.x).abs(), a.id))
        .min()
}

fn eval_bond(state: &MatchState, di: usize) -> Option<UnitId> {
    let d = state.units[di];
    let (reach, _, _, hysteresis, _) = march_of(state, &d)?;
    // A live bond in range holds unless another candidate is at least the
    // hysteresis closer (deterministic: nearest, then lowest id).
    if let Some(bid) = d.fx.bond {
        if let Some(bi) = state.unit_index(bid) {
            let b = &state.units[bi];
            let dist = (b.x - d.x).abs();
            if b.hp > 0 && b.seat == d.seat && dist <= reach {
                let switched = bond_candidate(state, &d, reach, Some(bid))
                    .is_some_and(|(od, _)| od + hysteresis <= dist);
                if !switched {
                    return Some(bid);
                }
            }
        }
    }
    bond_candidate(state, &d, reach, None).map(|(_, id)| id)
}

/// Cargo Crab boarding (P48-A, §C.15): every tick, each crab without a
/// passenger boards the nearest eligible ally within `board` (conventional,
/// not a Tank, not carried, not under a re-board lock; ties lowest id).
/// In practice the passenger is the next unit bought after the crab, so the
/// player chooses it through purchase order. An enemy windup already
/// targeting the passenger hits the crab instead (ally windups keep their
/// target: heals reach carried units). B2 Overdrive re-arms 220 ticks after
/// the surge lapses (a 15 s cooldown on a 4 s surge).
pub(crate) fn boarding(state: &mut MatchState, ev: &mut Vec<Event>) {
    let crabs: Vec<usize> = state
        .units
        .iter()
        .enumerate()
        .filter(|(_, u)| u.hp > 0 && u.fx.passenger.is_none() && carry_of(state, u).is_some())
        .map(|(i, _)| i)
        .collect();
    for ci in crabs {
        let crab = state.units[ci];
        let Some((board, _lock, _, _, _, surge)) = carry_of(state, &crab) else {
            continue;
        };
        let mut best: Option<(i32, UnitId)> = None;
        for a in &state.units {
            // Conventional non-Tanks only; summons and carried units never
            // board, and a re-board lock is respected.
            if a.seat != crab.seat
                || a.hp <= 0
                || a.id == crab.id
                || a.is_summon()
                || state.rules().def(a.kind).archetype == Archetype::Tank
                || a.fx.carrier.is_some()
                || state.tick < a.fx.board_lock
            {
                continue;
            }
            let dist = (a.x - crab.x).abs();
            if dist <= board && best.is_none_or(|(d, id)| (dist, a.id) < (d, id)) {
                best = Some((dist, a.id));
            }
        }
        let Some((_, pid)) = best else {
            continue;
        };
        let Some(pi) = state.unit_index(pid) else {
            continue;
        };
        // Board: sync the passenger now (run_units syncs every tick after).
        state.units[pi].fx.carrier = Some(crab.id);
        state.units[pi].x = crab.x;
        if let Some(ci) = state.unit_index(crab.id) {
            state.units[ci].fx.passenger = Some(pid);
            // B2 Overdrive: +40 % move for 4 s, re-armed 15 s after the
            // previous surge started (surge lasts 80 of those ticks).
            if surge > 0
                && (state.units[ci].fx.surge_until == 0
                    || state.tick >= state.units[ci].fx.surge_until.saturating_sub(80) + 300)
            {
                state.units[ci].fx.surge_until = state.tick.saturating_add(80);
            }
        }
        // Enemy windups already aimed at the passenger hit the crab instead;
        // so do betrayals, which never hit a carried unit (§C.10).
        for u in state.units.iter_mut() {
            if let Some((target, ticks)) = u.windup {
                if target == crate::state::Target::Unit(pid) && (u.seat != crab.seat || u.fx.betray)
                {
                    u.windup = Some((crate::state::Target::Unit(crab.id), ticks));
                }
            }
        }
        ev.push(Event::Boarded {
            crab: crab.id,
            unit: pid,
        });
    }
}

/// The crab's Carry params, if any.
fn carry_of(state: &MatchState, u: &Unit) -> Option<(i32, u16, u16, u16, u16, u16)> {
    abilities_of(state, u)
        .into_iter()
        .find_map(|ability| match ability {
            Ability::Carry {
                board,
                lock,
                dr_bp,
                move_bp,
                shield_bp,
                surge_bp,
            } => Some((board, lock, dr_bp, move_bp, shield_bp, surge_bp)),
            _ => None,
        })
}

/// Crab movement bonus while carrying (D-56: base +20% / 2 000 bp, B1 steady, B2 surge): basis points.
pub(crate) fn carry_move_bp(state: &MatchState, u: &Unit) -> u16 {
    if u.fx.passenger.is_none() {
        return 0;
    }
    const BASE_CARRY_MOVE_BP: u16 = 2_000;
    match carry_of(state, u) {
        Some((_, _, _, move_bp, _, surge_bp)) => {
            let extra = if surge_bp > 0 && state.tick < u.fx.surge_until {
                surge_bp
            } else {
                move_bp
            };
            BASE_CARRY_MOVE_BP.saturating_add(extra)
        }
        None => BASE_CARRY_MOVE_BP,
    }
}

/// Crab damage reduction while carrying (A1, joins the BR-X05 cap).
pub(crate) fn carry_dr_bp(state: &MatchState, u: &Unit) -> u16 {
    if u.fx.passenger.is_none() {
        return 0;
    }
    match carry_of(state, u) {
        Some((_, _, dr_bp, _, _, _)) => dr_bp,
        None => 0,
    }
}

/// Dismount a passenger: 0.2 tile behind the crab (toward its own base)
/// with a 3 s re-board lock (P48-A, §C.15). Called from `remove_dead`.
pub(crate) fn dismount(
    state: &mut MatchState,
    crab: UnitId,
    passenger: UnitId,
    ev: &mut Vec<Event>,
) {
    let Some(ci) = state.unit_index(crab) else {
        return;
    };
    let seat = state.units[ci].seat;
    let x = (state.units[ci].x - seat.dir() * 200).clamp(0, LANE_LEN);
    state.units[ci].fx.passenger = None;
    if let Some(pi) = state.unit_index(passenger) {
        state.units[pi].fx.carrier = None;
        state.units[pi].x = x;
        state.units[pi].fx.board_lock = state.tick.saturating_add(60);
        ev.push(Event::Dismounted { unit: passenger });
    }
}

/// The caster's Puppet params, if any (absolute per form slot, like Brew).
pub(crate) fn puppet_of(state: &MatchState, u: &Unit) -> Option<(u16, u16, u16, u8, bool)> {
    abilities_of(state, u)
        .into_iter()
        .find_map(|ability| match ability {
            Ability::Puppet {
                window,
                immune,
                bonus_bp,
                targets,
                stumble,
            } => Some((window, immune, bonus_bp, targets, stumble)),
            _ => None,
        })
}

/// Puppetmaster target ranking (§C.10): enemy conventional units in range
/// with a basic attack (damage > 0, no heal), not Confused and not immune:
/// highest static hit damage first, ties nearest, then lowest id. Warded
/// and control-immune units are skipped.
fn puppet_ranked(state: &MatchState, u: &Unit, range: i32) -> Vec<UnitId> {
    let mut out: Vec<(i32, i32, UnitId)> = Vec::new();
    for e in state.units.iter() {
        if e.seat == u.seat
            || e.hp <= 0
            || (e.x - u.x).abs() > range
            || e.is_summon()
            || state.tick < e.fx.confused_until
            || state.tick < e.fx.immune_until
            // The Antidote ward blocks the cast (§C.9, §D); Oilpress A2 is
            // immune to control, confusion included (§C.8).
            || state.tick < e.fx.ward_until
            || crate::afflictions::is_immune_to_control(e)
        {
            continue;
        }
        let st = state.stats_of(e);
        if st.damage <= 0 || st.heal > 0 {
            continue;
        }
        out.push((st.damage, (e.x - u.x).abs(), e.id));
    }
    out.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
    out.into_iter().map(|(_, _, id)| id).collect()
}

/// The next victim to confuse (highest static hit damage).
pub(crate) fn puppet_target(state: &MatchState, u: &Unit, range: i32) -> Option<UnitId> {
    puppet_ranked(state, u, range).into_iter().next()
}

/// The Puppetmaster's cast (§C.10): confuse up to `targets` distinct enemies
/// for `window` ticks, storing the betrayal bonus. The windup target (the
/// one `AttackStart` showed) goes first while still eligible. Confused
/// units betray on the next attack they start. Emits StatusApplied per victim.
pub(crate) fn puppet_cast(
    state: &mut MatchState,
    attacker: &Unit,
    target: Target,
    range: i32,
    ev: &mut Vec<Event>,
) {
    let Some((window, _immune, bonus, targets, stumble)) = puppet_of(state, attacker) else {
        return;
    };
    let mut ranked = puppet_ranked(state, attacker, range);
    if let Some(pos) = ranked.iter().position(|&id| target == Target::Unit(id)) {
        let shown = ranked.remove(pos);
        ranked.insert(0, shown);
    }
    for id in ranked.into_iter().take(usize::from(targets.max(1))) {
        if let Some(i) = state.unit_index(id) {
            state.units[i].fx.confused_until = state.tick.saturating_add(u32::from(window));
            state.units[i].fx.betray_bp = bonus;
            state.units[i].fx.betray_stumble = stumble;
            ev.push(Event::StatusApplied { unit: id });
        }
    }
}

/// Betrayal target (§C.10): the betrayer's nearest own ally in range (not
/// itself, never a carried unit, never a Stronghold). Summons count: the
/// draft only shields carried units and Strongholds.
pub(crate) fn betray_target(state: &MatchState, u: &Unit, range: i32) -> Option<UnitId> {
    state
        .units
        .iter()
        .filter(|a| {
            a.seat == u.seat
                && a.hp > 0
                && a.id != u.id
                && a.fx.carrier.is_none()
                && (a.x - u.x).abs() <= range
        })
        .map(|a| ((a.x - u.x).abs(), a.id))
        .min()
        .map(|(_, id)| id)
}

/// Windup hook (§C.10): a confused unit that starts an attack on an enemy
/// retargets its nearest own ally instead (the AttackStart already shows
/// the betrayal). Returns the windup target and whether it is a betrayal;
/// heals and casts are never redirected.
pub(crate) fn betray_windup(
    state: &MatchState,
    u: &Unit,
    range: i32,
    target: Target,
) -> (Target, bool) {
    let is_enemy_target = match target {
        Target::Unit(id) => state
            .unit_index(id)
            .is_some_and(|i| state.units[i].seat != u.seat),
        Target::Stronghold(seat) => seat != u.seat,
    };
    if !is_enemy_target || state.tick >= u.fx.confused_until {
        return (target, false);
    }
    match betray_target(state, u, range) {
        // No ally in range: the attack is normal and the confusion waits
        // for a later attack (it is not consumed).
        None => (target, false),
        Some(ally) => (Target::Unit(ally), true),
    }
}

/// The betrayal itself (§C.10): the betrayer deals its normal static hit
/// damage (Furnace multiplier and sacrifice included) through the ally's
/// damage reduction, with no splash, blast, on-hit status, lifesteal,
/// Essence, situational or Arcana-conditional bonus. A betrayed kill is a
/// normal eligible kill for the Puppetmaster's seat. Afterwards the
/// confusion ends and the unit is immune for 12 s (PUPPET_IMMUNITY).
pub(crate) fn betray_resolve(
    state: &mut MatchState,
    attacker: &Unit,
    target: Target,
    st: &Stats,
    ev: &mut Vec<Event>,
) {
    let Target::Unit(victim) = target else {
        return;
    };
    let Some(vi) = state.unit_index(victim) else {
        return;
    };
    if state.units[vi].hp <= 0 || state.units[vi].seat != attacker.seat {
        return;
    }
    let fur = crate::hits::furnace_bonus(state, attacker, target);
    let plates = crate::hits::plates_bonus(state, attacker);
    let amount = crate::hits::scale(
        st.damage,
        fur.saturating_add(plates)
            .saturating_add(i32::from(attacker.fx.betray_bp)),
    );
    let _removed = crate::hits::damage(state, target, amount.max(1), ev);
    // A2 Stumble: the betrayer is slowed 30 % for 2 s (ward-aware).
    if attacker.fx.betray_stumble {
        let _applied = crate::hits::apply_on_hit(
            state,
            attacker.id,
            OnHit::Slow {
                bp: 3_000,
                ticks: 40,
            },
            attacker.seat,
            ev,
        );
        #[cfg(feature = "lab")]
        crate::lab::status(state, attacker, _applied);
    }
    if let Some(ai) = state.unit_index(attacker.id) {
        state.units[ai].fx.betray = false;
        state.units[ai].fx.confused_until = 0;
        state.units[ai].fx.immune_until = state.tick.saturating_add(u32::from(PUPPET_IMMUNITY));
    }
    ev.push(Event::Betrayal {
        unit: attacker.id,
        ally: victim,
    });
}

/// Puppetmaster immunity after a betrayal: 12 s (§C.10).
const PUPPET_IMMUNITY: u16 = 240;
