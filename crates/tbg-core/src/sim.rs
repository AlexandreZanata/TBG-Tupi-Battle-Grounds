//! `MatchState::new` and the fixed tick. Matches have no gameplay clock:
//! they end only by Stronghold destruction, forfeit, or operational safety
//! outside the simulation.

use crate::arcana::ArcanaPreset;
use crate::augment::{self, AugmentState};
use crate::content::{BOUNTY_WINDOW_TICKS, RUNNING_START};
use crate::rng::Pcg32;
use crate::ruleset::{Ruleset, CANON};
use crate::state::{EndReason, Event, MatchState, Outcome, Phase, PlayerState};
use crate::types::Seat;
use crate::{combat, shop, summon, synergy};

impl MatchState {
    /// A match without Arcanas (tests and the lab's neutral baseline).
    pub fn new(seed: u64) -> MatchState {
        MatchState::with_rules(seed, &CANON)
    }

    /// The online match: each seat's frozen, validated preset (BR-AR05).
    pub fn with_arcanas(seed: u64, arcanas: [ArcanaPreset; 2]) -> MatchState {
        MatchState::lab(seed, &CANON, arcanas)
    }

    /// Practice preview (ROSTER-EXPANSION §F.5): the full staging pool,
    /// offline only. Identical to canon until P44 adds new kinds.
    pub fn with_preview(seed: u64, arcanas: [ArcanaPreset; 2]) -> MatchState {
        MatchState::lab(seed, Ruleset::preview(), arcanas)
    }

    /// A match on another ruleset: the offline balance lab only (D-33).
    pub fn with_rules(seed: u64, rules: &'static Ruleset) -> MatchState {
        MatchState::lab(seed, rules, [ArcanaPreset::NEUTRAL; 2])
    }

    /// Any ruleset and presets: the lab's Arcana experiments (D-33, D-40).
    pub fn lab(seed: u64, rules: &'static Ruleset, arcanas: [ArcanaPreset; 2]) -> MatchState {
        let player = |stream: u64| {
            let mut rng = Pcg32::new(seed, stream);
            PlayerState {
                energy: rules.energy_start,
                regen_frac: 0,
                offers: shop::draw_offers(&mut rng, rules.pool),
                stronghold_hp: rules.stronghold_hp,
                stronghold_cooldown: 0,
                synergy: [0; 4],
                last_seq: 0,
                rng,
                arcanas: ArcanaPreset::NEUTRAL,
                augments: Default::default(),
                augment_resolved: 0,
                augment_pick: None,
                bounty_window: 0,
                mend_window_start: 0,
                mend_in_window: 0,
                shield: 0,
                shield_until: 0,
                last_stand_used: false,
                fury_stacks: 0,
                fury_until: 0,
            }
        };
        let first_round = if rules.augment.round_ticks == 0 {
            0
        } else {
            RUNNING_START.saturating_add(rules.augment.round_ticks)
        };
        MatchState {
            tick: 0,
            clock: 0,
            phase: Phase::Countdown,
            players: [0, 1].map(|i| PlayerState {
                arcanas: arcanas[i],
                ..player(i as u64 + 1)
            }),
            units: Vec::new(),
            next_unit_id: 1,
            outcome: None,
            augments: AugmentState::new(seed, first_round),
            army: Default::default(),
            pending: Vec::new(),
            shrapnel: Vec::new(),
            rules,
            #[cfg(feature = "lab")]
            lab: crate::lab::LabStats::default(),
        }
    }

