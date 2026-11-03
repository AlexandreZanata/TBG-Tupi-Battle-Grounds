//! One fixed simulation tick, split by responsibility.

mod combat;
mod movement;
mod outcome;
mod rewards;
pub(crate) mod spawn;
mod upkeep;

use crate::state::MobaMatch;

impl MobaMatch {
    /// Advances the match by one tick (1 / `TICK_HZ` s). Intentions for this
    /// tick must already be applied (see [`crate::host::tick`]).
    pub fn step(&mut self) {
        if self.outcome.is_some() {
            return;
        }
        self.tick += 1;
        upkeep::income(self);
        spawn::waves(self);
        spawn::camps(self);
        spawn::respawns(self);
        let positions = self.units.iter().map(|u| u.pos).collect::<Vec<_>>();
        self.grid.rebuild(positions.into_iter());
        let engaged = combat::fight(self);
        movement::advance(self, &engaged);
        movement::separate(self);
        upkeep::fountain(self);
        rewards::resolve_deaths(self);
        outcome::check(self);
    }
}
