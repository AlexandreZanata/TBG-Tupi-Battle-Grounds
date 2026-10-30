//! Resolving one hit: damage or heal, splash, situational Augment damage,
//! damage reduction, lifesteal, heal cut, Second Wind, shields and on-hit
//! statuses (GAME-DESIGN §4, §10.7).

use crate::arcana;
use crate::augment;
use crate::content::{Ability, Stats, BP};
use crate::state::{stronghold_x, Event, MatchState, Target, Unit};
use crate::targeting;
use crate::types::{OnHit, Seat, Status, UnitId};

pub(crate) fn resolve_hit(
    state: &mut MatchState,
    attacker: &Unit,
    target: Target,
    st: &Stats,
    ev: &mut Vec<Event>,
) {
    let tx = targeting::target_x(state, target);
    // A betrayed attack never heals: it hits the ally instead (§C.10).
    // Checked before healing, since the windup was retargeted onto an ally.
    if attacker.fx.betray {
        crate::support::betray_resolve(state, attacker, target, st, ev);
        return;
    }
    // Puppetmasters confuse instead of dealing damage (§C.10).
    if crate::support::puppet_of(state, attacker).is_some() {
        crate::support::puppet_cast(state, attacker, target, st.range, ev);
        return;
    }
    let healing = matches!(target, Target::Unit(id)
        if state.unit_index(id).is_some_and(|i| state.units[i].seat == attacker.seat));
    if healing {
        let Target::Unit(id) = target else { return };
        // Action units brew or transfer instead of healing.
        if let Some((strip, ward, second)) = crate::afflictions::brew_of(state, attacker) {
            crate::afflictions::brew_cast(state, attacker, st.range, id, strip, ward, second, ev);
            return;
        }
        // Bloodweavers transfer instead of healing.
        if let Some((amount, low_bp, second)) = crate::support::transfer_of(state, attacker) {
            crate::support::transfer_cast(state, attacker, st.range, amount, low_bp, second, ev);
            return;
        }
        let _restored = heal(state, attacker.seat, id, st.heal, true, ev);
        #[cfg(feature = "lab")]
        crate::lab::healed(state, attacker, _restored);
        if st.splash > 0 {
            // Index loop, no clone: heals never remove units mid-loop.
            for i in 0..state.units.len() {
                let a = state.units[i];
                let near = (a.x - tx).abs() <= st.splash;
                if a.seat == attacker.seat && a.id != id && a.hp > 0 && a.hp < a.max_hp && near {
                    let _restored = heal(state, attacker.seat, a.id, st.heal, true, ev);
                    #[cfg(feature = "lab")]
                    crate::lab::healed(state, attacker, _restored);
                }
            }
        }
        return;
    }
    let mut amount = st.damage;
    if !attacker.is_summon() {
        let target_status = match target {
            Target::Unit(id) => state
                .unit_index(id)
                .map_or(Status::None, |i| state.units[i].status),
            Target::Stronghold(_) => Status::None,
        };
        let army = state.army_of(attacker.seat);
        let arcana_bp = arcana::damage_bonus_bp(
            &state.player(attacker.seat).arcanas,
            &state.rules().arcana,
            state.rules().def(attacker.kind).archetype,
            target_status,
            army.tanks > 0,
        );
        amount = scale(
            amount,
            augment::situational_bp(state, attacker, target)
                + arcana_bp
                + conditional_bp(state, attacker, target)
                + furnace_bonus(state, attacker, target)
                + plates_bonus(state, attacker)
                + crate::stronghold::aura_bp(state, attacker),
        );
    }
    let removed = damage(state, target, amount, ev);
    #[cfg(feature = "lab")]
    if let Target::Unit(_) = target {
        crate::lab::dealt(state, attacker, removed);
    }
    if let Target::Unit(id) = target {
        let hostile = state
            .unit_index(id)
            .is_some_and(|i| state.units[i].seat != attacker.seat && state.units[i].hp > 0);
        if hostile {
            crate::afflictions::apply_stacks(state, attacker, st, id, ev);
            // P48-A: Leechroot Essence grows on HP removed from enemy
            // conventional units (not Strongholds: not a unit target;
            // not summons: conventional only; betrayal and poison pulses
            // never pass through this hook).
            let conventional = state.unit_index(id).is_some_and(|i| {
                let u = &state.units[i];
                !u.is_summon() && u.seat != attacker.seat
            });
            if conventional && removed > 0 {
                crate::afflictions::feed_essence(state, attacker, removed);
            }
        }
    }
    if let Target::Unit(id) = target {
        if !attacker.is_summon() {
            apply_heal_cut(state, attacker.seat, id);
            apply_form_heal_cut(state, attacker, id);
        }
    }
    // Lifesteal: primary hit on an enemy conventional unit only (BR-X06).
    if !attacker.is_summon() && removed > 0 {
        if let Target::Unit(id) = target {
            let conventional = state.unit_index(id).is_some_and(|i| {
                !state.units[i].is_summon() && state.units[i].seat != attacker.seat
            });
            // Form lifesteal (Leechroot B) joins the BR-X06 12 % cap with
            // Augments; the highest active tier wins (upgrades replace).
            let mut form_bp = 0;
            for tier in (1..=attacker.tier.min(2)).rev() {
                if let Some(f) = state.rules().form_of(attacker.kind, tier, attacker.branch) {
                    if f.lifesteal_bp > 0 {
                        form_bp = f.lifesteal_bp;
                        break;
                    }
                }
            }
            let bp = if form_bp > 0 {
                (augment::lifesteal_bp(&state.player(attacker.seat).augments) + form_bp).min(1_200)
            } else {
                augment::lifesteal_bp(&state.player(attacker.seat).augments)
            };
            if conventional && bp > 0 {
                let healed = (i64::from(removed) * i64::from(bp) / BP) as i32;
                let _restored = heal(state, attacker.seat, attacker.id, healed, false, ev);
                #[cfg(feature = "lab")]
                crate::lab::healed(state, attacker, _restored);
            }
        }
    }
    if let (Target::Unit(id), Some(on_hit)) = (target, st.on_hit) {
        let _applied = apply_on_hit(state, id, on_hit, attacker.seat, ev);
        #[cfg(feature = "lab")]
        crate::lab::status(state, attacker, _applied);
    }
    if st.splash > 0 {
        splash(state, attacker, target, tx, st, ev);
    } else {
        // Shatterback Rampage: plateless hits splash at the rampage radius.
        let rampage = rampage_radius(state, attacker);
        if rampage > 0 {
            let ramped = Stats {
                splash: rampage,
                splash_bp: 10_000,
                ..*st
            };
            splash(state, attacker, target, tx, &ramped, ev);
        }
    }
}

