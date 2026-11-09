//! Loads compressed model packs through the same asset source as the game.

use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext, LoadState},
    ecs::system::SystemParam,
    prelude::*,
};
use std::{collections::HashMap, io};

#[derive(Asset, TypePath)]
struct Pack(Vec<u8>);

#[derive(Default, TypePath)]
struct Loader;

impl AssetLoader for Loader {
    type Asset = Pack;
    type Settings = ();
    type Error = io::Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _: &(),
        _: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        Ok(Pack(bytes))
    }

    fn extensions(&self) -> &[&str] {
        &["tbgd"]
    }
}

#[derive(Default, Resource)]
struct Requests(HashMap<String, Handle<Pack>>);

pub(in crate::arena) fn plugin(app: &mut App) {
    app.init_asset::<Pack>()
        .init_asset_loader::<Loader>()
        .init_resource::<Requests>();
}

#[derive(SystemParam)]
pub(in crate::arena) struct Packs<'w> {
    server: Res<'w, AssetServer>,
    assets: Res<'w, Assets<Pack>>,
    requests: ResMut<'w, Requests>,
}

pub(in crate::arena) enum Status<'a> {
    Loading,
    Ready(&'a [u8]),
    Missing,
}

impl Packs<'_> {
    pub fn get(&mut self, path: String) -> Status<'_> {
        let handle = self
            .requests
            .0
            .entry(path)
            .or_insert_with_key(|path| self.server.load(path.clone()));
        if let Some(pack) = self.assets.get(handle.id()) {
            return Status::Ready(&pack.0);
        }
        if matches!(self.server.load_state(handle.id()), LoadState::Failed(_)) {
            Status::Missing
        } else {
            Status::Loading
        }
    }
}

#[cfg(test)]
pub(in crate::arena) fn seed(world: &mut World, path: String) {
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join(&path);
    let bytes = std::fs::read(source).expect("tracked Blender pack");
    let handle = world.resource_mut::<Assets<Pack>>().add(Pack(bytes));
    world.resource_mut::<Requests>().0.insert(path, handle);
}
