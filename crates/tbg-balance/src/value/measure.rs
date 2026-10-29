//! What one offline scenario produced, measured from state deltas and the
//! existing event stream. No production event or wire field is added.

use std::collections::BTreeMap;

use tbg_core::{Event, MatchState, Seat, Status, Target, UnitId};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SideMetrics {
    /// Effective HP removed from enemy units (overkill excluded).
    pub unit_damage: i64,
    /// Nominal `Hit` amounts above the HP actually removed.
    pub overkill: i64,
    /// Effective HP removed from the enemy Stronghold.
    pub stronghold_damage: i64,
    /// Effective healing received by this side's units.
    pub healing: i64,
    pub heal_events: u32,
    pub attacks: u32,
    pub stronghold_attacks: u32,
    pub deaths: u32,
    /// Ticks this side's units spent rooted or slowed (control suffered).
    pub root_ticks: u32,
    pub slow_ticks: u32,
    pub first_attack_tick: Option<u32>,
    pub first_stronghold_hit_tick: Option<u32>,
}

#[derive(Clone, Debug)]
pub struct Snapshot {
    pub tick: u32,
    /// `(id, seat, hp, rooted, slowed)`; ids are sorted.
    pub units: Vec<(UnitId, Seat, i32, bool, bool)>,
    pub strongholds: [i32; 2],
}

impl Snapshot {
    pub fn new(state: &MatchState) -> Snapshot {
        Snapshot {
            tick: state.tick,
            units: state
                .units
                .iter()
                .filter(|u| u.hp > 0)
                .map(|u| {
                    let rooted = matches!(u.status, Status::Rooted { .. });
                    let slowed = matches!(u.status, Status::Slowed { .. });
                    (u.id, u.seat, u.hp, rooted, slowed)
                })
                .collect(),
            strongholds: [0, 1].map(|i| state.players[i].stronghold_hp),
        }
    }

    fn unit(&self, id: UnitId) -> Option<&(UnitId, Seat, i32, bool, bool)> {
        self.units
            .binary_search_by_key(&id, |u| u.0)
            .ok()
            .map(|i| &self.units[i])
    }
}

#[derive(Clone, Debug, Default)]
pub struct Observer {
    pub metrics: [SideMetrics; 2],
}

impl Observer {
    pub fn observe(&mut self, before: &Snapshot, after: &Snapshot, events: &[Event]) {
        let mut nominal: BTreeMap<UnitId, i64> = BTreeMap::new();
        let mut effective: BTreeMap<UnitId, i64> = BTreeMap::new();

        for old in &before.units {
            let (id, seat, hp, _, _) = *old;
            let new_hp = after.unit(id).map_or(0, |u| u.2);
            if new_hp < hp {
                let dealt = i64::from(hp) - i64::from(new_hp);
                self.metrics[seat.other().index()].unit_damage += dealt;
                effective.insert(id, dealt);
            }
            if let Some(u) = after.unit(id) {
                if u.3 {
                    self.metrics[seat.index()].root_ticks += 1;
                }
                if u.4 {
                    self.metrics[seat.index()].slow_ticks += 1;
                }
            }
        }

        for (seat, hp) in before.strongholds.iter().enumerate() {
            let lost = i64::from(*hp) - i64::from(after.strongholds[seat]);
            if lost > 0 {
                self.metrics[1 - seat].stronghold_damage += lost;
            }
        }

        for event in events {
            match *event {
                Event::Hit {
                    to: Target::Unit(id),
                    amount,
                } => {
                    *nominal.entry(id).or_default() += i64::from(amount);
                }
                Event::Hit {
                    to: Target::Stronghold(seat),
                    ..
                } => {
                    let attacker = seat.other().index();
                    self.metrics[attacker]
                        .first_stronghold_hit_tick
                        .get_or_insert(before.tick);
                }
                Event::Heal { to, amount } => {
                    if let Some(u) = before.unit(to).or_else(|| after.unit(to)) {
                        self.metrics[u.1.index()].healing += i64::from(amount);
                        self.metrics[u.1.index()].heal_events += 1;
                    }
                }
                Event::AttackStart {
                    from: Target::Unit(id),
                    ..
                } => {
                    if let Some(u) = before.unit(id).or_else(|| after.unit(id)) {
                        let side = &mut self.metrics[u.1.index()];
                        side.attacks += 1;
                        side.first_attack_tick.get_or_insert(before.tick);
                    }
                }
                Event::AttackStart {
                    from: Target::Stronghold(seat),
                    ..
                } => {
                    self.metrics[seat.index()].stronghold_attacks += 1;
                }
                Event::Death { unit } => {
                    if let Some(u) = before.unit(unit) {
                        self.metrics[u.1.index()].deaths += 1;
                    }
                }
                _ => {}
            }
        }

        for old in &before.units {
            let (id, seat, _, _, _) = *old;
            let dead = effective.get(&id).copied().unwrap_or(0);
            let nominal = nominal.get(&id).copied().unwrap_or(0);
            let over = (nominal - dead).max(0);
            self.metrics[seat.other().index()].overkill += over;
        }
    }
}
