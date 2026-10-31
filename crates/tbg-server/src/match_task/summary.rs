//! P61 §4: bounded per-side match summary for ranked statistics. Fixed
//! arrays indexed by catalog IDs, updated on successful commands and folded
//! once per tick from the semantic events the step already produced: no
//! event log, no per-unit map, no SQL, and no change to `MatchState::step`.

use serde_json::{json, Value};

use tbg_core::content::{AUGMENT_CATALOG, AUGMENT_OPTIONS, RUNNING_START};
use tbg_core::ruleset_version::SUMMARY_VERSION;
use tbg_core::types::ALL_KINDS;
use tbg_core::{Event, MatchState, Seat, UnitId, UnitKind};

/// Hard bound of the encoded summary (P61 §4, UTF-8 bytes).
pub const SUMMARY_CAP: usize = 16_384;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UnitSummary {
    pub bought: u32,
    /// Combat-relative tick of the first successful purchase.
    pub first_buy_tick: Option<u32>,
    /// Observed paid forms: bits A1, A2, B1, B2.
    pub evolved_forms: u8,
    /// Highest tier reached by any copy, dead copies included.
    pub max_tier: u8,
    /// Successful transitions to tier 1 and to tier 2.
    pub evolved_copies: [u32; 2],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SideSummary {
    pub units: [UnitSummary; ALL_KINDS.len()],
    pub synergy_max: [u8; tbg_core::FACTION_COUNT],
    /// Combat ticks at level >= 1 and >= 2, per Faction.
    pub synergy_ticks: [[u32; 2]; tbg_core::FACTION_COUNT],
    pub augment_offered: [u32; AUGMENT_CATALOG],
    pub augment_chosen: [u32; AUGMENT_CATALOG],
    pub closed_rounds: u32,
    pub cancelled_rounds: u32,
    /// Current Synergy level and the combat tick it started.
    level: [u8; tbg_core::FACTION_COUNT],
    since: [u32; tbg_core::FACTION_COUNT],
}

impl Default for SideSummary {
    fn default() -> SideSummary {
        SideSummary {
            units: [UnitSummary::default(); ALL_KINDS.len()],
            synergy_max: [0; tbg_core::FACTION_COUNT],
            synergy_ticks: [[0; 2]; tbg_core::FACTION_COUNT],
            augment_offered: [0; AUGMENT_CATALOG],
            augment_chosen: [0; AUGMENT_CATALOG],
            closed_rounds: 0,
            cancelled_rounds: 0,
            level: [0; tbg_core::FACTION_COUNT],
            since: [0; tbg_core::FACTION_COUNT],
        }
    }
}

impl SideSummary {
    /// Close the open interval of Faction `f` at combat tick `now`.
    fn close(&mut self, f: usize, now: u32) {
        let span = now.saturating_sub(self.since[f]);
        if self.level[f] >= 1 {
            self.synergy_ticks[f][0] = self.synergy_ticks[f][0].saturating_add(span);
        }
        if self.level[f] >= 2 {
            self.synergy_ticks[f][1] = self.synergy_ticks[f][1].saturating_add(span);
        }
        self.since[f] = now;
    }
}

/// The open Augment round as seen from events: its options, captured once
/// at `RoundOpened`, and which seats already resolved it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct OpenRound {
    round: u32,
    options: [u8; AUGMENT_OPTIONS],
    resolved: [bool; 2],
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SummaryCollector {
    pub sides: [SideSummary; 2],
    open: Option<OpenRound>,
    /// Set when any counter saturated: the match leaves every denominator.
    pub saturated: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SummaryError {
    Overflow,
    InvalidCatalog,
    TooLarge,
}

fn combat_tick(state: &MatchState) -> u32 {
    state.tick.saturating_sub(RUNNING_START)
}

fn bump(counter: &mut u32, saturated: &mut bool) {
    match counter.checked_add(1) {
        Some(n) => *counter = n,
        None => *saturated = true,
    }
}

impl SummaryCollector {
    /// A successful Shop purchase of `kind` (captured from the validated
    /// result, never from a rejected command or a generated unit).
    pub fn purchased(&mut self, seat: Seat, kind: UnitKind, state: &MatchState) {
        let u = &mut self.sides[seat.index()].units[kind as usize];
        bump(&mut u.bought, &mut self.saturated);
        u.first_buy_tick.get_or_insert(combat_tick(state));
    }

    /// A successful Evolution of `unit`, read after it was applied.
    pub fn evolved(&mut self, seat: Seat, unit: UnitId, state: &MatchState) {
        let Some(i) = state.unit_index(unit) else {
            return;
        };
        let u = state.units[i];
        let (tier, branch) = (u.tier.min(2), u.branch.min(1));
        if tier == 0 || u.kind.is_summon() {
            return;
        }
        let s = &mut self.sides[seat.index()].units[u.kind as usize];
        s.evolved_forms |= 1 << (usize::from(branch) * 2 + usize::from(tier) - 1);
        s.max_tier = s.max_tier.max(tier);
        bump(
            &mut s.evolved_copies[usize::from(tier) - 1],
            &mut self.saturated,
        );
    }

    /// Fold one tick's events, once, before they fan out to recipients.
    pub fn observe(&mut self, events: &[Event], state: &MatchState) {
        // Events of this step happened at the tick before the increment.
        let now = state.tick.saturating_sub(1).saturating_sub(RUNNING_START);
        for e in events {
            match *e {
                Event::SynergyChanged {
                    seat,
                    faction,
                    level,
                } => {
                    let side = &mut self.sides[seat.index()];
                    let f = faction.index();
                    side.close(f, now);
                    side.level[f] = level;
                    side.synergy_max[f] = side.synergy_max[f].max(level);
                }
                Event::RoundOpened { round } => {
                    self.open = Some(OpenRound {
                        round,
                        options: state.augments.options,
                        resolved: [false; 2],
                    });
                }
                Event::AugmentChosen { seat, id } => self.resolve(seat, Some(id)),
                Event::Forgone { seat, .. } => self.resolve(seat, None),
                _ => {}
            }
        }
    }

    /// One completed seat-round: every offered id gains one opportunity;
    /// a duplicate resolution of the same seat is ignored.
    fn resolve(&mut self, seat: Seat, chosen: Option<u8>) {
        let Some(open) = self.open.as_mut() else {
            return;
        };
        if open.resolved[seat.index()] {
            return;
        }
        open.resolved[seat.index()] = true;
        let options = open.options;
        let side = &mut self.sides[seat.index()];
        for id in options {
            if let Some(c) = side.augment_offered.get_mut(usize::from(id)) {
                bump(c, &mut self.saturated);
            }
        }
        if let Some(c) = chosen.and_then(|id| side.augment_chosen.get_mut(usize::from(id))) {
            bump(c, &mut self.saturated);
        }
        bump(&mut side.closed_rounds, &mut self.saturated);
    }

    /// Close intervals and encode JSON v1 (P61 §4). A cancelled round is
    /// counted, never turned into four declined choices.
    pub fn finish(
        mut self,
        state: &MatchState,
        wall_ms: u64,
        catalog_hash: u64,
    ) -> Result<Value, SummaryError> {
        let end = combat_tick(state);
        if let Some(open) = self.open.take() {
            for seat in Seat::BOTH {
                if !open.resolved[seat.index()] {
                    let side = &mut self.sides[seat.index()];
                    bump(&mut side.cancelled_rounds, &mut self.saturated);
                }
            }
        }
        if self.saturated {
            return Err(SummaryError::Overflow);
        }
        let sides: Vec<Value> = Seat::BOTH
            .iter()
            .map(|&seat| {
                let side = &mut self.sides[seat.index()];
                for f in 0..tbg_core::FACTION_COUNT {
                    side.close(f, end);
                }
                encode_side(side, state, seat)
            })
            .collect();
        let v = json!({
            "schema": SUMMARY_VERSION,
            "catalog_hash": format!("{catalog_hash:016x}"),
            "arena": state.arena.index(),
            "combat_ticks": end,
            "wall_ms": wall_ms,
            "sides": sides,
        });
        validate(&v)?;
        Ok(v)
    }
}

fn encode_side(side: &SideSummary, state: &MatchState, seat: Seat) -> Value {
    let units: Vec<Value> = side
        .units
        .iter()
        .enumerate()
        .filter(|(_, u)| u.bought > 0 || u.evolved_forms != 0)
        .map(|(k, u)| {
            json!([
                k,
                u.bought,
                u.first_buy_tick,
                u.evolved_forms,
                u.max_tier,
                u.evolved_copies[0],
                u.evolved_copies[1],
            ])
        })
        .collect();
    let sparse = |counts: &[u32]| -> Vec<Value> {
        counts
            .iter()
            .enumerate()
            .filter(|(_, &n)| n > 0)
            .map(|(id, &n)| json!([id, n]))
            .collect()
    };
    let p = state.player(seat);
    json!({
        "units": units,
        "synergy_max": side.synergy_max,
        "synergy_ticks": side.synergy_ticks,
        "offered": sparse(&side.augment_offered),
        "chosen": sparse(&side.augment_chosen),
        "rounds": [side.closed_rounds, side.cancelled_rounds],
        "stronghold": [p.stronghold.index(), usize::from(p.fort_tier), usize::from(p.fort_branch)],
        "arcanas": p.arcanas.ranks,
        "complete": true,
    })
}

/// The P61 §4 checks before storage: catalog bounds, form bits, choices
/// within offers, two seats, complete Arcana ranks and the byte cap.
pub fn validate(v: &Value) -> Result<(), SummaryError> {
    if v.to_string().len() > SUMMARY_CAP {
        return Err(SummaryError::TooLarge);
    }
    let sides = v["sides"].as_array().ok_or(SummaryError::InvalidCatalog)?;
    if sides.len() != 2 {
        return Err(SummaryError::InvalidCatalog);
    }
    for side in sides {
        let ok = side_is_valid(side).ok_or(SummaryError::InvalidCatalog)?;
        if !ok {
            return Err(SummaryError::InvalidCatalog);
        }
    }
    Ok(())
}

fn side_is_valid(side: &Value) -> Option<bool> {
    let mut last = None;
    for u in side["units"].as_array()? {
        let kind = u.get(0)?.as_u64()?;
        let bought = u.get(1)?.as_u64()?;
        let forms = u.get(3)?.as_u64()?;
        let (t1, t2) = (u.get(5)?.as_u64()?, u.get(6)?.as_u64()?);
        let sorted = last.is_none_or(|l| kind > l);
        last = Some(kind);
        // No generated unit can evolve at this baseline: tier2 <= tier1 <= bought.
        if !sorted || kind as usize >= ALL_KINDS.len() || forms > 0b1111 || t2 > t1 || t1 > bought {
            return Some(false);
        }
    }
    let pairs = |key: &str| -> Option<Vec<(u64, u64)>> {
        side[key]
            .as_array()?
            .iter()
            .map(|p| Some((p.get(0)?.as_u64()?, p.get(1)?.as_u64()?)))
            .collect()
    };
    let (offered, chosen) = (pairs("offered")?, pairs("chosen")?);
    for (id, n) in &chosen {
        let offers = offered.iter().find(|(o, _)| o == id).map_or(0, |(_, c)| *c);
        if *id as usize >= AUGMENT_CATALOG || *n > offers {
            return Some(false);
        }
    }
    Some(side["arcanas"].as_array()?.len() == tbg_core::arcana::NODES)
}

#[cfg(test)]
mod tests {
    //! P61 §4 E0 collector rules (BR-ST, D-88).
    use super::*;
    use tbg_core::{Faction, PlayerIntention};

    /// A running canonical match with plenty of Energy on seat A.
    fn running() -> MatchState {
        let mut state = MatchState::new(61);
        while state.phase != tbg_core::Phase::Running {
            state.step();
        }
        state.players[0].energy = 1_000_000;
        state
    }

    fn buy(state: &mut MatchState, c: &mut SummaryCollector, seq: u32) -> UnitId {
        state
            .apply_intention(Seat::A, seq, PlayerIntention::Purchase { offer: 0 })
            .expect("purchase");
        let u = *state.units.last().expect("bought unit");
        c.purchased(Seat::A, u.kind, state);
        u.id
    }

    #[test]
    fn evolution_survives_the_unit_and_respects_copy_order() {
        let mut state = running();
        let mut c = SummaryCollector::default();
        let id = buy(&mut state, &mut c, 1);
        let kind = state.units[state.unit_index(id).unwrap()].kind;
        for (seq, branch) in [(2, 1), (3, 1)] {
            state
                .apply_intention(Seat::A, seq, PlayerIntention::Evolve { unit: id, branch })
                .expect("evolve");
            c.evolved(Seat::A, id, &state);
        }
        // The unit dies: history stays (no inference from survivors).
        state.units.retain(|u| u.id != id);
        let u = c.sides[0].units[kind as usize];
        assert_eq!(u.bought, 1);
        assert_eq!(u.evolved_forms, 0b1100, "B1 and B2");
        assert_eq!((u.max_tier, u.evolved_copies), (2, [1, 1]));
        assert_eq!(u.first_buy_tick, Some(state.tick - RUNNING_START));
        let v = c.finish(&state, 1_000, 7).expect("summary");
        let row = &v["sides"][0]["units"][0];
        assert_eq!(row[0], kind as usize);
        assert_eq!(row[3], 0b1100);
        assert_eq!((row[5].as_u64(), row[6].as_u64()), (Some(1), Some(1)));
    }

    #[test]
    fn rounds_count_opportunities_once_and_cancellation_is_not_a_decline() {
        let mut state = running();
        let mut c = SummaryCollector::default();
        state.augments.options = [3, 5, 7, 9];
        c.observe(&[Event::RoundOpened { round: 1 }], &state);
        let chosen = Event::AugmentChosen {
            seat: Seat::A,
            id: 5,
        };
        c.observe(&[chosen, chosen], &state); // duplicate: counted once
        c.observe(
            &[Event::Forgone {
                seat: Seat::B,
                round: 1,
            }],
            &state,
        );
        state.augments.options = [1, 2, 3, 4];
        c.observe(&[Event::RoundOpened { round: 2 }], &state);
        c.observe(
            &[Event::Forgone {
                seat: Seat::A,
                round: 2,
            }],
            &state,
        );
        let a = &c.sides[0];
        assert_eq!((a.augment_offered[3], a.augment_offered[5]), (2, 1));
        assert_eq!(a.augment_chosen[5], 1);
        assert_eq!(c.sides[1].augment_chosen.iter().sum::<u32>(), 0);
        let v = c.finish(&state, 0, 0).expect("summary");
        assert_eq!(v["sides"][0]["rounds"], json!([2, 0]));
        // Seat B never resolved round 2: one cancelled round, no declines.
        assert_eq!(v["sides"][1]["rounds"], json!([1, 1]));
        assert_eq!(
            v["sides"][1]["offered"],
            json!([[3, 1], [5, 1], [7, 1], [9, 1]])
        );
    }

    #[test]
    fn synergy_uptime_integrates_level_intervals_in_combat_ticks() {
        let mut state = running();
        let mut c = SummaryCollector::default();
        let change = |level| Event::SynergyChanged {
            seat: Seat::A,
            faction: Faction::Plants,
            level,
        };
        let at = |state: &mut MatchState, t: u32| state.tick = RUNNING_START + t + 1;
        at(&mut state, 100);
        c.observe(&[change(1)], &state);
        at(&mut state, 300);
        c.observe(&[change(2)], &state);
        at(&mut state, 350);
        c.observe(&[change(0)], &state);
        state.tick = RUNNING_START + 1_000;
        let plants = Faction::Plants.index();
        let v = c.finish(&state, 0, 0).expect("summary");
        assert_eq!(v["sides"][0]["synergy_ticks"][plants], json!([250, 50]));
        assert_eq!(v["sides"][0]["synergy_max"][plants], 2);
    }

    #[test]
    fn generated_units_and_rejections_are_never_purchases() {
        let mut state = running();
        let mut c = SummaryCollector::default();
        // A rejected (stale) purchase never reaches `purchased`.
        assert!(state
            .apply_intention(Seat::B, 0, PlayerIntention::Purchase { offer: 9 })
            .is_err());
        // Summons appear only through events; the collector ignores Spawn.
        c.observe(
            &[Event::Spawn {
                unit: 99,
                seat: Seat::A,
                kind: UnitKind::Beetling,
            }],
            &state,
        );
        let v = c.finish(&state, 0, 0).expect("summary");
        assert_eq!(v["sides"][0]["units"], json!([]));
        assert_eq!(v["sides"][1]["units"], json!([]));
    }

    #[test]
    fn saturation_and_invalid_evidence_are_rejected() {
        let state = running();
        let mut c = SummaryCollector::default();
        c.sides[0].units[0].bought = u32::MAX;
        c.purchased(Seat::A, ALL_KINDS[0], &state);
        assert_eq!(c.clone().finish(&state, 0, 0), Err(SummaryError::Overflow));
        let mut good = SummaryCollector::default()
            .finish(&state, 0, 0)
            .expect("summary");
        good["sides"][0]["chosen"] = json!([[4, 1]]);
        assert_eq!(
            validate(&good),
            Err(SummaryError::InvalidCatalog),
            "choice without offer"
        );
        good["sides"][0]["chosen"] = json!([]);
        good["sides"][0]["units"] = json!([[0, 1, null, 0, 1, 2, 0]]);
        assert_eq!(
            validate(&good),
            Err(SummaryError::InvalidCatalog),
            "tier1 > bought"
        );
    }
}
