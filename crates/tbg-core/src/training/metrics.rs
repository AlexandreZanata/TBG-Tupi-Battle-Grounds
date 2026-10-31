//! Attributed integer meters for one trial (P62 §4.2, R09). Rows record
//! actual HP mutations reported by `training_hooks`, per unit instance,
//! dead instances included. Unknown sources stay visible as unattributed.

use crate::state::Unit;
use crate::types::{Seat, UnitId, UnitKind};

use super::Attributed;
use crate::training_hooks::Effect;

/// Retained instance rows per trial (GAME-DESIGN §16, **Proposed**, O02).
/// The session pauses at this count and asks for Reset.
pub const MAX_ROWS: usize = 4_096;
/// Rows still accepted after `MAX_ROWS` inside the tick that reached it,
/// so attribution is not lost before the session pauses.
const ROW_HEADROOM: usize = 512;

/// One unit instance's totals. Sums are HP points; times are active ticks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InstanceRow {
    pub id: UnitId,
    pub seat: Seat,
    pub kind: UnitKind,
    pub tier: u8,
    pub branch: u8,
    /// The parent for summons (Nester, Smuggler).
    pub summoner: Option<UnitId>,
    pub dummy: bool,
    /// HP when the row started.
    pub initial_hp: i32,
    /// HP removed from enemy units, by effect.
    pub dealt: [u64; Effect::COUNT],
    /// HP removed from own-seat units (betrayal, friendly blast, self-burn).
    pub dealt_friendly: [u64; Effect::COUNT],
    pub dealt_stronghold: u64,
    /// HP this instance lost, by effect.
    pub taken: [u64; Effect::COUNT],
    /// Part of `taken` with no known source.
    pub taken_unattributed: u64,
    /// Damage stopped before HP by a shield or by plates.
    pub shield_absorbed: u64,
    pub plates_absorbed: u64,
    /// HP this instance restored to allies and to itself (overheal excluded).
    pub healed_allies: u64,
    pub healed_self: u64,
    /// HP this instance regained from any source.
    pub restored: u64,
    /// HP moved by max-HP refreshes (Synergy changes): neither damage nor heal.
    pub rescaled: i64,
    /// HP left when the instance vanished without damage (expiry, spent crate).
    pub vanished: u64,
    pub spawned_at: u32,
    pub died_at: Option<u32>,
}

impl InstanceRow {
    fn new(u: &Unit, now: u32) -> InstanceRow {
        InstanceRow {
            id: u.id,
            seat: u.seat,
            kind: u.kind,
            tier: u.tier,
            branch: u.branch,
            summoner: u.summoner,
            dummy: false,
            initial_hp: u.hp,
            dealt: [0; Effect::COUNT],
            dealt_friendly: [0; Effect::COUNT],
            dealt_stronghold: 0,
            taken: [0; Effect::COUNT],
            taken_unattributed: 0,
            shield_absorbed: 0,
            plates_absorbed: 0,
            healed_allies: 0,
            healed_self: 0,
            restored: 0,
            rescaled: 0,
            vanished: 0,
            spawned_at: now,
            died_at: None,
        }
    }

    pub fn dealt_total(&self) -> u64 {
        self.dealt.iter().sum()
    }

    pub fn taken_total(&self) -> u64 {
        self.taken.iter().sum()
    }

    /// HP implied by the recorded mutations; equals the unit's actual HP
    /// (0 once it left the lane) when every mutation was reported.
    pub fn expected_hp(&self) -> i64 {
        i64::from(self.initial_hp) - self.taken_total() as i64
            + self.restored as i64
            + self.rescaled
            - self.vanished as i64
    }
}

/// Seat totals; summons count once, under their own rows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TeamTotals {
    pub dealt: u64,
    pub dealt_friendly: u64,
    pub dealt_stronghold: u64,
    pub taken: u64,
    pub healed: u64,
    /// Faction Synergy regeneration (no unit source).
    pub regen: u64,
    /// Damage this seat's units took from an unknown source.
    pub taken_unattributed: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TrainingMetrics {
    /// Sorted by id.
    rows: Vec<InstanceRow>,
    regen: [u64; 2],
    /// Healing received from an unknown source, by receiving seat.
    unattributed_heal: [u64; 2],
    /// Stronghold damage from an unknown source, by Stronghold seat.
    unattributed_tower: [u64; 2],
    /// Records dropped past the row headroom (should stay 0: the session
    /// pauses at `MAX_ROWS`).
    pub dropped: u64,
    /// Active tick stamped on new rows; maintained by the session.
    pub(crate) now: u32,
}

impl TrainingMetrics {
    pub fn rows(&self) -> &[InstanceRow] {
        &self.rows
    }