pub(crate) fn scale(value: i32, bp: i32) -> i32 {
    if bp == 0 {
        return value;
    }
    let out = i64::from(value) * (BP + i64::from(bp)) / BP;
    out.clamp(0, i64::from(i32::MAX)) as i32
}

/// Form conditional damage (P44): each active form's `vs_below_bp` /
/// `vs_above_bp` applies when the target's HP fraction meets its threshold.
/// Read at hit resolution (BR-X11); unit targets only.
fn conditional_bp(state: &MatchState, attacker: &Unit, target: Target) -> i32 {
    let Target::Unit(id) = target else {
        return 0;
    };
    let Some(i) = state.unit_index(id) else {
        return 0;
    };
    let victim = state.units[i];
    let mut bp = 0;
    let rules = state.rules();
    for tier in 1..=attacker.tier.min(2) {
        if let Some(f) = rules.form_of(attacker.kind, tier, attacker.branch) {
            if f.vs_below_bp != 0
                && i64::from(victim.hp) * BP
                    < i64::from(f.vs_below_hp_bp) * i64::from(victim.max_hp)
            {
                bp += f.vs_below_bp;
            }
            if f.vs_above_bp != 0
                && i64::from(victim.hp) * BP
                    > i64::from(f.vs_above_hp_bp) * i64::from(victim.max_hp)
            {
                bp += f.vs_above_bp;
            }
        }
    }
    bp
}

