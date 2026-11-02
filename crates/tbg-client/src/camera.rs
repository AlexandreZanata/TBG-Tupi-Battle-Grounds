//! Framing camera: perspective 35° FOV over the lane center.
//!
//! Landscape shows blue left/red right for every player and spectator; portrait
//! retains its existing vertical framing.
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
        #[cfg(target_os = "android")]
        Msaa::Off,
        #[cfg(target_os = "android")]
        bevy::render::view::NoIndirectDrawing,
        Tonemapping::AcesFitted,
        // Soft haze on the far rim of the diorama; the lane stays crisp.
        DistanceFog {
            color: crate::arena::theme::Theme::selected().palette().sky,
            falloff: crate::arena::theme::Theme::selected().fog(),
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
    state: Res<State<crate::GameState>>,
    time: Res<Time>,
) {
    let size = Vec2::new(window.width(), window.height());
    let resized = resize.read().count() > 0;
    let orbit = desktop(size.x, size.y)
        && *state.get() == crate::GameState::Menu
        && !crate::config::arena_preview()
        && crate::config::url_param("bench").is_none();
    if !resized && size == *last && !orbit && !state.is_changed() {
        return;
    }
    *last = size;
    let (mut transform, mut projection) = camera.into_inner();
    frame(&mut transform, &mut projection, size.x, size.y);
    if orbit {
        let angle = 8f32.to_radians() * (time.elapsed_secs() * std::f32::consts::TAU / 40.0).sin();
        let offset = transform.translation - LANE_CENTER;
        *transform =
            Transform::from_translation(LANE_CENTER + Quat::from_rotation_y(angle) * offset)
                .looking_at(LANE_CENTER, Vec3::Y);
    }
}

pub(crate) fn frame(transform: &mut Transform, projection: &mut Projection, w: f32, h: f32) {
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
    let dist = if desktop(w, h) && w / h >= 1.5 {
        (13.5 / tan_h).max(7.1 / tan_v)
    } else if portrait {
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
    // the -Z side, but that reverses the common blue-left/red-right orientation.
    // +Z gives both players and spectators the same authoritative view.
    let offset = if portrait {
        Vec3::new(-across, up, 0.0)
    } else {
        Vec3::new(0.0, up, across)
    };
    *transform = Transform::from_translation(LANE_CENTER + offset).looking_at(LANE_CENTER, Vec3::Y);
}

/// D-27 desktop shell breakpoint; the original phone/tablet framing stays intact.
pub fn desktop(width: f32, height: f32) -> bool {
    width >= 1280.0 && height >= 720.0 && width > height
}

#[cfg(test)]
mod side_tests {
    use super::*;

    #[test]
    fn side01_desktop_camera_keeps_blue_left_and_red_right() {
        for (width, height) in [(1280., 720.), (1440., 900.), (1920., 1080.), (2560., 1440.)] {
            let mut camera = Transform::default();
            let mut projection = Projection::Perspective(PerspectiveProjection::default());
            frame(&mut camera, &mut projection, width, height);
            assert_eq!(camera.translation.x, LANE_CENTER.x);
            assert!(camera.translation.z > 0.);
            assert!((camera.rotation * Vec3::X).dot(Vec3::X) > 0.99);
        }
    }
}
