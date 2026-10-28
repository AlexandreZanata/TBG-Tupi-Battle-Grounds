//! Framing camera: perspective 35° FOV over the lane center.
//!
//! Landscape shows the lane horizontal (own Stronghold left); portrait
//! rotates 90° around Y so the lane is vertical (own Stronghold bottom).
//! Distance is recomputed on every `WindowResized` so x in [-2, 18]
//! stays visible with ~10% margin.

use bevy::{core_pipeline::tonemapping::Tonemapping, prelude::*, window::WindowResized};

/// Lane center the camera looks at.
pub const LANE_CENTER: Vec3 = Vec3::new(8.0, 0.0, 0.0);
/// Vertical field of view, 35 degrees in radians.
pub const FOV_Y: f32 = 35.0 * std::f32::consts::PI / 180.0;
/// Camera elevation above the horizon (~55 degrees).
const ELEVATION: f32 = 55.0 * std::f32::consts::PI / 180.0;
/// Half of the x range [-2, 18] plus ~10% margin.
const NEED_LONG: f32 = 11.0;
/// Half of the across-lane extent plus margin.
const NEED_SHORT: f32 = 3.0;

#[derive(Component)]
pub struct ArenaCamera;

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_camera)
        .add_systems(Update, reframe_on_resize);
}

fn spawn_camera(mut commands: Commands, window: Single<&Window>) {
    let mut transform = Transform::default();
    let mut projection = Projection::Perspective(PerspectiveProjection {
        fov: FOV_Y,
        ..default()
    });
    frame(
        &mut transform,
        &mut projection,
        window.width(),
        window.height(),
    );
    // AcesFitted needs no LUT textures; the default (TonyMcMapface) needs the
    // `tonemapping_luts` feature, which we leave out to keep the WASM small.
    commands.spawn((
        Camera3d::default(),
        Tonemapping::AcesFitted,
        // Soft haze on the far rim of the diorama; the lane stays crisp.
        DistanceFog {
            color: crate::art::palette::SKY,
            falloff: FogFalloff::Linear {
                start: 38.0,
                end: 95.0,
            },
            ..default()
        },
        transform,
        projection,
        ArenaCamera,
    ));
}

/// Recompute framing whenever the window size changes (covers startup too).
fn reframe_on_resize(
    camera: Single<(&mut Transform, &mut Projection), With<ArenaCamera>>,
    window: Single<&Window>,
    mut resize: MessageReader<WindowResized>,
    mut last: Local<Vec2>,
) {
    let size = Vec2::new(window.width(), window.height());
    let resized = resize.read().count() > 0;
    if !resized && size == *last {
        return;
    }
    *last = size;
    let (mut transform, mut projection) = camera.into_inner();
    frame(&mut transform, &mut projection, size.x, size.y);
}

fn frame(transform: &mut Transform, projection: &mut Projection, w: f32, h: f32) {
    if let Projection::Perspective(p) = projection {
        p.fov = FOV_Y;
    }
    if w <= 0.0 || h <= 0.0 {
        *transform = Transform::from_xyz(8.0, 14.0, -10.0).looking_at(LANE_CENTER, Vec3::Y);
        return;
    }
    let portrait = h > w;
    let tan_v = (FOV_Y / 2.0).tan();
    let tan_h = tan_v * w / h;
    // Slant range so the long axis (lane) and short axis both fit.
    let dist = if portrait {
        (NEED_LONG / tan_v).max(NEED_SHORT / tan_h)
    } else {
        (NEED_LONG / tan_h).max(NEED_SHORT / tan_v)
    };
    let up = dist * ELEVATION.sin();
    let across = dist * ELEVATION.cos();
    // ponytail: closed-form fit ignores foreshortening along the view
    // direction; the 10% margin covers it. Upgrade: project lane corners.
    //
    // Deviation from spec text: the spec says the landscape camera sits on
    // the -Z side, but that mirrors the lane (own Stronghold on the right).
    // The camera sits on +Z so own is on the left per the acceptance check.
    let offset = if portrait {
        Vec3::new(-across, up, 0.0)
    } else {
        Vec3::new(0.0, up, across)
    };
    *transform = Transform::from_translation(LANE_CENTER + offset).looking_at(LANE_CENTER, Vec3::Y);
}
