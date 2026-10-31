//! Pin packaged raster artwork once so navigation and randomized offers reuse ready handles.

use super::gpu::Prepared;
use bevy::{asset::LoadState, prelude::*};

include!(concat!(env!("OUT_DIR"), "/native_art.rs"));

#[derive(Resource, Default)]
pub(super) struct Cache(pub Vec<Handle<Image>>);

impl Cache {
    pub fn settled(&self, server: &AssetServer, gpu: &Prepared) -> bool {
        !self.0.is_empty()
            && self.0.iter().all(|image| {
                matches!(server.load_state(image.id()), LoadState::Failed(_))
                    || gpu.image_ready(image.id())
            })
    }
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Cache>().add_systems(Startup, preload);
}

fn preload(server: Res<AssetServer>, mut cache: ResMut<Cache>) {
    cache.0 = ART
        .iter()
        .map(|path| server.load((*path).to_owned()))
        .collect();
}