/// On-hit heal cut from the attacker's forms (P44, Farwatch B2): summed bp
/// with the longest duration, applied after the Augment cut.
fn apply_form_heal_cut(state: &mut MatchState, attacker: &Unit, id: u32) {
    let (mut bp, mut ticks) = (0, 0);
    let rules = state.rules();
    for tier in 1..=attacker.tier.min(2) {
        if let Some(f) = rules.form_of(attacker.kind, tier, attacker.branch) {
            if f.heal_cut_bp > 0 {
                bp += f.heal_cut_bp;
                ticks = ticks.max(f.heal_cut_ticks);
            }
        }
    }
    if bp <= 0 {
        return;
    }
    let until = state.tick.saturating_add(u32::from(ticks));
    if let Some(i) = state.unit_index(id) {
        state.units[i].heal_cut_until = until;
        state.units[i].heal_cut_bp = bp;
    }
}

fn apply_heal_cut(state: &mut MatchState, attacker: Seat, id: u32) {
    let bp = augment::heal_cut_bp(state, attacker);
    if bp <= 0 {
        return;
    }
    let ticks = augment::heal_cut_ticks(state, attacker);
    let until = state.tick.saturating_add(u32::from(ticks));
    if let Some(i) = state.unit_index(id) {
        state.units[i].heal_cut_until = until;
        state.units[i].heal_cut_bp = bp;
    }
}

/// Cargo Crab A2 emergency shield (P48-A): a temporary absorb pool with an
/// expiry, on the same absorb path as Shatterback plates (before them).
fn absorb_shield(state: &mut MatchState, id: UnitId, amount: i32) -> i32 {
    let Some(i) = state.unit_index(id) else {
        return amount;
    };
    let u = &mut state.units[i];
    if u.fx.shield == 0 || state.tick >= u.fx.shield_until {
        return amount;
    }
    let absorb = amount.min(i32::from(u.fx.shield));
    u.fx.shield = (u32::from(u.fx.shield).saturating_sub(absorb as u32)) as u16;
    amount - absorb
}

/// Shatterback plates (P46): damage after reduction empties the current
/// plate before HP. Each broken plate emits `PlateBroken`; an A2 plate also
/// queues shrapnel (drained in `remove_dead`, never recursive). Plates are
/// never restored: heals, regen and transfers only touch HP.
fn absorb_plates(state: &mut MatchState, id: u32, amount: i32, ev: &mut Vec<Event>) -> i32 {
    let Some(i) = state.unit_index(id) else {
        return amount;
    };
    let [a, b] = state
        .rules()
        .def(state.units[i].kind)
        .ability(state.units[i].tier, state.units[i].branch);
    let plates = [a, b].into_iter().find_map(|ability| match ability {
        Ability::Plates {
            count,
            hp,
            shrapnel,
            ..
        } => Some((count, hp, shrapnel)),
        _ => None,
    });
    let Some((_count, plate_max, shrapnel)) = plates else {
        return amount;
    };
    let mut rest = amount;
    while rest > 0 {
        let (left, cur) = {
            let u = &state.units[i];
            (u.fx.plates, i32::from(u.fx.plate_hp))
        };
        if left == 0 || cur <= 0 {
            break;
        }
        let take = rest.min(cur);
        rest -= take;
        let broken = take >= cur;
        {
            let u = &mut state.units[i];
            if broken {
                u.fx.plates = left - 1;
                u.fx.plate_hp = if u.fx.plates > 0 { plate_max as i16 } else { 0 };
            } else {
                u.fx.plate_hp = (cur - take) as i16;
            }
        }
        if broken {
            ev.push(Event::PlateBroken { unit: id });
            if shrapnel > 0 {
                let u = &state.units[i];
                state.shrapnel.push(crate::state::Shrapnel {
                    x: u.x,
                    seat: u.seat,
                    amount: i32::from(shrapnel),
                });
            }
        }
    }
    rest
}

