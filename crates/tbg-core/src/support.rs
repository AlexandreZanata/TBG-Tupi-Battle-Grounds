//! Ally-support actions (P46–P47): Bloodweaver transfers, Drumguard bonds.
//! Cargo Crab boarding joins in P48 (ROSTER-EXPANSION §F.3).

use std::cmp::Reverse;

use crate::afflictions::abilities_of;
use crate::content::Ability;
use crate::state::{Event, MatchState, Unit};
use crate::types::UnitId;

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
    abilities_of(state, u).into_iter().find_map(|ability| match ability {
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

/// Bonded ally candidates: living allied conventional non-Drummers in
/// range; nearest first, ties lowest id.
fn bond_candidates(
    state: &MatchState,
    d: &Unit,
    reach: i32,
    exclude: Option<UnitId>,
) -> Vec<(i32, UnitId)> {
    let mut out: Vec<(i32, UnitId)> = state
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
        .collect();
    out.sort();
    out
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
                let switched = bond_candidates(state, &d, reach, Some(bid))
                    .into_iter()
                    .next()
                    .is_some_and(|(od, _)| od + hysteresis <= dist);
                if !switched {
                    return Some(bid);
                }
            }
        }
    }
    bond_candidates(state, &d, reach, None)
        .into_iter()
        .next()
        .map(|(_, id)| id)
}

/// Oilpress formation anchor check (§C.8): an Oilpress counts as engaged
/// while a living enemy unit stands within `engage` in front of it.
pub(crate) fn engaged_oilpress(state: &MatchState, idx: usize) -> Option<(i32, i32)> {
    let u = &state.units[idx];
    let [a, b] = state.rules().def(u.kind).ability(u.tier, u.branch);
    let formation = [a, b].into_iter().find_map(|ability| match ability {
        Ability::Formation { engage, gap } => Some((engage, gap)),
        _ => None,
    });
    let (engage, gap) = formation?;
    let dir = u.seat.dir();
    let engaged = state.units.iter().any(|e| {
        e.seat != u.seat && e.hp > 0 && (e.x - u.x) * dir > 0 && (e.x - u.x).abs() <= engage
    });
    engaged.then_some((u.x, gap))
}
