//! Shop portrait handles follow authoritative offers without changing purchase rules.

use super::hud::Hud;
use crate::sync::LatestView;
use bevy::prelude::*;

pub(super) fn update(
    view: Res<LatestView>,
    assets: Res<AssetServer>,
    mut portraits: Query<(&Hud, &mut ImageNode)>,
) {
    let Some(v) = &view.0 else { return };
    for (hud, mut image) in &mut portraits {
        if let Hud::Portrait(i) = hud {
            let slug = format!("{:?}", v.offers[*i]).to_ascii_lowercase();
            let handle = assets.load(format!("battle/portraits/{slug}.webp"));
            if image.image != handle {
                image.image = handle;
            }
        }
    }
}
