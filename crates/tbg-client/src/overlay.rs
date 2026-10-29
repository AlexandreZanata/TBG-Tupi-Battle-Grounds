//! Diagnostics overlay: FPS, frame time, RTT, unit-visual count; with
//! `?measure` also frame-time percentiles, view gaps, the render timeline
//! and a motion probe (P30-C). Updated twice per second, placed below the
//! HUD's top bar.

use std::collections::{HashMap, VecDeque};

use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

use crate::sync::{ViewBuffer, ViewGaps};
use crate::visuals::UnitVisual;

/// Latest measured round-trip time in milliseconds (written by `net`).
#[derive(Resource, Default)]
pub struct OverlayStats {
    pub rtt_ms: Option<f64>,
    /// p50 tap-to-acknowledgement latency in ms (`None` before any ack).
    pub ack_ms: Option<f64>,
    /// Simulated latency from `?lag=MS` (0 when off).
    pub lag_ms: u32,
}

#[derive(Resource)]
struct RefreshTimer(Timer);

#[derive(Component)]
pub(crate) struct StatsText;

/// `?measure` only: recent frame times and what moving units did per frame.
#[derive(Resource, Default)]
struct Probe {
    frames_ms: VecDeque<f32>,
    /// Last world x and last per-frame step of each unit visual.
    units: HashMap<Entity, (f32, f32)>,
    moving: u32,
    stalls: u32,
    jumps: u32,
}

pub fn plugin(app: &mut App) {
    app.add_plugins(FrameTimeDiagnosticsPlugin::default())
        .init_resource::<OverlayStats>()
        .insert_resource(RefreshTimer(Timer::from_seconds(0.5, TimerMode::Repeating)))
        .add_systems(Startup, spawn_overlay)
        .add_systems(Update, update_overlay);
    if crate::config::url_param("measure").is_some() {
        app.init_resource::<Probe>()
            .add_systems(PostUpdate, probe_motion);
    }
}

fn spawn_overlay(mut commands: Commands) {
    commands.spawn((
        Text::new("FPS --"),
        TextFont {
            font_size: FontSize::Px(11.0),
            ..default()
        },
        TextColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
        Node {
            position_type: PositionType::Absolute,
            top: px(52.0),
            left: px(8.0),
            ..default()
        },
        StatsText,
        if crate::config::url_param("gallery").is_some() {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        },
    ));
}

/// A moving unit that does not move this frame is a stall; one that moves
/// three times its previous step is a jump (P30-C motion quality).
fn probe_motion(
    time: Res<Time>,
    mut probe: ResMut<Probe>,
    units: Query<(Entity, &Transform), With<UnitVisual>>,
) {
    let ms = time.delta_secs() * 1000.0;
    probe.frames_ms.push_back(ms);
    if probe.frames_ms.len() > 600 {
        probe.frames_ms.pop_front();
    }
    let mut seen = HashMap::with_capacity(probe.units.len());
    for (e, t) in &units {
        let x = t.translation.x;
        if let Some(&(last_x, last_step)) = probe.units.get(&e) {
            let step = (x - last_x).abs();
            if last_step > 0.002 {
                probe.moving += 1;
                probe.stalls += u32::from(step < 1e-5);
                probe.jumps += u32::from(step > 3.0 * last_step && step > 0.01);
            }
            seen.insert(e, (x, step));
        } else {
            seen.insert(e, (x, 0.0));
        }
    }
    probe.units = seen;
}

fn pct(sorted: &[f32], p: usize) -> f32 {
    sorted
        .get((sorted.len() * p / 100).min(sorted.len().saturating_sub(1)))
        .copied()
        .unwrap_or(0.0)
}

#[allow(clippy::too_many_arguments)] // one read-only system for every diagnostic
fn update_overlay(
    time: Res<Time>,
    mut timer: ResMut<RefreshTimer>,
    diagnostics: Res<DiagnosticsStore>,
    stats: Res<OverlayStats>,
    units: Query<(), With<UnitVisual>>,
    gaps: Res<ViewGaps>,
    buffer: Res<ViewBuffer>,
    probe: Option<Res<Probe>>,
    mut text: Single<&mut Text, With<StatsText>>,
    _all_entities: Query<Entity>,
    _meshes: Res<Assets<Mesh>>,
    _materials: Res<Assets<StandardMaterial>>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }
    #[cfg(target_arch = "wasm32")]
    if crate::config::url_param("measure").is_some() {
        if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
            if let Some(el) = doc.get_element_by_id("tbg-measure") {
                let _ =
                    el.set_attribute("data-entities", &_all_entities.iter().count().to_string());
                let _ = el.set_attribute("data-meshes", &_meshes.len().to_string());
                let _ = el.set_attribute("data-materials", &_materials.len().to_string());
            }
        }
    }
    let get = |d| diagnostics.get(d).and_then(|d| d.smoothed()).unwrap_or(0.0);
    // FRAME_TIME is already in milliseconds.
    let (fps, ms) = (
        get(&FrameTimeDiagnosticsPlugin::FPS),
        get(&FrameTimeDiagnosticsPlugin::FRAME_TIME),
    );
    let rtt = stats
        .rtt_ms
        .map_or_else(|| "offline".into(), |v| format!("{v:.0} ms"));
    let ack = stats
        .ack_ms
        .map_or_else(|| "--".into(), |v| format!("{v:.0} ms"));
    let mut g: Vec<f32> = gaps.0.iter().copied().collect();
    g.sort_by(f32::total_cmp);
    let view = if g.is_empty() {
        "--".into()
    } else {
        format!("{:.0}", pct(&g, 95))
    };
    let lag = if stats.lag_ms > 0 {
        format!(" | lag sim {} ms", stats.lag_ms)
    } else {
        String::new()
    };
    text.0 = format!(
        "FPS {fps:.0} | {ms:.1} ms | RTT {rtt} | ack {ack} | view p95 {view} ms | units {}{lag}",
        units.iter().count()
    );
    let Some(probe) = probe else { return };
    let mut f: Vec<f32> = probe.frames_ms.iter().copied().collect();
    f.sort_by(f32::total_cmp);
    let s = buffer.timeline.stats;
    let moving = probe.moving.max(1) as f32;
    let line = format!(
        "frame p50/p95/p99 {:.1}/{:.1}/{:.1} ms >33ms {} | view p99 {:.0} | delay {:.0} ms buf {} starve {} snap {} stale {} | motion stall {:.2}% jump {:.2}%",
        pct(&f, 50),
        pct(&f, 95),
        pct(&f, 99),
        f.iter().filter(|&&v| v > 33.3).count(),
        pct(&g, 99),
        s.delay_ms,
        s.depth,
        s.starved,
        s.snaps,
        s.stale,
        100.0 * probe.stalls as f32 / moving,
        100.0 * probe.jumps as f32 / moving,
    );
    text.0.push('\n');
    text.0.push_str(&line);
    export(&format!(
        "fps={fps:.1};rtt={rtt};ack={ack};units={};{line}",
        units.iter().count()
    ));
}

/// `?measure`: the same numbers on `<html data-tbg-measure>` for scripts.
fn export(line: &str) {
    #[cfg(target_arch = "wasm32")]
    if let Some(root) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.document_element())
    {
        let _ = root.set_attribute("data-tbg-measure", line);
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = line;
}
