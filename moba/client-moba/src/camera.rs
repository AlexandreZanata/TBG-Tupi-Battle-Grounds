//! The RTS camera (pan, zoom, follow) and the top-down minimap camera.

use bevy::camera::visibility::RenderLayers;
use bevy::camera::{ScalingMode, Viewport};
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::*;
use tbg_moba_core::map::SIZE;

use crate::app::Screen;
use crate::input::PointerOverUi;
use crate::scene::to_world;
use crate::units::MINIMAP_LAYER;

const FOV: f32 = 32.0 * std::f32::consts::PI / 180.0;
const PITCH: f32 = 56.0 * std::f32::consts::PI / 180.0;
pub const MIN_DISTANCE: f32 = 32.0;
pub const MAX_DISTANCE: f32 = 290.0;
const EDGE: f32 = 6.0;

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct MinimapCamera;

/// Where the main camera looks (map units) and from how far.
#[derive(Resource)]
pub struct Rig {
    pub focus: Vec2,
    pub distance: f32,
}

impl Default for Rig {
    fn default() -> Self {
        Self {
            focus: Vec2::splat(SIZE as f32 / 2.0),
            distance: 262.0,
        }
    }
}

/// The minimap's square on screen, in logical pixels from the top-left.
#[derive(Resource, Clone, Copy, Default)]
pub struct MinimapRect {
    pub min: Vec2,
    pub size: f32,
}

impl MinimapRect {
    pub fn contains(&self, p: Vec2) -> bool {
        p.x >= self.min.x
            && p.y >= self.min.y
            && p.x <= self.min.x + self.size
            && p.y <= self.min.y + self.size
    }

    /// Screen point inside the minimap → map units.
    pub fn point_on_map(&self, p: Vec2) -> Vec2 {
        let t = (p - self.min) / self.size;
        // The ortho view frames the map with a 2-unit border on each side.
        Vec2::new(t.x, 1.0 - t.y) * (SIZE as f32 + 4.0) - Vec2::splat(2.0)
    }
}

pub fn plugin(app: &mut App) {
    app.init_resource::<Rig>()
        .init_resource::<MinimapRect>()
        .add_systems(Startup, spawn)
        .add_systems(
            Update,
            (
                orbit.run_if(in_state(Screen::Menu)),
                control.run_if(in_state(Screen::Match)),
                place,
            )
                .chain(),
        )
        .add_systems(Update, minimap_viewport)
        .add_systems(OnEnter(Screen::Match), |mut rig: ResMut<Rig>| {
            rig.distance = 95.0
        })
        .add_systems(OnEnter(Screen::Menu), |mut rig: ResMut<Rig>| {
            *rig = Rig::default()
        });
}

fn spawn(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: FOV,
            far: 900.0,
            ..default()
        }),
        // AcesFitted needs no LUT textures (no `tonemapping_luts` feature).
        Tonemapping::AcesFitted,
        Transform::default(),
        IsDefaultUiCamera,
        MainCamera,
        DistanceFog {
            color: crate::palette::SKY,
            falloff: FogFalloff::Linear {
                start: 260.,
                end: 700.,
            },
            ..default()
        },
    ));
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: 1,
            is_active: false,
            clear_color: ClearColorConfig::Custom(crate::palette::SKY),
            ..default()
        },
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::Fixed {
                width: SIZE as f32 + 4.0,
                height: SIZE as f32 + 4.0,
            },
            far: 1000.0,
            ..OrthographicProjection::default_3d()
        }),
        Tonemapping::AcesFitted,
        // North up: look straight down with -Z as the screen's up.
        Transform::from_translation(to_world(SIZE as f32 / 2.0, SIZE as f32 / 2.0, 400.0))
            .looking_at(
                to_world(SIZE as f32 / 2.0, SIZE as f32 / 2.0, 0.0),
                Vec3::NEG_Z,
            ),
        RenderLayers::from_layers(&[0, MINIMAP_LAYER]),
        MinimapCamera,
    ));
}

fn orbit(time: Res<Time>, mut rig: ResMut<Rig>) {
    let a = time.elapsed_secs() * 0.05;
    rig.focus = Vec2::splat(SIZE as f32 / 2.0) + Vec2::new(a.cos(), a.sin()) * 12.0;
}

#[allow(clippy::too_many_arguments)] // keyboard, mouse and window all steer the one rig
fn control(
    mut rig: ResMut<Rig>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    scroll: Res<AccumulatedMouseScroll>,
    motion: Res<AccumulatedMouseMotion>,
    window: Single<&Window>,
    over_ui: Res<PointerOverUi>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();
    let speed = rig.distance * 1.1;
    let mut pan = Vec2::ZERO;
    for (key, dir) in [
        (KeyCode::ArrowUp, Vec2::Y),
        (KeyCode::ArrowDown, Vec2::NEG_Y),
        (KeyCode::ArrowLeft, Vec2::NEG_X),
        (KeyCode::ArrowRight, Vec2::X),
    ] {
        if keys.pressed(key) {
            pan += dir;
        }
    }
    if let Some(cursor) = window.cursor_position().filter(|_| window.focused) {
        let size = window.size();
        if cursor.x < EDGE {
            pan.x -= 1.0;
        }
        if cursor.x > size.x - EDGE {
            pan.x += 1.0;
        }
        if cursor.y < EDGE {
            pan.y += 1.0;
        }
        if cursor.y > size.y - EDGE {
            pan.y -= 1.0;
        }
    }
    rig.focus += pan.normalize_or_zero() * speed * dt;
    if mouse.pressed(MouseButton::Middle) {
        let k = rig.distance / window.height().max(1.0) * 1.3;
        rig.focus += Vec2::new(-motion.delta.x, motion.delta.y) * k;
    }
    if !over_ui.0 && scroll.delta.y != 0.0 {
        rig.distance = (rig.distance * (1.0 - scroll.delta.y.signum() * 0.12))
            .clamp(MIN_DISTANCE, MAX_DISTANCE);
    }
    rig.focus = rig.focus.clamp(Vec2::splat(0.0), Vec2::splat(SIZE as f32));
}

fn place(rig: Res<Rig>, mut camera: Single<&mut Transform, With<MainCamera>>) {
    let target = to_world(rig.focus.x, rig.focus.y, 0.0);
    let offset = Vec3::new(0.0, PITCH.sin(), PITCH.cos()) * rig.distance;
    **camera = Transform::from_translation(target + offset).looking_at(target, Vec3::Y);
}

/// Keeps the minimap camera's viewport on the HUD frame (physical pixels).
fn minimap_viewport(
    rect: Res<MinimapRect>,
    window: Single<&Window>,
    screen: Res<State<Screen>>,
    mut camera: Single<&mut Camera, With<MinimapCamera>>,
) {
    let active = *screen.get() == Screen::Match && rect.size > 0.0;
    camera.is_active = active;
    if !active {
        return;
    }
    let scale = window.scale_factor();
    let position = (rect.min * scale).max(Vec2::ZERO).as_uvec2();
    let size = UVec2::splat((rect.size * scale) as u32).max(UVec2::ONE);
    let limit = window.physical_size();
    if position.x + size.x > limit.x || position.y + size.y > limit.y {
        camera.is_active = false;
        return;
    }
    let viewport = Viewport {
        physical_position: position,
        physical_size: size,
        ..default()
    };
    if camera
        .viewport
        .as_ref()
        .map(|v| (v.physical_position, v.physical_size))
        != Some((position, size))
    {
        camera.viewport = Some(viewport);
    }
}
