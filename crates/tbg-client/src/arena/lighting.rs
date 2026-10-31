//! Warm upper-left sunlight with one focused shadow cascade and cool sky fill.

use super::theme::Theme;
use crate::{art::palette::SKY_TOP, config};
use bevy::{
    light::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
    prelude::*,
};

pub(super) fn spawn(commands: &mut Commands, theme: Theme) {
    let palette = theme.palette();
    // The Forest keeps the owner-approved seascape contrast; the themed
    // arenas keep the levels their scenes were approved with.
    let (lux, ambient) = match theme {
        Theme::Forest => (13_000.0, 950.0),
        // Storm Valley is the moody one: dimmer sun, darker fill.
        Theme::Storm => (9_500.0, 950.0),
        _ => (12_000.0, 1200.0),
    };
    // Always set: switching back to the Forest must restore its sky.
    commands.insert_resource(ClearColor(if theme == Theme::Forest {
        SKY_TOP
    } else {
        palette.sky
    }));
    commands.insert_resource(DirectionalLightShadowMap { size: 1024 });
    commands.spawn((
        super::ArenaScene,
        DirectionalLight {
            color: palette.key,
            illuminance: lux,
            shadow_maps_enabled: config::url_param("shadows").as_deref() != Some("0"),
            ..default()
        },
        Transform::from_xyz(-4.0, 18.0, 10.0).looking_at(Vec3::new(8.0, 0.0, 0.0), Vec3::Y),
        CascadeShadowConfigBuilder {
            num_cascades: 1,
            maximum_distance: 42.0,
            ..default()
        }
        .build(),
    ));
    commands.insert_resource(GlobalAmbientLight {
        color: palette.ambient,
        brightness: ambient,
        ..default()
    });
}
