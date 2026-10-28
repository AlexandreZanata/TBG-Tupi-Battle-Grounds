//! `?gallery`: every unit (columns) in all 5 forms (rows) on the real
//! arena, cycling through every animation clip, for visual review and
//! screenshots. Logs triangles per form at startup (see `art`).

mod blockout;
mod palette;

use bevy::prelude::*;
use tbg_core::UNIT_KINDS;

use crate::art::anim::{Animator, Clip, CLIPS};
use crate::art::{FigureLibrary, FORMS};
use crate::camera::ArenaCamera;
use crate::config;
use crate::visuals::{spawn_unit_visual, UnitLook, VisualAssets};
use crate::GameState;

#[derive(Component)]
struct Cycle {
    offset: usize,
    shown: Option<usize>,
}

const CLIP_SECS: f64 = 1.4;

pub fn plugin(app: &mut App) {
    if config::url_param("gallery").is_some() {
        app.add_systems(Startup, |mut next: ResMut<NextState<GameState>>| {
            next.set(GameState::Gallery)
        })
        .add_systems(OnEnter(GameState::Gallery), spawn)
        .add_systems(
            Update,
            (cycle, close_up).run_if(in_state(GameState::Gallery)),
        );
    }
}

fn spawn(
    mut commands: Commands,
    assets: Res<VisualAssets>,
    lib: Res<FigureLibrary>,
    meshes: ResMut<Assets<Mesh>>,
    mats: ResMut<Assets<StandardMaterial>>,
    arena: Query<Entity, With<Mesh3d>>,
) {
    if config::url_param("gallery").as_deref() == Some("blockout") {
        if config::url_param("silhouette").is_some() {
            for entity in &arena {
                commands.entity(entity).insert(Visibility::Hidden);
            }
        }
        blockout::spawn(&mut commands, &lib, mats);
        return;
    }
    if config::url_param("gallery").as_deref() == Some("palette") {
        palette::spawn(&mut commands, meshes, mats);
        return;
    }
    for (col, kind) in UNIT_KINDS.into_iter().enumerate() {
        for (row, (tier, branch)) in FORMS.into_iter().enumerate() {
            info!(
                "gallery {kind:?} tier {tier} branch {branch}: {} triangles",
                lib.get(kind, tier, branch).triangles
            );
            let look = UnitLook {
                id: (col * 5 + row) as u32,
                kind,
                tier,
                branch,
                own: true,
            };
            let pos = Vec3::new(0.6 + col as f32 * 2.1, 0.0, -2.8 + row as f32 * 1.4);
            let e = spawn_unit_visual(&mut commands, &assets, &lib, look, pos);
            commands.entity(e).insert(Cycle {
                offset: col + row,
                shown: None,
            });
        }
    }
}

fn cycle(time: Res<Time>, mut figures: Query<(&mut Animator, &mut Cycle)>) {
    let step = (time.elapsed_secs_f64() / CLIP_SECS) as usize;
    for (mut anim, mut c) in &mut figures {
        let i = (step + c.offset) % CLIPS.len();
        if c.shown == Some(i) {
            continue;
        }
        c.shown = Some(i);
        match CLIPS[i] {
            base @ (Clip::Idle | Clip::Walk) => anim.set_base(base),
            clip => anim.play(clip),
        }
    }
}

/// `?gallery=N` frames unit column N (0..7) up close for review.
fn close_up(mut camera: Single<&mut Transform, With<ArenaCamera>>, window: Single<&Window>) {
    let mode = config::url_param("gallery").unwrap_or_default();
    if mode == "blockout" || mode == "palette" {
        let center = Vec3::new(8.0, 0.7, 0.0);
        let aspect = window.width() / window.height();
        let offset = if mode == "palette" {
            Vec3::new(0.0, 11.0, 14.0).normalize() * (6.0 / (0.315 * aspect)).max(18.0)
        } else {
            Vec3::new(0.0, 1.0, 1.0).normalize() * (9.0 / (0.315 * aspect)).max(20.0)
        };
        let offset = if mode == "blockout" && config::url_param("silhouette").is_some() {
            Vec3::Z * (9.0 / (0.315 * aspect)).max(20.0)
        } else {
            offset
        };
        **camera = Transform::from_translation(center + offset).looking_at(center, Vec3::Y);
        return;
    }
    let Some(col) = config::url_param("gallery").and_then(|v| v.parse::<f32>().ok()) else {
        return;
    };
    let x = 0.6 + col * 2.1;
    **camera =
        Transform::from_xyz(x + 1.2, 3.4, 8.8).looking_at(Vec3::new(x + 0.3, 0.5, -0.2), Vec3::Y);
}
