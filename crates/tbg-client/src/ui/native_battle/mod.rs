//! Linux battle presentation, using the browser reference and authoritative views.

mod arena;
pub(super) mod chrome;
mod resources;
mod score;

pub(super) use resources::spawn as resources;
pub(super) use score::spawn as score;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    arena::plugin(app);
    score::plugin(app);
    resources::plugin(app);
}
