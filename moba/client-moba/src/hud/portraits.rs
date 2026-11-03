//! Card and roster portraits rendered from the real 3D figures into textures,
//! so the HUD always matches the units on the field.

use bevy::camera::visibility::RenderLayers;
use bevy::camera::RenderTarget;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use tbg_moba_core::types::Side;
use tbg_moba_core::{Role, Team, TroopKind, UnitKind};

use crate::scene::Painted;
use crate::units::models;

/// First render layer used by portrait stages (one layer each).
const FIRST_LAYER: usize = 10;

#[derive(Resource, Clone)]
pub struct Portraits {
    pub troops: [Handle<Image>; 5],
    pub commanders: [Handle<Image>; 2],
}

pub fn plugin(app: &mut App) {
    app.add_systems(Startup, setup.after(crate::scene::setup_material));
}

fn target(images: &mut Assets<Image>, w: u32, h: u32) -> Handle<Image> {
    images.add(Image::new_target_texture(
        w,
        h,
        TextureFormat::Rgba8Unorm,
        Some(TextureFormat::Rgba8UnormSrgb),
    ))
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    painted: Res<Painted>,
) {
    let troops = TroopKind::ALL.map(|_| target(&mut images, 236, 200));
    let commanders = [target(&mut images, 96, 96), target(&mut images, 96, 96)];
    let mut stages: Vec<(UnitKind, Side, Handle<Image>, f32)> = TroopKind::ALL
        .into_iter()
        .zip(troops.iter().cloned())
        .map(|(t, image)| (UnitKind::Troop(t), Side::Team(Team::Blue), image, 1.0))
        .collect();
    for team in Team::ALL {
        stages.push((
            UnitKind::Commander(Role::Mid),
            Side::Team(team),
            commanders[team.index()].clone(),
            0.95,
        ));
    }
    let layers =
        RenderLayers::from_layers(&(FIRST_LAYER..FIRST_LAYER + stages.len()).collect::<Vec<_>>());
    commands.spawn((
        DirectionalLight {
            illuminance: 9_000.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(-2.0, 4.0, -5.0).looking_at(Vec3::ZERO, Vec3::Y),
        layers,
    ));
    for (k, (kind, side, image, zoom)) in stages.into_iter().enumerate() {
        let layer = RenderLayers::layer(FIRST_LAYER + k);
        // Stages sit far below the valley so no main camera ever frames them.
        let origin = Vec3::new(k as f32 * 40.0, -500.0, 0.0);
        let mesh = meshes.add(models::figure(kind, side));
        let height = if kind == UnitKind::Troop(TroopKind::Colossus) {
            2.4
        } else {
            1.9
        };
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(painted.0.clone()),
            Transform::from_translation(origin).with_rotation(Quat::from_rotation_y(-0.45)),
            layer.clone(),
        ));
        let look = origin + Vec3::Y * height * if zoom < 1.0 { 0.66 } else { 0.5 };
        let eye = look + Vec3::new(0.9, 0.45, -3.4) * height * 0.4 * zoom;
        commands.spawn((
            Camera3d::default(),
            Camera {
                order: -1 - k as isize,
                clear_color: ClearColorConfig::Custom(Color::srgb(0.13, 0.17, 0.24)),
                ..default()
            },
            RenderTarget::Image(image.into()),
            Tonemapping::AcesFitted,
            Transform::from_translation(eye).looking_at(look, Vec3::Y),
            layer,
        ));
    }
    commands.insert_resource(Portraits { troops, commanders });
}
