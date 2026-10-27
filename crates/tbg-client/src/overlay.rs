//! Diagnostics overlay: FPS, frame time, RTT, walker count.
//!
//! Updated twice per second. `?stress=N` keeps N walkers on the lane.

use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

use crate::{config, walkers};

/// Latest measured round-trip time in milliseconds (written by `net`).
#[derive(Resource, Default)]
pub struct OverlayStats {
    pub rtt_ms: Option<f64>,
}

#[derive(Resource, Default)]
struct StressTarget(usize);

#[derive(Resource)]
struct RefreshTimer(Timer);

#[derive(Component)]
struct StatsText;

pub fn plugin(app: &mut App) {
    app.add_plugins(FrameTimeDiagnosticsPlugin::default())
        .init_resource::<OverlayStats>()
        .init_resource::<StressTarget>()
        .insert_resource(RefreshTimer(Timer::from_seconds(
            0.5,
            TimerMode::Repeating,
        )))
        .add_systems(Startup, (spawn_overlay, spawn_stress))
        .add_systems(Update, (update_overlay, maintain_stress));
}

fn spawn_overlay(mut commands: Commands) {
    commands.spawn((
        Text::new("FPS -- | -- ms | RTT -- ms | walkers 0"),
        TextFont {
            font_size: FontSize::Px(18.0),
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: px(8.0),
            left: px(8.0),
            ..default()
        },
        StatsText,
    ));
}

fn spawn_stress(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut state: ResMut<walkers::SpawnState>,
    mut count: ResMut<walkers::WalkerCount>,
    mut target: ResMut<StressTarget>,
) {
    let n: usize = config::url_param("stress")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    if n == 0 {
        return;
    }
    target.0 = n;
    for i in 0..n {
        let x = 0.5 + 15.0 * i as f32 / n as f32;
        walkers::spawn_walker_at(&mut commands, &mut meshes, &mut materials, &mut state, &mut count, x);
    }
}

fn maintain_stress(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut state: ResMut<walkers::SpawnState>,
    mut count: ResMut<walkers::WalkerCount>,
    target: Res<StressTarget>,
) {
    if target.0 == 0 {
        return;
    }
    let mut missing = target.0.saturating_sub(count.0);
    while missing > 0 {
        walkers::spawn_walker_at(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut state,
            &mut count,
            0.5,
        );
        missing -= 1;
    }
}

fn update_overlay(
    time: Res<Time>,
    mut timer: ResMut<RefreshTimer>,
    diagnostics: Res<DiagnosticsStore>,
    stats: Res<OverlayStats>,
    count: Res<walkers::WalkerCount>,
    mut text: Single<&mut Text, With<StatsText>>,
) {
    timer.0.tick(time.delta());
    if !timer.0.just_finished() {
        return;
    }
    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0);
    let ms = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.0)
        * 1000.0;
    let rtt = stats
        .rtt_ms
        .map(|v| format!("{v:.0} ms"))
        .unwrap_or_else(|| "offline".into());
    text.0 = format!("FPS {fps:.0} | {ms:.1} ms | RTT {rtt} | walkers {}", count.0);
}
