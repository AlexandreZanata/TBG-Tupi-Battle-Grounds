//! Warm upper-left sunlight with one tight shadow cascade and cool sky fill.

use crate::{art::palette::*, config};
use bevy::{
    light::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
    prelude::*,
};

pub(super) fn spawn(commands: &mut Commands) {
    commands.insert_resource(DirectionalLightShadowMap { size: 1024 });
    commands.spawn((
        DirectionalLight {
            color: KEY,
            illuminance: 12_000.0,
            shadow_maps_enabled: config::url_param("shadows").as_deref() != Some("0"),
            ..default()
        },
        Transform::from_xyz(-4.0, 16.0, -5.0).looking_at(Vec3::new(8.0, 0.0, 0.0), Vec3::Y),
        CascadeShadowConfigBuilder {
            num_cascades: 1,
            maximum_distance: 60.0,
            ..default()
        }
        .build(),
    ));
    commands.insert_resource(GlobalAmbientLight {
        color: AMBIENT_SKY,
        brightness: 320.0,
        ..default()
    });
}
