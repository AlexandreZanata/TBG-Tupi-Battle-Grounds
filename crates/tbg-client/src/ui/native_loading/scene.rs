//! A nonblocking world backdrop while the current arena's meshes and pipelines are prepared.

use super::{assets::Cache, gpu::Prepared, Decoration, Status};
use bevy::prelude::*;

#[derive(Component)]
struct Veil;
#[derive(Component)]
struct Piece(Color);
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(super) struct Prepare;
#[derive(Resource)]
struct Reveal {
    frames: u8,
    alpha: f32,
    settled: bool,
}

pub(super) fn plugin(app: &mut App) {
    app.insert_resource(Reveal {
        frames: 0,
        alpha: 1.,
        settled: false,
    })
    .add_systems(Startup, spawn)
    .add_systems(
        PostUpdate,
        update
            .in_set(Prepare)
            .after(bevy::ui::UiSystems::Layout)
            .before(bevy::ui::UiSystems::Stack),
    );
}

fn spawn(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                ..default()
            },
            BackgroundColor(Color::srgb_u8(7, 15, 22)),
            Veil,
            Decoration,
            GlobalZIndex(-100),
        ))
        .with_children(|root| {
            for (x, w) in [(32., 14.), (54., 14.)] {
                for row in 0..3 {
                    let color = if x < 50. {
                        Color::srgb_u8(31, 62, 81)
                    } else {
                        Color::srgb_u8(72, 40, 45)
                    };
                    root.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: percent(x),
                            top: percent(39. + row as f32 * 5.),
                            width: percent(w),
                            height: px(13),
                            border_radius: BorderRadius::all(px(5)),
                            ..default()
                        },
                        BackgroundColor(color),
                        Piece(color),
                        Decoration,
                    ));
                }
            }
        });
}

#[allow(clippy::too_many_arguments)]
fn update(
    time: Res<Time>,
    server: Res<AssetServer>,
    cache: Res<Cache>,
    gpu: Res<Prepared>,
    arena: Res<crate::arena::ActiveArena>,
    meshes: Query<(&Mesh3d, &InheritedVisibility)>,
    mut reveal: ResMut<Reveal>,
    mut status: ResMut<Status>,
    mut veil: Query<(&mut BackgroundColor, &mut Visibility), With<Veil>>,
    mut pieces: Query<(&Piece, &mut BackgroundColor), Without<Veil>>,
    mut clear: ResMut<ClearColor>,
) {
    if arena.is_changed() {
        reveal.frames = 0;
        reveal.alpha = 1.;
        reveal.settled = false;
    }
    let scene_ready = !gpu.supported
        || (gpu.frames > 0
            && gpu.pipelines
            && meshes
                .iter()
                .filter(|(_, v)| v.get())
                .all(|(m, _)| gpu.meshes.contains(&m.0.id())));
    let ready = cache.settled(&server, &gpu) && scene_ready && !super::forced();
    reveal.frames = if ready {
        reveal.frames.saturating_add(1)
    } else {
        0
    };
    // Confirm several rendered frames, as in Bevy's loading_screen example.
    if reveal.frames >= 3 {
        reveal.settled = true;
    }
    // New troops/effects must not darken an already prepared arena mid-battle.
    let waiting = !reveal.settled;
    if !waiting {
        status.boot_ready = true;
        clear.0 = if crate::config::url_param("portrait").is_some() {
            Color::srgb_u8(20, 32, 42)
        } else {
            crate::art::palette::SKY_TOP
        };
    }
    status.scene_waiting = waiting;
    reveal.alpha = if waiting {
        (reveal.alpha + time.delta_secs() * 5.).min(1.)
    } else {
        (reveal.alpha - time.delta_secs() * 6.).max(0.)
    };
    for (mut color, mut visible) in &mut veil {
        color.0 = Color::srgba(0.027, 0.059, 0.086, reveal.alpha);
        *visible = if reveal.alpha > 0. && !crate::config::arena_preview() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    for (piece, mut color) in &mut pieces {
        color.0 = piece.0.with_alpha(reveal.alpha);
    }
}
