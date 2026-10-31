//! Report actual GPU asset and pipeline preparation using Bevy's loading-screen pattern.

use bevy::{
    asset::AssetId,
    prelude::*,
    render::{
        mesh::RenderMesh, render_asset::RenderAssets, render_resource::PipelineCache,
        texture::GpuImage, ExtractSchedule, MainWorld, RenderApp,
    },
};
use std::collections::HashSet;

#[derive(Resource, Default)]
pub(super) struct Prepared {
    pub images: HashSet<AssetId<Image>>,
    pub meshes: HashSet<AssetId<Mesh>>,
    pub pipelines: bool,
    pub frames: u64,
    pub supported: bool,
}

impl Prepared {
    pub fn image_ready(&self, id: AssetId<Image>) -> bool {
        !self.supported || self.images.contains(&id)
    }
}

pub(super) fn plugin(app: &mut App) {
    let supported = app.get_sub_app(RenderApp).is_some();
    app.insert_resource(Prepared {
        supported,
        ..default()
    });
    if let Some(render) = app.get_sub_app_mut(RenderApp) {
        render.add_systems(ExtractSchedule, report);
    }
}

fn report(
    mut main: ResMut<MainWorld>,
    images: Res<RenderAssets<GpuImage>>,
    meshes: Res<RenderAssets<RenderMesh>>,
    pipelines: Res<PipelineCache>,
) {
    if let Some(mut ready) = main.get_resource_mut::<Prepared>() {
        ready.images = images.iter().map(|(id, _)| id).collect();
        ready.meshes = meshes.iter().map(|(id, _)| id).collect();
        ready.pipelines = pipelines.waiting_pipelines().count() == 0;
        ready.frames = ready.frames.wrapping_add(1);
    }
}
