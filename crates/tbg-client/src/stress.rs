//! `?stress=N`: N unit visuals walking the lane in the menu, for FPS checks
//! on phones without playing a match.

use bevy::prelude::*;
use tbg_core::{UnitKind, UNIT_KINDS};

use crate::art::FigureLibrary;
use crate::config;
use crate::visuals::{spawn_unit_visual, UnitLook, VisualAssets};

#[derive(Component)]
struct StressWalker {
    speed: f32,
    next_attack: f32,
}

pub fn plugin(app: &mut App) {
    // Startup, not OnEnter(Menu): the first state transition runs before
    // PreStartup, when `VisualAssets` does not exist yet.
    app.add_systems(Startup, spawn).add_systems(Update, walk);
}

fn spawn(mut commands: Commands, assets: Res<VisualAssets>, lib: Res<FigureLibrary>) {
    let n: usize = config::url_param("stress")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    if n == 0 {
        return;
    }
    for i in 0..n {
        let kind = if config::url_param("gallery").as_deref() == Some("archer") {
            UnitKind::Archer
        } else {
            UNIT_KINDS[i % UNIT_KINDS.len()]
        };
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
        let e = spawn_unit_visual(&mut commands, &assets, &lib, look, pos);
        commands.entity(e).insert(StressWalker {
            speed: tbg_core::content::effective_stats(kind, 0, 0, 0).speed as f32 / 1000.0
                * if i % 2 == 0 { 1.0 } else { -1.0 },
            next_attack: i as f32 * 0.11,
        });
    }
}

fn walk(
    time: Res<Time>,
    mut q: Query<(
        &mut Transform,
        &mut crate::art::anim::Animator,
        &mut StressWalker,
    )>,
) {
    for (mut t, mut animator, mut walker) in &mut q {
        t.translation.x += time.delta_secs() * walker.speed;
        if t.translation.x > 15.5 {
            t.translation.x = 0.5;
        }
        if t.translation.x < 0.5 {
            t.translation.x = 15.5;
        }
        if time.elapsed_secs() >= walker.next_attack {
            animator.play(crate::art::anim::Clip::AttackRanged);
            walker.next_attack = time.elapsed_secs() + 1.4;
        }
    }
}
