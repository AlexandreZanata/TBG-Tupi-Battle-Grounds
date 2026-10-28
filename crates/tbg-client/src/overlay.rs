//! Diagnostics overlay: FPS, frame time, RTT, unit-visual count.
//! Updated twice per second, placed below the HUD's top bar.

use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

use crate::sync::ViewGaps;
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
struct StatsText;

pub fn plugin(app: &mut App) {
    app.add_plugins(FrameTimeDiagnosticsPlugin::default())
        .init_resource::<OverlayStats>()
        .insert_resource(RefreshTimer(Timer::from_seconds(0.5, TimerMode::Repeating)))
        .add_systems(Startup, spawn_overlay)
        .add_systems(Update, update_overlay);
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

fn update_overlay(
    time: Res<Time>,
    mut timer: ResMut<RefreshTimer>,
    diagnostics: Res<DiagnosticsStore>,
    stats: Res<OverlayStats>,
    units: Query<(), With<UnitVisual>>,
    gaps: Res<ViewGaps>,
    mut text: Single<&mut Text, With<StatsText>>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
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
    let view = g
        .get((g.len() * 95 / 100).min(g.len().saturating_sub(1)))
        .map_or_else(|| "--".into(), |v| format!("{v:.0}"));
    let lag = if stats.lag_ms > 0 {
        format!(" | lag sim {} ms", stats.lag_ms)
    } else {
        String::new()
    };
    text.0 = format!(
        "FPS {fps:.0} | {ms:.1} ms | RTT {rtt} | ack {ack} | view p95 {view} ms | units {}{lag}",
        units.iter().count()
    );
}
