//! Linux battle presentation, using the browser reference and authoritative views.

mod arena;
mod cards;
pub(super) mod chrome;
mod council;
mod fortify;
mod reserve;
mod resources;
mod score;
mod unit_panel;

pub(super) const SIDE_LEFT: f32 = 16.;
pub(super) const SHOP_BOTTOM: f32 = 36.;
pub(super) const SHOP_SCALE: f32 = 0.9;

pub(super) use cards::spawn as cards;
pub(super) use resources::spawn as resources;
pub(super) use score::spawn as score;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    arena::plugin(app);
    score::plugin(app);
    resources::plugin(app);
    fortify::plugin(app);
    cards::plugin(app);
    unit_panel::plugin(app);
    reserve::plugin(app);
    council::plugin(app);
}

pub(super) use council::CouncilControl;

pub(super) fn auxiliary_visible(hud: &super::hud::Hud, view: &tbg_core::MatchView) -> Option<bool> {
    use super::hud::Hud;
    let me = view.you.index();
    Some(match hud {
        Hud::Augments => view.augments[me].iter().any(|copies| *copies > 0),
        Hud::Info => view.synergy[me].iter().any(|count| *count > 0),
        _ => return None,
    })
}
