//! Linux battle presentation, using the browser reference and authoritative views.

mod arena;
pub(super) mod chrome;
mod fortify;
mod resources;
mod score;

pub(super) use resources::spawn as resources;
pub(super) use score::spawn as score;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    arena::plugin(app);
    score::plugin(app);
    resources::plugin(app);
    fortify::plugin(app);
}

pub(super) fn auxiliary_visible(hud: &super::hud::Hud, view: &tbg_core::MatchView) -> Option<bool> {
    use super::hud::Hud;
    let me = view.you.index();
    Some(match hud {
        Hud::Augments => view.augments[me].iter().any(|copies| *copies > 0),
        Hud::Info => view.synergy[me].iter().any(|count| *count > 0),
        Hud::CouncilStatus => view.augment_round.is_some(),
        _ => return None,
    })
}
