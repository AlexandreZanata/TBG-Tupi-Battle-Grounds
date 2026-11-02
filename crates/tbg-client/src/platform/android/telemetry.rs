//! Internal APK frame pacing and scene counts for the one authorized device observation.

use crate::{sync::LatestView, visuals::UnitVisual, GameState};
use bevy::prelude::*;

#[derive(Default)]
pub struct Samples {
    frames: Vec<f32>,
    since: f64,
    max_units: usize,
    max_visuals: usize,
    inactive: usize,
}

pub fn collect(
    time: Res<Time<Real>>,
    window: Single<&Window>,
    state: Res<State<GameState>>,
    view: Res<LatestView>,
    visuals: Query<(), With<UnitVisual>>,
    mut samples: Local<Samples>,
) {
    let dt = time.delta_secs() * 1000.;
    if window.focused && dt > 0. && dt < 1000. {
        samples.frames.push(dt);
        samples.max_units = samples
            .max_units
            .max(view.0.as_ref().map_or(0, |v| v.units.len()));
        samples.max_visuals = samples.max_visuals.max(visuals.iter().count());
    } else {
        samples.inactive += 1;
    }
    let now = time.elapsed_secs_f64();
    if now - samples.since < 60. {
        return;
    }
    samples.frames.sort_by(f32::total_cmp);
    let n = samples.frames.len();
    let total: f32 = samples.frames.iter().sum();
    let percentile = |p: usize| {
        samples
            .frames
            .get((n * p / 100).min(n.saturating_sub(1)))
            .copied()
            .unwrap_or(0.)
    };
    info!("TBG_ANDROID_MEASURE state={:?} elapsed_s={now:.1} samples={n} fps={:.1} frame_ms_p50={:.2} p95={:.2} p99={:.2} max_units={} max_visuals={} inactive_or_resume_frames={}", state.get(), if total > 0. { n as f32 * 1000. / total } else { 0. }, percentile(50), percentile(95), percentile(99), samples.max_units, samples.max_visuals, samples.inactive);
    *samples = Samples {
        since: now,
        ..default()
    };
}

/// Internal render-target census every ten seconds: windows, cameras and UI roots.
pub fn census(
    time: Res<Time<Real>>,
    windows: Query<(Entity, &Window, Has<bevy::window::PrimaryWindow>)>,
    cameras: Query<(Entity, &Camera, &bevy::camera::RenderTarget, Has<Camera3d>)>,
    nodes: Query<&ComputedNode>,
    roots: Query<&ComputedNode, (With<Node>, Without<ChildOf>)>,
    mut next: Local<f64>,
) {
    let now = time.elapsed_secs_f64();
    if now < *next {
        return;
    }
    *next = now + 10.;
    for (entity, window, primary) in &windows {
        info!(
            "TBG_ANDROID_CENSUS window {entity} primary={primary} physical={}x{} scale={}",
            window.physical_width(),
            window.physical_height(),
            window.scale_factor()
        );
    }
    for (entity, camera, target, is_3d) in &cameras {
        info!(
            "TBG_ANDROID_CENSUS camera {entity} 3d={is_3d} active={} order={} target={target:?} viewport={:?} size={:?}",
            camera.is_active,
            camera.order,
            camera.viewport.as_ref().map(|v| (v.physical_position, v.physical_size)),
            camera.physical_viewport_size()
        );
    }
    let sizes: Vec<_> = roots.iter().map(|n| n.size).take(4).collect();
    info!(
        "TBG_ANDROID_CENSUS ui_nodes={} roots={} root_sizes={sizes:?}",
        nodes.iter().count(),
        roots.iter().count()
    );
}
