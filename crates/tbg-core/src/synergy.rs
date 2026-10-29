//! Faction synergies (BR-Y01, BR-Y02): levels from living-unit counts,
//! stat refresh on level change, Plants regeneration.

use crate::content::{self, SynergyEffect, BP, REGEN_PERIOD, SYNERGIES, SYNERGY_THRESHOLDS};
use crate::state::{Event, MatchState};
use crate::types::{Seat, FACTIONS};

pub(crate) fn update(state: &mut MatchState, ev: &mut Vec<Event>) {
    for seat in Seat::BOTH {
        // u32: armies are uncapped (BR-U07); summons never count (BR-U08).
        let mut counts = [0u32; 4];
        for u in state
            .units
            .iter()
            .filter(|u| u.seat == seat && u.hp > 0 && !u.kind.is_summon())
        {
            counts[content::def(u.kind).faction.index()] += 1;
        }
        for faction in FACTIONS {
            let level = level_for(counts[faction.index()]);
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
            for idx in 0..state.units.len() {
                let u = state.units[idx];
                if u.seat == seat && content::def(u.kind).faction == faction {
                    state.refresh_max_hp(idx);
                }
            }
        }
    }
    if state.tick.is_multiple_of(REGEN_PERIOD) {
        regenerate(state);
    }
}

fn level_for(count: u32) -> u8 {
    SYNERGY_THRESHOLDS
        .iter()
        .filter(|&&t| count >= u32::from(t))
        .count() as u8
}

/// Units of a faction with a `Regen` synergy heal a share of max HP.
fn regenerate(state: &mut MatchState) {
    for idx in 0..state.units.len() {
        let u = state.units[idx];
        let faction = content::def(u.kind).faction.index();
        let level = state.player(u.seat).synergy[faction];
        let (effect, bps) = SYNERGIES[faction];
        if level == 0 || effect != SynergyEffect::Regen || u.hp <= 0 || u.kind.is_summon() {
            continue;
        }
        let bp = i64::from(bps[usize::from(level) - 1]);
        let heal = (i64::from(u.max_hp) * bp / BP) as i32;
        let unit = &mut state.units[idx];
        unit.hp = (unit.hp + heal).min(unit.max_hp);
    }
}
