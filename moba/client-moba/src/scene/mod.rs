//! The static Tupi Valley diorama, generated once from `tbg_moba_core::map`.

use bevy::light::{CascadeShadowConfigBuilder, DirectionalLightShadowMap};
use bevy::prelude::*;

pub mod fog;
mod kit;
pub(crate) mod landmarks;
mod motion;
mod props;
use fog::{FogMask, FogMaterial, FogTexture};
pub mod mesh;
mod nature;
pub mod terrain;

/// World units (map space, y = north) → Bevy space (x right, -z north).
pub fn to_world(x: f32, y: f32, h: f32) -> Vec3 {
    Vec3::new(x - 100.0, h, 100.0 - y)
}

/// Bevy ground point → map units.
pub fn to_map(p: Vec3) -> Vec2 {
    Vec2::new(p.x + 100.0, 100.0 - p.z)
}

/// The one material every vertex-coloured mesh shares.
#[derive(Resource, Clone)]
pub struct Painted(pub Handle<FogMaterial>);

pub fn plugin(app: &mut App) {
    fog::plugin(app);
    motion::plugin(app);
    app.insert_resource(DirectionalLightShadowMap { size: 4096 })
        .add_systems(Startup, (setup_material, build).chain());
}

pub fn setup_material(
    mut commands: Commands,
    mut materials: ResMut<Assets<FogMaterial>>,
    fog: Res<FogTexture>,
) {
    commands.insert_resource(Painted(materials.add(FogMaterial {
        base: StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 0.92,
            reflectance: 0.18,
            ..default()
        },
        extension: FogMask {
            texture: fog.0.clone(),
        },
    })));
}

fn build(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<FogMaterial>>,
    fog: Res<FogTexture>,
    painted: Res<Painted>,
) {
    let paint = painted.0.clone();
    let mut batches = 0usize;
    let mut triangles = 0usize;
    let mut put = |mesh: Mesh| {
        batches += 1;
        triangles += crate::art::mesh::triangles(&mesh);
        commands
            .spawn((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(paint.clone()),
                Transform::default(),
            ))
            .id()
    };
    put(terrain::ground());
    put(terrain::lanes());
    for chunk in nature::forest() {
        put(chunk);
    }
    put(landmarks::bases());
    put(landmarks::plaza());
    put(landmarks::camps());
    put(landmarks::bridges());
    put(props::streets());
    put(props::rim());
    let waterfall = put(props::waterfalls());
    commands.entity(waterfall).insert(motion::WaterfallSurface);
    let details = nature::details();
    triangles += crate::art::mesh::triangles(&details);
    batches += 1;
    commands.spawn((
        Mesh3d(meshes.add(details)),
        MeshMaterial3d(paint.clone()),
        Transform::default(),
        bevy::light::NotShadowCaster,
    ));
    info!(batches,triangles,"P77 static mesh batches before culling; excludes water/crystal/flames, dynamic units and shadow passes");
    let water = materials.add(FogMaterial {
        base: StandardMaterial {
            base_color: Color::srgba(0.20, 0.62, 0.78, 0.86),
            perceptual_roughness: 0.12,
            reflectance: 0.55,
            alpha_mode: AlphaMode::Blend,
            ..default()
        },
        extension: FogMask {
            texture: fog.0.clone(),
        },
    });
    commands.spawn((
        Mesh3d(meshes.add(terrain::water())),
        motion::WaterSurface,
        bevy::light::NotShadowCaster,
        MeshMaterial3d(water),
        Transform::default(),
    ));
    let crystal = materials.add(FogMaterial {
        base: StandardMaterial {
            base_color: Color::srgb(0.55, 0.9, 1.0),
            emissive: LinearRgba::rgb(0.6, 2.2, 3.2),
            ..default()
        },
        extension: FogMask {
            texture: fog.0.clone(),
        },
    });
    commands.spawn((
        Mesh3d(meshes.add(landmarks::plaza_crystal())),
        MeshMaterial3d(crystal),
        Transform::default(),
    ));
    commands.spawn((
        PointLight {
            color: Color::srgb(0.5, 0.85, 1.0),
            intensity: 400_000.0,
            range: 18.0,
            ..default()
        },
        Transform::from_translation(to_world(100.0, 100.0, 5.0)),
    ));
    lights(&mut commands);
}

fn lights(commands: &mut Commands) {
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(1.0, 0.95, 0.86),
            illuminance: 14_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        // Low sun from the south-west: long readable shadows towards the north-east.
        Transform::from_xyz(-60.0, 90.0, 70.0).looking_at(Vec3::ZERO, Vec3::Y),
        CascadeShadowConfigBuilder {
            num_cascades: 3,
            first_cascade_far_bound: 70.0,
            maximum_distance: 320.0,
            ..default()
        }
        .build(),
    ));
    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgb(0.72, 0.80, 1.0),
        brightness: 850.0,
        ..default()
    });
}
