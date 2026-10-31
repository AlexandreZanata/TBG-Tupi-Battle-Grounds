//! Faction synergies (BR-Y01, BR-Y02): levels from living-unit counts,
//! stat refresh on level change, Plants regeneration.

use crate::augment;
use crate::content::{SynergyEffect, BP, REGEN_PERIOD};
use crate::state::{Event, MatchState};
use crate::types::{Seat, UnitKind, FACTIONS, FACTION_COUNT};

pub(crate) fn update(state: &mut MatchState, ev: &mut Vec<Event>) {
    let rules = state.rules;
    for seat in Seat::BOTH {
        // u32: armies are uncapped (BR-U07); summons never count (BR-U08).
        let mut counts = [0u32; FACTION_COUNT];
        for u in state
            .units
            .iter()
            .filter(|u| u.seat == seat && u.hp > 0 && !u.is_summon())
        {
            counts[rules.def(u.kind).faction.index()] += 1;
        }
        for faction in FACTIONS {
            let level = level_for(rules.synergy_thresholds, counts[faction.index()]);
            let current = &mut state.player_mut(seat).synergy[faction.index()];
            if *current == level {
                continue;
            }
            *current = level;
            ev.push(Event::SynergyChanged {
                seat,
                faction,
                level,
            });
            // Convergence is conditional on two active Synergies, so a
            // level change can move every unit; otherwise only the faction.
            let convergence = augment::unit_bonus(
                &state.player(seat).augments,
                state.player(seat).synergy,
                UnitKind::Shieldbearer,
                0,
            );
            if convergence.hp_bp > 0 || convergence.dmg_bp > 0 {
                state.refresh_units(seat);
            } else {
                for idx in 0..state.units.len() {
                    let u = state.units[idx];
                    if u.seat == seat && rules.def(u.kind).faction == faction {
                        state.refresh_max_hp(idx);
                    }
                }
            }
        }
    }
    if state.tick.is_multiple_of(REGEN_PERIOD) {
        regenerate(state);
    }
}

fn level_for(thresholds: [u8; 2], count: u32) -> u8 {
    thresholds
        .iter()
        .filter(|&&t| count >= u32::from(t))
        .count() as u8
}

/// Units of a faction with a `Regen` synergy heal a share of max HP.
fn regenerate(state: &mut MatchState) {
    let rules = state.rules;
    for idx in 0..state.units.len() {
        let u = state.units[idx];
        let faction = rules.def(u.kind).faction.index();
        let level = state.player(u.seat).synergy[faction];
        let (effect, bps) = rules.synergies[faction];
        if level == 0 || effect != SynergyEffect::Regen || u.hp <= 0 || u.is_summon() {
            continue;
        }
        let bp = i64::from(bps[usize::from(level) - 1]);
        let mut heal = (i64::from(u.max_hp) * bp / BP) as i32;
        let cut = crate::roster_slots::heal_cut_now(&u, state.tick);
        if cut > 0 {
            heal = (i64::from(heal) * i64::from(10_000 - cut).max(0) / 10_000) as i32;
        }
        if heal <= 0 {
            continue;
        }
        let unit = &mut state.units[idx];
        unit.hp = (unit.hp + heal).min(unit.max_hp);
        let restored = unit.hp - u.hp;
        crate::training_hooks::scoped(
            state,
            crate::training_hooks::Source::Team(u.seat),
            crate::training_hooks::Effect::Regen,
            |s| crate::training_hooks::restored(s, &u, restored),
        );
    }
}