/// Blood Furnace sacrifice (P46): when a hit resolves, the Furnace burns a
/// fraction of its current max HP — ignoring reduction, shields, plates
/// and Second Wind — and that hit deals base × (1 + 3 × lost) through one
/// truncation (BR-X04). The multiplier is off vs Strongholds, but the burn
/// is still paid. A burn to 0 HP still resolves at the maximum multiplier;
/// the Furnace then dies at the end of the tick (Deathblow).
pub(crate) fn furnace_bonus(state: &mut MatchState, attacker: &Unit, target: Target) -> i32 {
    let [a, b] = state
        .rules()
        .def(attacker.kind)
        .ability(attacker.tier, attacker.branch);
    let furnace = [a, b].into_iter().find_map(|ability| match ability {
        Ability::Furnace { burn_bp, max_bp } => Some((burn_bp, max_bp)),
        _ => None,
    });
    let Some((burn_bp, max_bp)) = furnace else {
        return 0;
    };
    // Root (no windup) and dead targets cancel at no cost: only a resolving
    // hit burns.
    let resolves = match target {
        Target::Unit(id) => state.unit_index(id).is_some_and(|i| state.units[i].hp > 0),
        Target::Stronghold(seat) => state.player(seat).stronghold_hp > 0,
    };
    if !resolves {
        return 0;
    }
    let max_hp = attacker.max_hp.max(1);
    let burn = (i64::from(max_hp) * i64::from(burn_bp) / 10_000).max(1) as i32;
    if let Some(ai) = state.unit_index(attacker.id) {
        state.units[ai].hp = (state.units[ai].hp - burn).max(0);
    }
    if matches!(target, Target::Stronghold(_)) {
        return 0;
    }
    let hp_after = state
        .unit_index(attacker.id)
        .map_or(0, |ai| state.units[ai].hp);
    let lost_bp = i64::from(max_hp - hp_after) * 10_000 / i64::from(max_hp);
    (i64::from(max_bp) * lost_bp / 10_000) as i32
}

/// Shatterback Unleashed (P46): +8 % damage per broken plate.
pub(crate) fn plates_bonus(state: &MatchState, attacker: &Unit) -> i32 {
    let [a, b] = state
        .rules()
        .def(attacker.kind)
        .ability(attacker.tier, attacker.branch);
    let bonus = [a, b].into_iter().find_map(|ability| match ability {
        Ability::Plates { count, dmg_bp, .. } => Some((count, dmg_bp)),
        _ => None,
    });
    let Some((count, dmg_bp)) = bonus else {
        return 0;
    };
    if dmg_bp == 0 {
        return 0;
    }
    let broken = count.saturating_sub(attacker.fx.plates);
    i32::from(dmg_bp) * i32::from(broken)
}

/// Shatterback Rampage (P46): with no plates left, hits splash at the
/// rampage radius (enemies only, full share).
fn rampage_radius(state: &MatchState, attacker: &Unit) -> i32 {
    let [a, b] = state
        .rules()
        .def(attacker.kind)
        .ability(attacker.tier, attacker.branch);
    let rampage = [a, b].into_iter().find_map(|ability| match ability {
        Ability::Plates { rampage, .. } => Some(rampage),
        _ => None,
    });
    match rampage {
        Some(r) if r > 0 && attacker.fx.plates == 0 => r,
        _ => 0,
    }
}

