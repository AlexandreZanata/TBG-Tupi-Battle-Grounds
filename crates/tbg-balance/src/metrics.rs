//! What one lab match produced (P16-C step 7), per participant rather than
//! per seat, so a pair of seat-swapped games compares like with like.

use tbg_core::content::{RUNNING_START, TICK_HZ};
use tbg_core::{EndReason, MatchState, Seat, ALL_KINDS};

pub const KINDS: usize = ALL_KINDS.len();
/// The comeback checkpoint: 2:00 of play (GAME-DESIGN §6).
pub const CHECKPOINT: u32 = RUNNING_START + 120 * TICK_HZ;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Side {
    pub purchases: [u16; KINDS],
    /// Evolutions per kind and branch (both tiers).
    pub evolutions: [[u16; 2]; KINDS],
    pub max_army: u16,
}

impl Side {
    /// Strict purchase-majority faction index, if any (ties are mixed).
    pub fn majority(&self) -> Option<usize> {
        let mut counts = [0u32; 4];
        for k in ALL_KINDS {
            counts[tbg_core::content::def(k).faction.index()] +=
                u32::from(self.purchases[k as usize]);
        }
        let top = *counts.iter().max()?;
        (top > 0 && counts.iter().filter(|&&c| c == top).count() == 1)
            .then(|| counts.iter().position(|&c| c == top))
            .flatten()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MatchResult {
    /// Participant index of the winner; `None` is a draw.
    pub winner: Option<usize>,
    pub reason: EndReason,
    pub ticks: u32,
    /// Stronghold HP per participant at 2:00 (if the match got there).
    pub hp_at_checkpoint: Option<[i32; 2]>,
    pub sides: [Side; 2],
    /// Participant playing seat A.
    pub seat_a: usize,
}

impl MatchResult {
    /// Win 1, draw ½, loss 0 for a participant.
    pub fn score(&self, participant: usize) -> f64 {
        match self.winner {
            Some(w) if w == participant => 1.0,
            Some(_) => 0.0,
            None => 0.5,
        }
    }

    /// The participant behind at 2:00 won.
    pub fn comeback(&self) -> Option<bool> {
        let hp = self.hp_at_checkpoint?;
        let w = self.winner?;
        (hp[0] != hp[1]).then_some(hp[w] < hp[1 - w])
    }
}

/// Participant index of a seat.
pub fn participant(seat: Seat, seat_a: usize) -> usize {
    match seat {
        Seat::A => seat_a,
        Seat::B => 1 - seat_a,
    }
}

/// Track army size and the checkpoint after each tick.
pub fn observe_tick(
    state: &MatchState,
    seat_a: usize,
    sides: &mut [Side; 2],
    hp: &mut Option<[i32; 2]>,
) {
    for seat in Seat::BOTH {
        let p = participant(seat, seat_a);
        let army = state.army_size(seat).min(usize::from(u16::MAX)) as u16;
        sides[p].max_army = sides[p].max_army.max(army);
    }
    if state.tick == CHECKPOINT {
        let mut v = [0; 2];
        for seat in Seat::BOTH {
            v[participant(seat, seat_a)] = state.player(seat).stronghold_hp;
        }
        *hp = Some(v);
    }
}
