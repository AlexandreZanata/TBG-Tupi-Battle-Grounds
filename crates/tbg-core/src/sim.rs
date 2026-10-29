//! `MatchState::new` and the fixed tick, in the order fixed by
//! docs/phases/P2-core-simulation.md ("step() order").

use crate::content::{self, MATCH_END, OVERTIME_START, RUNNING_START};
use crate::rng::Pcg32;
use crate::ruleset::{Ruleset, CANON};
use crate::state::{EndReason, Event, MatchState, Outcome, Phase, PlayerState};
use crate::types::Seat;
use crate::{combat, shop, summon, synergy};

impl MatchState {
    pub fn new(seed: u64) -> MatchState {
        MatchState::with_rules(seed, &CANON)
    }

    /// A match on another ruleset: the offline balance lab only (D-33).
    pub fn with_rules(seed: u64, rules: &'static Ruleset) -> MatchState {
        let player = |stream: u64| {
            let mut rng = Pcg32::new(seed, stream);
            PlayerState {
                energy: rules.energy_start,
                offers: shop::initial_offers(&mut rng),
                stronghold_hp: rules.stronghold_hp,
                stronghold_cooldown: 0,
                synergy: [0; 4],
                last_seq: 0,
                rng,
            }
        };
        MatchState {
            tick: 0,
            phase: Phase::Countdown,
            players: [player(1), player(2)],
            units: Vec::new(),
            next_unit_id: 1,
            outcome: None,
            pending: Vec::new(),
            rules,
        }
    }

    /// Advance exactly one tick and return its events.
    pub fn step(&mut self) -> Vec<Event> {
        let mut ev = std::mem::take(&mut self.pending);
        if self.phase == Phase::Ended {
            return ev;
        }
        self.advance_phase(&mut ev);
        if self.phase != Phase::Ended {
            self.regen_energy();
            shop::rotate(self);
            synergy::update(self, &mut ev);
            combat::tick_timers(self);
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
        if self.tick == RUNNING_START {
            self.phase = Phase::Running;
            ev.push(Event::PhaseChanged {
                phase: Phase::Running,
            });
        } else if self.tick == OVERTIME_START || self.tick == MATCH_END {
            let [a, b] = [0, 1].map(|i| self.players[i].stronghold_hp);
            if a != b {
                let winner = if a > b { Seat::A } else { Seat::B };
                self.end(Some(winner), EndReason::TimeUp, ev);
            } else if self.tick == MATCH_END {
                self.end(None, EndReason::TimeUp, ev);
            } else {
                self.phase = Phase::Overtime;
                ev.push(Event::PhaseChanged {
                    phase: Phase::Overtime,
                });
            }
        }
    }

    fn regen_energy(&mut self) {
        let regen = match self.phase {
            Phase::Running => self.rules.regen_running,
            Phase::Overtime => self.rules.regen_overtime,
            _ => return,
        };
        let cap = self.rules.energy_cap;
        for p in &mut self.players {
            p.energy = (p.energy + regen).min(cap);
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

/// Ticks left in the current phase (0 once ended).
pub fn phase_ticks_left(state: &MatchState) -> u32 {
    match state.phase {
        Phase::Countdown => RUNNING_START - state.tick,
        Phase::Running => OVERTIME_START.saturating_sub(state.tick),
        Phase::Overtime => content::MATCH_END.saturating_sub(state.tick),
        Phase::Ended => 0,
    }
}
