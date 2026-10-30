//! The Classic tower's cloth flag (spawned by `life.rs`): it flies only on
//! a Classic Stronghold and falls with the crown at <= 66 % HP. The towers
//! themselves are built in `strongholds`.

use crate::{config, sync::LatestView};
use bevy::prelude::*;
use tbg_core::StrongholdKind;

#[derive(Component)]
pub(super) struct TowerPiece {
    pub own: bool,
    pub hide_below: f32,
}

/// Show each flag while its tower is Classic and above its HP threshold.
pub(super) fn damage_states(
    view: Res<LatestView>,
    mut pieces: Query<(&TowerPiece, &mut Visibility)>,
) {
    if config::url_param("gallery").is_some() {
        return;
    }
    // Ratios are of each kind's own max HP after Fortification.
    let state = view.0.as_ref().map(|v| {
        let me = v.you.index();
        [me, 1 - me].map(|i| {
            let s = &v.strongholds[i];
            let fraction = v.stronghold_hp[i] as f32 / s.max_hp.max(1) as f32;
            (s.kind == StrongholdKind::Classic, fraction)
        })
    });
    for (piece, mut vis) in &mut pieces {
        let (classic, fraction) = state.map_or(
            (true, 1.0),
            |[own, enemy]| if piece.own { own } else { enemy },
        );
        let want = if classic && fraction > piece.hide_below {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *vis != want {
            *vis = want;
        }
    }
}
