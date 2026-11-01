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
    stage: Option<Entity>,
    key: Option<(tbg_core::UnitKind, u8, u8, bool)>,
    camera: Option<(Transform, Projection, DistanceFog)>,
    size: (f32, f32),
    ambient: Option<GlobalAmbientLight>,
    theme: Option<crate::arena::theme::Theme>,
    clear: Option<Color>,
    clock: f32,
    clip: Option<Clip>,
    revision: u32,
}
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Preview>()
        .add_systems(Update, drag.run_if(in_state(GameState::Menu)))
        .add_systems(Last, backdrop)
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
    mut c: ResMut<Catalog>,
    time: Res<Time>,
    mut preview: ResMut<Preview>,
    lib: Res<FigureLibrary>,
    assets: (ResMut<Assets<Mesh>>, ResMut<Assets<StandardMaterial>>),
    arena: Res<crate::arena::ActiveArena>,
    mut ambient: ResMut<GlobalAmbientLight>,
    mut camera: Single<(&mut Transform, &mut Projection, &mut DistanceFog), With<ArenaCamera>>,
    window: Single<&Window>,
    mut clear: ResMut<ClearColor>,
    mut scenery: Query<&mut Visibility, With<crate::arena::ArenaScene>>,
    mut model: Query<(Entity, &mut Transform, &mut Animator), ModelFilter>,
    #[cfg(tbg_desktop)] mut demo: Query<
        &mut Visibility,
        (
            With<crate::sync::lobby_battle::DemoUnit>,
            Without<crate::arena::ArenaScene>,
        ),
    >,
) {
    let (mut meshes, mut materials) = assets;
    let same_theme = preview.theme.is_none_or(|theme| theme == arena.0);
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

        #[cfg(tbg_desktop)]
        for mut visible in &mut demo {
            *visible = if active {
                Visibility::Hidden
            } else {
                Visibility::Inherited
            };
        }
    }
    if !active {
        if changed {
            // Reduced motion is a temporary inspection aid, not the next visit's playback mode.
            c.reduced = false;
        }
        if let Some(stage) = preview.stage.take() {
            commands.entity(stage).try_despawn();
        }
        if let Some((transform, projection, fog)) = preview.camera.take() {
            *camera.0 = transform;
            *camera.1 = projection;
            if same_theme {
                *camera.2 = fog;
            }
            if *state.get() != GameState::Menu {
                let (mut t, mut p, _) = camera.into_inner();
                crate::camera::frame(&mut t, &mut p, window.width(), window.height());
            }
        }
        if preview.key.take().is_some() {
            for (e, _, _) in &model {
                commands.entity(e).try_despawn();
            }
        }
        if let Some(saved) = preview.ambient.take() {
            if same_theme {
                *ambient = saved;
            }
        }
        if let Some(saved) = preview.clear.take() {
            if same_theme {
                clear.0 = saved;
            }
        }
        return;
    }
    if preview.stage.is_none() {
        preview.ambient = Some(ambient.clone());
        preview.theme = Some(arena.0);
        preview.clear = Some(clear.0);
        preview.stage = Some(super::stage::spawn(
            &mut commands,
            &mut meshes,
            &mut materials,
        ));
    }
    clear.0 = super::stage::SKY;
    *ambient = GlobalAmbientLight {
        color: Color::srgb(0.66, 0.80, 0.92),
        brightness: 700.,
        ..default()
    };
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
            Transform::from_xyz(8., super::stage::FLOOR, 0.),
            0,
            c.reduced,
        );
        commands.entity(root).insert(Model);
    }
    if preview.camera.is_none() {
        preview.camera = Some((*camera.0, camera.1.clone(), camera.2.clone()));
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
    // Normalize only this event-free display actor; gameplay models retain their real scale.
    let scale = (4.0 / height.max(0.1)).min(2.2 / radius.max(0.1));
    let tan = (crate::camera::FOV_Y * 0.5).tan();
    let aspect = window.width() / window.height().max(1.);
    let distance = (10.8_f32).max(6.4 / aspect) / c.zoom;
    let aim = Vec3::new(8. + distance * tan * aspect * 0.025, 1.12, 0.);
    *camera.0 = Transform::from_translation(aim + Vec3::new(0., 0.22, 1.).normalize() * distance)
        .looking_at(aim, Vec3::Y);
    camera.2.color = super::stage::SKY;
    camera.2.falloff = FogFalloff::Linear {
        start: 19.,
        end: 48.,
    };
    for (_, mut transform, mut a) in &mut model {
        transform.scale = Vec3::splat(scale);
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
    viewport: Query<(&ComputedNode, &UiGlobalTransform), With<super::catalog::viewer::Viewport>>,
    mut c: ResMut<Catalog>,
) {
    let inside = shell.panel == Panel::Troops
        && window.cursor_position().is_some_and(|p| {
            viewport
                .iter()
                .any(|(node, t)| node.contains_point(*t, p * window.scale_factor()))
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

// Loading readiness also writes the sky during PostUpdate; keep the lab backdrop authoritative
// only while its presentation is active, without changing arena loading behavior.
fn backdrop(state: Res<State<GameState>>, shell: Res<Shell>, mut clear: ResMut<ClearColor>) {
    if *state.get() == GameState::Menu && shell.panel == Panel::Troops {
        clear.0 = super::stage::SKY;
    }
}
