//! Permanent standard emoji ids, sourced from the attributed Twemoji 14.0.2 set.
use bevy::prelude::*;
pub fn handle(assets: &AssetServer, id: u8) -> Handle<Image> {
    assets.load(format!("emoji/standard/{}.png", id.min(15)))
}

#[derive(Resource)]
struct Preloaded {
    // HUD rebuilds may temporarily release every ImageNode. Keep these tiny
    // shared assets alive instead of unloading/reloading them between frames.
    _handles: [Handle<Image>; 16],
}
pub(super) fn preload(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(Preloaded {
        _handles: std::array::from_fn(|id| handle(&assets, id as u8)),
    });
}
