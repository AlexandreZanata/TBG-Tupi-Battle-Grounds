//! Which `Event`s a spectator may see (BR-LV03). The match is exhaustive on
//! purpose: a new variant must be classified here before it compiles.

use crate::state::Event;

/// True for every event a spectator may receive. Each variant is a combat,
/// board, or resolution fact that both seats already see in `MatchView`.
/// An Augment pick is only emitted when its round resolves (`AugmentChosen`,
/// `Forgone`), so it stays hidden until then (BR-AG14, BR-LV03).
pub fn is_public(event: &Event) -> bool {
    match event {
        Event::Spawn { .. }
        | Event::AttackStart { .. }
        | Event::Hit { .. }
        | Event::Heal { .. }
        | Event::StatusApplied { .. }
        | Event::Death { .. }
        | Event::Evolved { .. }
        | Event::Fortified { .. }
        | Event::SynergyChanged { .. }
        | Event::PhaseChanged { .. }
        | Event::RoundOpened { .. }
        | Event::AugmentChosen { .. }
        | Event::Forgone { .. }
        | Event::ShieldRaised { .. }
        | Event::SecondWind { .. }
        | Event::PoisonTick { .. }
        | Event::Betrayal { .. }
        | Event::Cleansed { .. }
        | Event::PlateBroken { .. }
        | Event::CrateOpened { .. }
        | Event::Transfer { .. }
        | Event::Boarded { .. }
        | Event::Dismounted { .. } => true,
    }
}