/// Splash around the primary impact (P44): the attacker's splash share of
/// the hit. A friendly blast (Bombard) reaches every unit of both seats —
/// allies, the attacker itself — and the enemy Stronghold, but never the
/// own Stronghold. Situational bonuses and statuses stay primary-only.
fn splash(
    state: &mut MatchState,
    attacker: &Unit,
    primary: Target,
    tx: i32,
    st: &Stats,
    ev: &mut Vec<Event>,
) {
    let enemy = attacker.seat.other();
    let friendly = state
        .rules()
        .def(attacker.kind)
        .ability(attacker.tier, attacker.branch)
        .contains(&Ability::Blast { friendly: true });
    let splash_carry = st.on_hit.is_some()
        && state
            .rules()
            .form_of(attacker.kind, attacker.tier, attacker.branch)
            .is_some_and(|f| f.splash_hit);
    let amount = scale(st.damage, st.splash_bp - 10_000);
    // Index loop, no clone: dead units are only removed in `remove_dead`.
    for i in 0..state.units.len() {
        let e = state.units[i];
        let near = (e.x - tx).abs() <= st.splash;
        if e.hp > 0 && near && primary != Target::Unit(e.id) && (friendly || e.seat == enemy) {
            let _removed = damage(state, Target::Unit(e.id), amount, ev);
            #[cfg(feature = "lab")]
            crate::lab::dealt(state, attacker, _removed);
            if !attacker.is_summon() {
                apply_heal_cut(state, attacker.seat, e.id);
                apply_form_heal_cut(state, attacker, e.id);
            }
            // Oilpress B2 (P47): the slow reaches every splashed unit.
            if splash_carry {
                if let Some(on_hit) = st.on_hit {
                    let _applied = apply_on_hit(state, e.id, on_hit, attacker.seat, ev);
                    #[cfg(feature = "lab")]
                    crate::lab::status(state, attacker, _applied);
                }
            }
        }
    }
    if friendly {
        let base = Target::Stronghold(enemy);
        if primary != base && (stronghold_x(enemy) - tx).abs() <= st.splash {
            damage(state, base, amount, ev);
        }
        return;
    }
    let base = Target::Stronghold(enemy);
    if primary != base && (stronghold_x(enemy) - tx).abs() <= st.splash {
        damage(state, base, amount, ev);
    }
}

/// Apply damage after the target owner's Augment reduction and shield;
/// returns the HP actually removed.
pub(crate) fn damage(
    state: &mut MatchState,
    target: Target,
    amount: i32,
    ev: &mut Vec<Event>,
) -> i32 {
    let amount = amount.max(0);
    match target {
        Target::Unit(id) => {
            let Some(i) = state.unit_index(id) else {
                return 0;
            };
            let victim = state.units[i];
            let owner = victim.seat;
            let conventional = !victim.is_summon();
            let archetype = state.rules().def(victim.kind).archetype;
            let army = state.army_of(owner);
            let arcana_dr = if conventional {
                arcana::damage_reduction_bp(
                    &state.player(owner).arcanas,
                    &state.rules().arcana,
                    archetype,
                    army.tanks > 0,
                    army.dps > 0,
                )
            } else {
                0
            };
            let dr = (augment::damage_reduction_bp(state, &victim)
                + arcana_dr
                + i32::from(victim.fx.march_dr_bp)
                + i32::from(crate::support::carry_dr_bp(state, &victim)))
            .min(2_000);
            let amount = if dr > 0 {
                let reduced = i64::from(amount) * (BP - i64::from(dr)) / BP;
                if amount >= 1 {
                    (reduced as i32).max(1)
                } else {
                    0
                }
            } else {
                amount
            };
            // P48-A: Cargo Crab A2 emergency shield absorbs before plates.
            let amount = absorb_shield(state, id, amount);
            // P46: Shatterback plates absorb after damage reduction, before HP.
            let amount = absorb_plates(state, id, amount, ev);
            let copies = augment::second_wind_charges(&state.player(owner).augments);
            let u = &mut state.units[i];
            if u.hp <= 0 {
                return 0;
            }
            let before = u.hp;
            if !u.is_summon() && amount >= before && u.second_wind_used < copies {
                u.hp = 1;
                u.second_wind_used = u.second_wind_used.saturating_add(1);
                ev.push(Event::SecondWind { unit: id });
            } else {
                u.hp = (u.hp - amount).max(0);
            }
            let removed = before - u.hp;
            ev.push(Event::Hit { to: target, amount });
            if removed > 0 && u.hp <= 0 && conventional {
                state.army[owner.index()].remove(archetype);
            }
            #[cfg(feature = "lab")]
            crate::lab::taken(state, owner, victim.kind, removed);
            removed
        }
        Target::Stronghold(seat) => {
            let tick = state.tick;
            let p = state.player_mut(seat);
            let between = p.stronghold_hp;
            let mut incoming = amount;
            if tick < p.shield_until && p.shield > 0 {
                let absorbed = p.shield.min(incoming);
                p.shield -= absorbed;
                incoming -= absorbed;
            }
            p.stronghold_hp = (p.stronghold_hp - incoming).max(0);
            let removed = between - p.stronghold_hp;
            ev.push(Event::Hit { to: target, amount });
            if removed > 0 {
                augment::after_stronghold_damage(state, seat, ev);
            }
            #[cfg(feature = "lab")]
            if removed > 0 {
                crate::lab::progress(state);
            }
            removed
        }
    }
}

