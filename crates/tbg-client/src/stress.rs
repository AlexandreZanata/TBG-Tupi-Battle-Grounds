//! `?stress=N`: N unit visuals walking the lane in the menu, for FPS checks
//! on phones without playing a match. `&gallery=<unit>` makes them all that
//! unit, cycling its five forms; each attacks a point ahead every 1.4 s.

use bevy::prelude::*;
use tbg_core::UNIT_KINDS;

use crate::art::{FigureLibrary, FORMS};
use crate::config;
use crate::visuals::{
    play_cue, spawn_unit_visual, AimTarget, UnitLook, UnitVisual, VisualAssets, VisualCue,
};

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
    let only = config::url_param("gallery").and_then(|m| {
        UNIT_KINDS
            .into_iter()
            .find(|k| format!("{k:?}").eq_ignore_ascii_case(&m))
    });
    for i in 0..n {
        let kind = only.unwrap_or(UNIT_KINDS[i % UNIT_KINDS.len()]);
        let (tier, branch) = if only.is_some() { FORMS[i % 5] } else { (0, 0) };
        let look = UnitLook {
            id: i as u32,
            kind,
            tier,
            branch,
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
    assets: Res<VisualAssets>,
    mut q: Query<(
        &mut Transform,
        &mut crate::art::anim::Animator,
        &mut UnitVisual,
        &mut StressWalker,
    )>,
) {
    for (mut t, mut animator, mut visual, mut walker) in &mut q {
        t.translation.x += time.delta_secs() * walker.speed;
        if t.translation.x > 15.5 {
            t.translation.x = 0.5;
        }
        if t.translation.x < 0.5 {
            t.translation.x = 15.5;
        }
        if time.elapsed_secs() >= walker.next_attack {
            let ahead = Vec3::new(walker.speed.signum() * 3.0, 0.6, 0.0);
            visual.engage(AimTarget::Point(t.translation + ahead));
            play_cue(&mut visual, &mut animator, &assets, VisualCue::AttackStart);
            walker.next_attack = time.elapsed_secs() + 1.4;
        }
    }
}
