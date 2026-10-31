//! Linux battle presentation, using the browser reference and authoritative views.

mod arena;
pub(super) mod chrome;
mod resources;
mod score;

pub(super) use resources::spawn as resources;
pub(super) use score::spawn as score;

use crate::GameState;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    arena::plugin(app);
    app.add_systems(
        Update,
        (score::update, resources::update).run_if(in_state(GameState::InMatch)),
    );
}