fn heal(
    state: &mut MatchState,
    healer: Seat,
    id: u32,
    amount: i32,
    direct: bool,
    ev: &mut Vec<Event>,
) -> i32 {
    let Some(i) = state.unit_index(id) else {
        return 0;
    };
    let target = state.units[i];
    let effective = augment::heal_amount(state, healer, &target, amount, direct);
    if effective <= 0 {
        return 0;
    }
    let u = &mut state.units[i];
    u.hp = (u.hp + effective).min(u.max_hp);
    ev.push(Event::Heal {
        to: id,
        amount: effective,
    });
    if !target.is_summon() {
        if let Some(status) = arcana::purify_status(
            &state.player(healer).arcanas,
            &state.rules().arcana,
            target.status,
        ) {
            state.units[i].status = status;
        }
    }
    effective
}

fn status_ticks(base: u16, bp: i32) -> u16 {
    if bp <= 0 {
        return base;
    }
    (i64::from(base) * (BP + i64::from(bp)) / BP).min(i64::from(u16::MAX)) as u16
}

/// Root replaces anything (keeping the longer root); a stronger slow
/// replaces a slow, an equal or weaker one refreshes it; slow never
/// replaces root. Only duration is modified by Augments (BR-X07).
pub(crate) fn apply_on_hit(
    state: &mut MatchState,
    id: u32,
    on_hit: OnHit,
    attacker: Seat,
    ev: &mut Vec<Event>,
) -> u16 {
    let Some(i) = state.unit_index(id) else {
        return 0;
    };
    // A ward ignores incoming slows, roots (and confusion): damage is not
    // blocked (§C.9). Oilpress A2 is immune to control outright (§C.8).
    if state.tick < state.units[i].fx.ward_until
        || crate::afflictions::is_immune_to_control(&state.units[i])
    {
        return 0;
    }
    let bonus = augment::status_duration_bp(&state.player(attacker).augments);
    let target = state.units[i];
    let archetype = state.rules().def(target.kind).archetype;
    let shorten = |ticks| {
        if target.is_summon() {
            ticks
        } else {
            arcana::shorten_status(
                &state.player(target.seat).arcanas,
                &state.rules().arcana,
                archetype,
                ticks,
            )
        }
    };
    let on_hit = match on_hit {
        OnHit::Slow { bp, ticks } => OnHit::Slow {
            bp,
            ticks: shorten(status_ticks(ticks, bonus)),
        },
        OnHit::Root { ticks } => OnHit::Root {
            ticks: shorten(status_ticks(ticks, bonus)),
        },
    };
    let applied = match on_hit {
        OnHit::Slow { ticks, .. } => ticks,
        OnHit::Root { ticks } => ticks,
    };
    let u = &mut state.units[i];
    if u.hp <= 0 {
        return 0;
    }
    u.status = match (on_hit, u.status) {
        (OnHit::Root { ticks }, Status::Rooted { ticks_left }) => Status::Rooted {
            ticks_left: ticks.max(ticks_left),
        },
        (OnHit::Root { ticks }, _) => Status::Rooted { ticks_left: ticks },
        (OnHit::Slow { .. }, rooted @ Status::Rooted { .. }) => rooted,
        (
            OnHit::Slow { bp, ticks },
            Status::Slowed {
                bp: old,
                ticks_left,
            },
        ) if bp <= old => Status::Slowed {
            bp: old,
            ticks_left: ticks_left.max(ticks),
        },
        (OnHit::Slow { bp, ticks }, _) => Status::Slowed {
            bp,
            ticks_left: ticks,
        },
    };
    ev.push(Event::StatusApplied { unit: id });
    applied
}