    pub fn row(&self, id: UnitId) -> Option<&InstanceRow> {
        self.rows
            .binary_search_by_key(&id, |r| r.id)
            .ok()
            .map(|i| &self.rows[i])
    }

    pub fn unattributed_heal(&self, seat: Seat) -> u64 {
        self.unattributed_heal[seat.index()]
    }

    pub fn unattributed_tower(&self, seat: Seat) -> u64 {
        self.unattributed_tower[seat.index()]
    }

    pub fn team(&self, seat: Seat) -> TeamTotals {
        let mut t = TeamTotals {
            regen: self.regen[seat.index()],
            ..TeamTotals::default()
        };
        for r in self.rows.iter().filter(|r| r.seat == seat) {
            t.dealt += r.dealt_total();
            t.dealt_friendly += r.dealt_friendly.iter().sum::<u64>();
            t.dealt_stronghold += r.dealt_stronghold;
            t.taken += r.taken_total();
            t.healed += r.healed_allies + r.healed_self;
            t.taken_unattributed += r.taken_unattributed;
        }
        t
    }

    /// Start a row for `u` (its HP before any recorded change) if absent.
    pub(crate) fn register(&mut self, u: &Unit, dummy: bool) {
        if let Some(r) = self.row_mut(u) {
            r.dummy |= dummy;
        }
    }

    pub(crate) fn died(&mut self, id: UnitId) {
        let now = self.now;
        if let Ok(i) = self.rows.binary_search_by_key(&id, |r| r.id) {
            self.rows[i].died_at.get_or_insert(now);
        }
    }

    pub(crate) fn lost(
        &mut self,
        victim: &Unit,
        source: Attributed<'_>,
        effect: Effect,
        removed: i32,
        shield: i32,
        plates: i32,
    ) {
        let removed = removed.max(0) as u64;
        let e = effect.index();
        // The victim's row first: it starts from the pre-change HP, even
        // when the victim is its own source (self-burn).
        let _ = self.row_mut(victim);
        let known = self.source_row(source).is_some();
        if let Some(r) = self.row_mut(victim) {
            r.taken[e] += removed;
            r.shield_absorbed += shield.max(0) as u64;
            r.plates_absorbed += plates.max(0) as u64;
            if !known {
                r.taken_unattributed += removed;
            }
        }
        if let Some(r) = self.source_row(source) {
            if r.seat == victim.seat {
                r.dealt_friendly[e] += removed;
            } else {
                r.dealt[e] += removed;
            }
        }
    }

    pub(crate) fn restored(&mut self, target: &Unit, source: Attributed<'_>, amount: i32) {
        if amount <= 0 {
            return;
        }
        let amount = amount as u64;
        if let Some(r) = self.row_mut(target) {
            r.restored += amount;
        }
        if let Attributed::Team(seat) = source {
            self.regen[seat.index()] += amount;
            return;
        }
        match self.source_row(source) {
            Some(r) if r.id == target.id => r.healed_self += amount,
            Some(r) => r.healed_allies += amount,
            None => self.unattributed_heal[target.seat.index()] += amount,
        }
    }

    pub(crate) fn lost_tower(&mut self, seat: Seat, source: Attributed<'_>, removed: i32) {
        let removed = removed.max(0) as u64;
        match self.source_row(source) {
            Some(r) => r.dealt_stronghold += removed,
            None => self.unattributed_tower[seat.index()] += removed,
        }
    }

    pub(crate) fn rescaled(&mut self, u: &Unit, delta: i32) {
        if let Some(r) = self.row_mut(u) {
            r.rescaled += i64::from(delta);
        }
    }

    pub(crate) fn vanished(&mut self, u: &Unit) {
        if let Some(r) = self.row_mut(u) {
            r.vanished += u.hp.max(0) as u64;
        }
    }

    /// The row of `u`, created from `u`'s current copy when absent.
    fn row_mut(&mut self, u: &Unit) -> Option<&mut InstanceRow> {
        match self.rows.binary_search_by_key(&u.id, |r| r.id) {
            Ok(i) => Some(&mut self.rows[i]),
            Err(i) if self.rows.len() < MAX_ROWS + ROW_HEADROOM => {
                self.rows.insert(i, InstanceRow::new(u, self.now));
                Some(&mut self.rows[i])
            }
            Err(_) => {
                self.dropped += 1;
                None
            }
        }
    }

    fn source_row(&mut self, source: Attributed<'_>) -> Option<&mut InstanceRow> {
        match source {
            Attributed::Unit(u) => self.row_mut(u),
            Attributed::Gone(id) => self
                .rows
                .binary_search_by_key(&id, |r| r.id)
                .ok()
                .map(|i| &mut self.rows[i]),
            Attributed::Team(_) | Attributed::Unknown => None,
        }
    }
}