    /// Advance exactly one step and return its events. While an Augment
    /// selection is open the step only runs the selection clock: combat,
    /// Energy, Interest and every timer stay frozen (BR-AG12).
    pub fn step(&mut self) -> Vec<Event> {
        let mut ev = std::mem::take(&mut self.pending);
        if self.phase == Phase::Ended {
            return ev;
        }
        self.clock += 1;
        if self.augments.selecting() {
            augment::selection_step(self, &mut ev);
            return ev;
        }
        self.advance_phase(&mut ev);
        if self.phase != Phase::Ended {
            if self.is_live() {
                self.regen_energy();
                if self.tick > RUNNING_START
                    && (self.tick - RUNNING_START).is_multiple_of(BOUNTY_WINDOW_TICKS)
                {
                    augment::reset_windows(self);
                }
                augment::tick(self, &mut ev);
                self.expire_shields();
            }
            synergy::update(self, &mut ev);
            combat::tick_timers(self);
            crate::afflictions::tick(self, &mut ev);
            summon::expire(self);
            summon::hatch(self, &mut ev);
            combat::run_units(self, &mut ev);
            combat::stronghold_fire(self, &mut ev);
            combat::remove_dead(self, &mut ev);
            self.check_strongholds(&mut ev);
        }
        self.tick += 1;
        ev
    }

    /// The seat gives up (or its reconnect window expired): the other wins.
    pub fn forfeit(&mut self, seat: Seat) {
        if self.phase == Phase::Ended {
            return;
        }
        let mut ev = Vec::new();
        self.end(Some(seat.other()), EndReason::Forfeit, &mut ev);
        self.pending.extend(ev);
    }

    /// FNV-1a 64 over the serialized state (determinism checks).
    pub fn hash(&self) -> u64 {
        let bytes = postcard::to_allocvec(self).unwrap_or_default();
        bytes.iter().fold(0xcbf2_9ce4_8422_2325, |h, b| {
            (h ^ u64::from(*b)).wrapping_mul(0x0100_0000_01b3)
        })
    }

    fn advance_phase(&mut self, ev: &mut Vec<Event>) {
        if self.tick == RUNNING_START && self.phase == Phase::Countdown {
            self.phase = Phase::Running;
            ev.push(Event::PhaseChanged {
                phase: Phase::Running,
            });
        }
    }

    fn regen_energy(&mut self) {
        let regen = u64::from(self.rules.regen_running);
        let rule = self.rules.interest;
        let cap = self.rules.energy_cap;
        for p in &mut self.players {
            // Interest (BR-E05–E07): the more Energy is held, the faster it
            // grows; the remainder carries over so the rate stays exact.
            let scaled = regen * u64::from(rule.regen_bp(p.energy)) + u64::from(p.regen_frac);
            p.regen_frac = (scaled % 10_000) as u32;
            let gain = u32::try_from(scaled / 10_000).unwrap_or(u32::MAX);
            p.energy = p.energy.saturating_add(gain);
            if let Some(cap) = cap {
                p.energy = p.energy.min(cap);
            }
        }
    }

    fn expire_shields(&mut self) {
        let tick = self.tick;
        for p in &mut self.players {
            if p.shield_until <= tick {
                p.shield = 0;
            }
            if p.fury_until <= tick {
                p.fury_stacks = 0;
            }
        }
    }

    fn check_strongholds(&mut self, ev: &mut Vec<Event>) {
        let [a, b] = [0, 1].map(|i| self.players[i].stronghold_hp <= 0);
        let winner = match (a, b) {
            (false, false) => return,
            (true, true) => None,
            (true, false) => Some(Seat::B),
            (false, true) => Some(Seat::A),
        };
        self.end(winner, EndReason::StrongholdDestroyed, ev);
    }

    fn end(&mut self, winner: Option<Seat>, reason: EndReason, ev: &mut Vec<Event>) {
        self.outcome = Some(Outcome { winner, reason });
        self.phase = Phase::Ended;
        ev.push(Event::PhaseChanged {
            phase: Phase::Ended,
        });
    }
}

/// Ticks left in the current phase (0 once running or ended: no gameplay
/// clock). Countdown is the only bounded phase.
pub fn phase_ticks_left(state: &MatchState) -> u32 {
    match state.phase {
        Phase::Countdown => RUNNING_START.saturating_sub(state.tick),
        Phase::Running | Phase::Ended => 0,
    }
}
