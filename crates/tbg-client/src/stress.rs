//! `?stress=N`: N unit visuals walking the lane in the menu, for FPS checks
//! on phones without playing a match.

use bevy::prelude::*;
use tbg_core::UNIT_KINDS;

use crate::config;
use crate::visuals::{spawn_unit_visual, UnitLook, VisualAssets};

#[derive(Component)]
struct StressWalker;

pub fn plugin(app: &mut App) {
    // Startup, not OnEnter(Menu): the first state transition runs before
    // PreStartup, when `VisualAssets` does not exist yet.
    app.add_systems(Startup, spawn).add_systems(Update, walk);
}

fn spawn(mut commands: Commands, assets: Res<VisualAssets>) {
    let n: usize = config::url_param("stress")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    if n == 0 {
        return;
    }
    for i in 0..n {
        let kind = UNIT_KINDS[i % UNIT_KINDS.len()];
        let look = UnitLook {
            id: i as u32,
            kind,
            tier: 0,
            branch: 0,
            own: i % 2 == 0,
        };
        let pos = Vec3::new(
            0.5 + 15.0 * i as f32 / n as f32,
            0.0,
            [-0.8, 0.0, 0.8][i % 3],
        );
        let e = spawn_unit_visual(&mut commands, &assets, look, pos);
        commands.entity(e).insert(StressWalker);
    }
}

fn walk(time: Res<Time>, mut q: Query<&mut Transform, With<StressWalker>>) {
    for mut t in &mut q {
        t.translation.x += time.delta_secs();
        if t.translation.x > 15.5 {
            t.translation.x = 0.5;
        }
    }
}
