//! Combat call sites for the local training laboratory (P62, D-80).
//!
//! Always compiled so the combat code needs no `cfg` at each site. Without
//! the `training` feature every function is a no-op or returns `false`.
//! With it, they only act while a `TrainingSession` installed its runtime:
//! normal matches keep identical state, events, RNG draws and outcomes.
//! Hooks observe actual HP mutations. They never change an amount.

use crate::state::{MatchState, Unit};
use crate::types::{Seat, UnitId};

/// What caused an HP change. Attribution is set by the call site that
/// knows it; anything else is recorded as `Unknown`, never guessed.
// Without the feature only the call sites name these.
#[cfg_attr(not(feature = "training"), allow(dead_code))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Effect {
    /// The primary hit or heal of an attack.
    Direct,
    /// Splash damage or splash healing around the primary target.
    Splash,
    /// Poison pulses. Their source is not yet tracked (unattributed).
    Periodic,
    /// A confused unit's hit on its own ally (Puppetmaster).
    Betrayal,
    /// Shatterback plate shrapnel. The plate owner is not recorded.
    Shrapnel,
    /// Healing returned by lifesteal.
    Lifesteal,
    /// Bloodweaver HP transfer: donor loss and receiver gain.
    Transfer,
    /// Blood Furnace self-burn on a resolving hit.
    SelfBurn,
    /// Faction Synergy regeneration.
    Regen,
    /// No call site claimed the change.
    Unknown,
}

#[cfg_attr(not(feature = "training"), allow(dead_code))]
impl Effect {
    pub const COUNT: usize = 10;
    pub const ALL: [Effect; Effect::COUNT] = [
        Effect::Direct,
        Effect::Splash,
        Effect::Periodic,
        Effect::Betrayal,
        Effect::Shrapnel,
        Effect::Lifesteal,
        Effect::Transfer,
        Effect::SelfBurn,
        Effect::Regen,
        Effect::Unknown,
    ];

    pub fn index(self) -> usize {
        self as usize
    }
}

/// Who caused an HP change.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Source {
    Unit(UnitId),
    /// A seat-wide effect with no unit (Synergy regeneration).
    Team(Seat),
    /// Known category, unknown source: reported as unattributed.
    Unknown,
}

/// Run `f` with `(source, effect)` as the attribution of every HP change
/// it causes; the previous attribution is restored afterwards (nesting:
/// a splash inside a hit).
#[inline]
pub(crate) fn scoped<R>(
    state: &mut MatchState,
    source: Source,
    effect: Effect,
    f: impl FnOnce(&mut MatchState) -> R,
) -> R {
    #[cfg(feature = "training")]
    {
        let Some(rt) = state.training.as_mut() else {
            return f(state);
        };
        let previous = rt.context.replace((source, effect));
        let out = f(state);
        if let Some(rt) = state.training.as_mut() {
            rt.context = previous;
        }
        out
    }
    #[cfg(not(feature = "training"))]
    {
        let _ = (source, effect);
        f(state)
    }
}

/// `victim` (its state before the change) lost `removed` HP; `shield` and
/// `plates` were absorbed before HP was touched.
#[inline]
pub(crate) fn lost(state: &mut MatchState, victim: &Unit, removed: i32, shield: i32, plates: i32) {
    #[cfg(feature = "training")]
    if let Some(rt) = state.training.as_mut() {
        let (source, effect) = rt.context.unwrap_or((Source::Unknown, Effect::Unknown));
        let source = source_unit(&state.units, source);
        if let Some(rt) = state.training.as_mut() {
            rt.meter
                .lost(victim, source, effect, removed, shield, plates);
        }
    }
    #[cfg(not(feature = "training"))]
    let _ = (state, victim, removed, shield, plates);
}

/// Like `lost`, for a call site that knows its own attribution.
#[inline]
pub(crate) fn lost_by(
    state: &mut MatchState,
    victim: &Unit,
    removed: i32,
    source: Source,
    effect: Effect,
) {
    scoped(state, source, effect, |s| lost(s, victim, removed, 0, 0));
}

/// `target` (its state before the change) actually regained `restored` HP
/// (overheal excluded).
#[inline]
pub(crate) fn restored(state: &mut MatchState, target: &Unit, restored: i32) {
    #[cfg(feature = "training")]
    if let Some(rt) = state.training.as_mut() {
        let (source, _) = rt.context.unwrap_or((Source::Unknown, Effect::Unknown));
        let source = source_unit(&state.units, source);
        if let Some(rt) = state.training.as_mut() {
            rt.meter.restored(target, source, restored);
        }
    }
    #[cfg(not(feature = "training"))]
    let _ = (state, target, restored);
}

/// A Stronghold lost `removed` HP to the current attribution.
#[inline]
pub(crate) fn lost_tower(state: &mut MatchState, seat: Seat, removed: i32) {
    #[cfg(feature = "training")]
    if let Some(rt) = state.training.as_mut() {
        let (source, _) = rt.context.unwrap_or((Source::Unknown, Effect::Unknown));
        let source = source_unit(&state.units, source);
        if let Some(rt) = state.training.as_mut() {
            rt.meter.lost_tower(seat, source, removed);
        }
    }
    #[cfg(not(feature = "training"))]
    let _ = (state, seat, removed);
}

/// Max-HP refresh changed `unit`'s HP by `delta` (not damage, not healing).
#[inline]
pub(crate) fn rescaled(state: &mut MatchState, unit: &Unit, delta: i32) {
    #[cfg(feature = "training")]
    if let Some(rt) = state.training.as_mut() {
        rt.meter.rescaled(unit, delta);
    }
    #[cfg(not(feature = "training"))]
    let _ = (state, unit, delta);
}

/// `unit` left with `hp` remaining but without damage (summon expiry,
/// a spent crate).
#[inline]
pub(crate) fn vanished(state: &mut MatchState, unit: &Unit) {
    #[cfg(feature = "training")]
    if let Some(rt) = state.training.as_mut() {
        rt.meter.vanished(unit);
    }
    #[cfg(not(feature = "training"))]
    let _ = (state, unit);
}

/// A training dummy: never acts, never opens, keeps its own max HP.
#[inline]
pub(crate) fn is_dummy(state: &MatchState, id: UnitId) -> bool {
    #[cfg(feature = "training")]
    {
        state
            .training
            .as_ref()
            .is_some_and(|rt| rt.dummies.contains(&id))
    }
    #[cfg(not(feature = "training"))]
    {
        let _ = (state, id);
        false
    }
}

/// The attributing unit's current copy, when it still exists.
#[cfg(feature = "training")]
fn source_unit(units: &[Unit], source: Source) -> crate::training::Attributed {
    use crate::training::Attributed;
    match source {
        Source::Unit(id) => units
            .binary_search_by_key(&id, |u| u.id)
            .ok()
            .map_or(Attributed::Gone(id), |i| Attributed::Unit(units[i])),
        Source::Team(seat) => Attributed::Team(seat),
        Source::Unknown => Attributed::Unknown,
    }
}
