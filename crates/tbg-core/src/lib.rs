//! `tbg-core`: pure deterministic game domain for Tupi Battle Grounds.
//!
//! Phase 00 exposes the settled domain vocabulary only (see
//! `docs/game-design/GLOSSARY.md`). Simulation logic (energy, shop,
//! combat, evolution, synergies) arrives in Phase 03 and must stay free
//! of graphics, database, network, wall-clock, and unseeded randomness.

/// Mechanical role of a unit (rule BR-U02).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Archetype {
    Tank,
    Dps,
    Support,
    Ranged,
    Control,
}

/// Thematic group defining visual language and synergy rules (rule BR-Y01).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Faction {
    Humans,
    Undead,
    Machines,
    Plants,
}

/// Space a unit occupies on the `BattleLine`: one or two slots (rule BR-U03).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SlotCount {
    One,
    Two,
}

/// All playable archetypes, in glossary order.
pub const ARCHETYPES: [Archetype; 5] = [
    Archetype::Tank,
    Archetype::Dps,
    Archetype::Support,
    Archetype::Ranged,
    Archetype::Control,
];

/// All factions, in glossary order.
pub const FACTIONS: [Faction; 4] = [
    Faction::Humans,
    Faction::Undead,
    Faction::Machines,
    Faction::Plants,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glossary_vocabulary_is_complete() {
        assert_eq!(ARCHETYPES.len(), 5);
        assert_eq!(FACTIONS.len(), 4);
        assert!(matches!(SlotCount::One, SlotCount::One));
        assert!(matches!(SlotCount::Two, SlotCount::Two));
    }
}
