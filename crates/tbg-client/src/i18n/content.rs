//! Core content ids mapped to catalog messages.
//!
//! Every `match` here is exhaustive on purpose: a new `tbg-core` variant is a
//! compile error naming the catalog key to add. Steps 8-9 extend this to
//! units, forms, factions, archetypes, synergies, and augments.

use super::{Lang, Msg};

/// Localized rejection reason (the server sends the `Rejection` code only).
pub fn rejection(lang: Lang, reason: tbg_core::Rejection) -> &'static str {
    lang.t(match reason {
        tbg_core::Rejection::NotRunning => Msg::RejectionNotRunning,
        tbg_core::Rejection::BadOffer => Msg::RejectionBadOffer,
        tbg_core::Rejection::NotEnoughEnergy => Msg::RejectionNotEnoughEnergy,
        tbg_core::Rejection::NotYourUnit => Msg::RejectionNotYourUnit,
        tbg_core::Rejection::UnitGone => Msg::RejectionUnitGone,
        tbg_core::Rejection::MaxTier => Msg::RejectionMaxTier,
        tbg_core::Rejection::WrongBranch => Msg::RejectionWrongBranch,
        tbg_core::Rejection::NotEvolvable => Msg::RejectionNotEvolvable,
        tbg_core::Rejection::Stale => Msg::RejectionStale,
        tbg_core::Rejection::RoundClosed => Msg::RejectionRoundClosed,
        tbg_core::Rejection::BadOption => Msg::RejectionBadOption,
        tbg_core::Rejection::Unavailable => Msg::RejectionUnavailable,
        tbg_core::Rejection::Paused => Msg::RejectionPaused,
        tbg_core::Rejection::StaleOffer => Msg::RejectionStaleOffer,
        tbg_core::Rejection::ReserveFull => Msg::RejectionReserveFull,
        tbg_core::Rejection::AlreadyHeld => Msg::RejectionAlreadyHeld,
        tbg_core::Rejection::NoReserve => Msg::RejectionNoReserve,
        tbg_core::Rejection::StalePrice => Msg::RejectionStalePrice,
    })
}
