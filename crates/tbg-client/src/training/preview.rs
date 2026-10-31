//! One event-free catalog actor with bounded framing, drag rotation and camera restoration.

use super::Catalog;
use crate::{
    art::{
        anim::{Animator, Clip},
        FigureLibrary,
    },
    camera::ArenaCamera,
    ui::shell::{Panel, Shell},
    GameState,
};
use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
};

#[derive(Component)]
struct Model;
#[derive(Resource, Default)]
struct Preview {
    active: bool,
    key: Option<(tbg_core::UnitKind, u8, u8, bool)>,
    camera: Option<(Transform, Projection)>,
    size: (f32, f32),
    clock: f32,
    clip: Option<Clip>,
    revision: u32,
}
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Preview>()
        .add_systems(Update, drag.run_if(in_state(GameState::Menu)))
        .add_systems(
            Last,
            |state: Res<State<GameState>>, shell: Res<Shell>, mut clear: ResMut<ClearColor>| {
                if *state.get() == GameState::Menu && shell.panel == Panel::Troops {
                    clear.0 = Color::srgb_u8(14, 22, 29);
                }
            },
        )
        .add_systems(
            PostUpdate,
            show.before(crate::art::anim::animate)
                .before(bevy::transform::TransformSystems::Propagate),
        );
}
type ModelFilter = (With<Model>, Without<ArenaCamera>);

#[allow(clippy::too_many_arguments)]
fn show(
    mut commands: Commands,
    state: Res<State<GameState>>,
    shell: Res<Shell>,
    c: Res<Catalog>,
    time: Res<Time>,
    mut preview: ResMut<Preview>,
    lib: Res<FigureLibrary>,
    meshes: Res<Assets<Mesh>>,
    mut camera: Single<(&mut Transform, &mut Projection), With<ArenaCamera>>,
    window: Single<&Window>,
    mut clear: ResMut<ClearColor>,
    mut scenery: Query<&mut Visibility, With<crate::arena::ArenaScene>>,
    mut model: Query<(Entity, &mut Transform, &mut Animator), ModelFilter>,
    #[cfg(target_os = "linux")] mut demo: Query<
        &mut Visibility,
        (
            With<crate::sync::lobby_battle::DemoUnit>,
            Without<crate::arena::ArenaScene>,
        ),
    >,
) {
    let active = *state.get() == GameState::Menu && shell.panel == Panel::Troops;
    let changed = preview.active != active;
    preview.active = active;
    if active || changed {
        for mut visible in &mut scenery {
            *visible = if active {
                Visibility::Hidden
            } else {
                Visibility::Inherited
            };
        }
        if active {
            clear.0 = Color::srgb_u8(14, 22, 29);
        }
        #[cfg(target_os = "linux")]
        for mut visible in &mut demo {
            *visible = if active {
                Visibility::Hidden
            } else {
                Visibility::Inherited
            };
        }
    }
    if !active {
        if let Some((transform, projection)) = preview.camera.take() {
            *camera.0 = transform;
            *camera.1 = projection;
            if *state.get() != GameState::Menu {
                let (mut t, mut p) = camera.into_inner();
                crate::camera::frame(&mut t, &mut p, window.width(), window.height());
            }
        }
        if preview.key.take().is_some() {
            for (e, _, _) in &model {
                commands.entity(e).try_despawn();
            }
        }
        return;
    }
    let key = (c.selected, c.form.0, c.form.1, c.reduced);
    if preview.key != Some(key) {
        for (e, _, _) in &model {
            commands.entity(e).try_despawn();
        }
        preview.key = Some(key);
        preview.clock = 0.;
        preview.clip = Some(c.clip);
        let form = lib.get(c.selected, c.form.0, c.form.1);
        preview.size = crate::art::celebration::bounds::envelope(
            form, &meshes, c.selected, c.form.0, c.form.1,
        );
        let root = crate::art::presentation::spawn(
            &mut commands,
            &lib,
            c.selected,
            c.form,
            Transform::from_xyz(8., 0.12, 0.),
            0,
            c.reduced,
        );
        commands.entity(root).insert(Model);
    }
    if preview.camera.is_none() {
        preview.camera = Some((*camera.0, camera.1.clone()));
    }
    if preview.clip != Some(c.clip) || preview.revision != c.revision {
        preview.revision = c.revision;
        preview.clock = 0.;
        preview.clip = Some(c.clip);
    }
    if !c.reduced {
        preview.clock += time.delta_secs();
    }
    let (radius, height) = preview.size;
    let tan = (crate::camera::FOV_Y * 0.5).tan();
    let aspect = window.width() / window.height().max(1.);
    let distance = (radius / (tan * aspect * 0.28)).max(height / (tan * 0.70)) / c.zoom;
    let centre = Vec3::new(8., height * 0.5, 0.);
    let aim = centre - Vec3::Y * (distance * tan * 0.28);
    *camera.0 = Transform::from_translation(aim + Vec3::new(0., 0.14, 1.).normalize() * distance)
        .looking_at(aim, Vec3::Y);
    for (_, mut transform, mut a) in &mut model {
        transform.rotation = Quat::from_rotation_y(c.yaw);
        let duration = a.duration(c.clip);
        let t = if c.reduced {
            duration * 0.4
        } else if c.clip.looping() || c.clip == Clip::Victory {
            preview.clock
        } else {
            preview.clock.min(duration - 0.001)
        };
        a.present(c.clip, t);
    }
}
fn drag(
    mut motion: MessageReader<MouseMotion>,
    mut wheel: MessageReader<MouseWheel>,
    mouse: Res<ButtonInput<MouseButton>>,
    window: Single<&Window>,
    shell: Res<Shell>,
    mut c: ResMut<Catalog>,
) {
    let inside = shell.panel == Panel::Troops
        && window.cursor_position().is_some_and(|p| {
            p.x > window.width() * 0.31
                && p.x < window.width() * 0.65
                && p.y > window.height() * 0.19
                && p.y < window.height() * 0.66
        });
    let dx: f32 = motion.read().map(|m| m.delta.x).sum();
    let dy: f32 = wheel.read().map(|m| m.y).sum();
    if !inside {
        return;
    }
    if mouse.pressed(MouseButton::Left) && dx != 0. {
        c.yaw += dx * 0.008;
    }
    if dy != 0. {
        c.zoom = (c.zoom + dy * 0.05).clamp(0.6, 1.7);
    }
}
