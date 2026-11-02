//! Procedural low-poly training courtyard, built with the arena's painted mesh pipeline.

use crate::{
    arena::{painted, Part},
    art::mesh::Shape::*,
};
use bevy::{light::CascadeShadowConfigBuilder, prelude::*};

const STONE: Color = Color::srgb_u8(116, 125, 127);
const EDGE: Color = Color::srgb_u8(162, 158, 138);
const DARK: Color = Color::srgb_u8(63, 77, 82);
const GOLD: Color = Color::srgb_u8(184, 139, 61);
const BLUE: Color = Color::srgb_u8(24, 66, 87);

pub(super) const FLOOR: f32 = 0.40;
pub(super) const SKY: Color = Color::srgb_u8(35, 81, 106);

pub(super) fn spawn(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    mats: &mut Assets<StandardMaterial>,
) -> Entity {
    let stone = meshes.add(painted(courtyard()));
    let flame = meshes.add(painted(flames()));
    let material = mats.add(StandardMaterial {
        perceptual_roughness: 1.,
        ..default()
    });
    let glow = mats.add(StandardMaterial {
        unlit: true,
        ..default()
    });
    commands
        .spawn((
            Name::new("Laboratory courtyard"),
            Transform::from_xyz(8., 0., 0.),
            Visibility::Inherited,
        ))
        .with_children(|root| {
            root.spawn((
                Mesh3d(stone),
                MeshMaterial3d(material),
                Transform::default(),
            ));
            root.spawn((
                Mesh3d(flame),
                MeshMaterial3d(glow),
                Transform::default(),
                bevy::light::NotShadowCaster,
            ));
            root.spawn((
                DirectionalLight {
                    color: Color::srgb(1., 0.89, 0.72),
                    illuminance: 9_000.,
                    shadow_maps_enabled: !crate::config::load_flag("video.low")
                        && crate::config::url_param("shadows").as_deref() != Some("0"),
                    ..default()
                },
                Transform::from_xyz(-4., 8., 6.).looking_at(Vec3::Y, Vec3::Y),
                CascadeShadowConfigBuilder {
                    num_cascades: 1,
                    maximum_distance: 30.,
                    ..default()
                }
                .build(),
            ));
        })
        .id()
}
fn part(parts: &mut Vec<Part>, shape: crate::art::mesh::Shape, at: Vec3, color: Color) {
    parts.push((shape, color, Transform::from_translation(at)));
}
fn block(parts: &mut Vec<Part>, at: Vec3, size: Vec3, color: Color) {
    part(parts, Bevel(size.x, size.y, size.z, 0.035), at, color);
}
fn courtyard() -> Vec<Part> {
    let mut p = Vec::new();
    block(
        &mut p,
        Vec3::new(0., -0.43, -1.),
        Vec3::new(30., 0.7, 22.),
        DARK,
    );
    // Seams and slight tone changes use real stone slabs, as in the arena BattleLine.
    for x in -9..=9 {
        for z in -6..=4 {
            let tint = if (x + z) % 3 == 0 {
                STONE
            } else {
                Color::srgb_u8(102, 114, 118)
            };
            block(
                &mut p,
                Vec3::new(x as f32 * 1.42, -0.04, z as f32 * 1.42),
                Vec3::new(1.38, 0.12, 1.38),
                tint,
            );
        }
    }
    for (r, h, y, color) in [
        (2.2, 0.18, 0.08, DARK),
        (2.02, 0.18, 0.24, STONE),
        (1.85, 0.08, 0.36, EDGE),
    ] {
        part(&mut p, Cyl(r, h), Vec3::Y * y, color);
    }
    part(&mut p, Cyl(1.58, 0.012), Vec3::Y * 0.406, DARK);
    part(&mut p, Ring(0.025, 1.65, 360.), Vec3::Y * 0.415, GOLD);
    for i in 0..8 {
        let a = i as f32 * std::f32::consts::FRAC_PI_4;
        block(
            &mut p,
            Vec3::new(a.sin() * 1.79, 0.416, a.cos() * 1.79),
            Vec3::new(0.08, 0.015, 0.08),
            GOLD,
        );
    }
    // A wide portal frames rather than hides a tall champion's silhouette.
    for x in [-3.25, 3.25] {
        column(&mut p, Vec3::new(x, 0., -3.8), 4.9);
        banner(&mut p, Vec3::new(x, 0., -3.36));
        brazier(&mut p, Vec3::new(x * 0.70, 0., -1.45));
        column(&mut p, Vec3::new(x * 1.95, 0., -5.3), 3.1);
    }
    block(
        &mut p,
        Vec3::new(0., 4.85, -3.8),
        Vec3::new(7.5, 0.48, 1.05),
        EDGE,
    );
    block(
        &mut p,
        Vec3::new(0., 5.15, -3.8),
        Vec3::new(6.7, 0.14, 0.88),
        GOLD,
    );
    block(
        &mut p,
        Vec3::new(0., 1.3, -6.0),
        Vec3::new(6.4, 2.6, 0.5),
        DARK,
    );
    for x in [-1.9, 1.9] {
        column(&mut p, Vec3::new(x, 0., -5.7), 3.0);
    }
    block(
        &mut p,
        Vec3::new(0., 2.0, -5.65),
        Vec3::new(2.2, 2.5, 0.10),
        BLUE,
    );
    for x in [-1.08, 1.08] {
        block(
            &mut p,
            Vec3::new(x, 2.0, -5.57),
            Vec3::new(0.035, 2.5, 0.03),
            GOLD,
        );
    }
    p.push((
        Bevel(0.45, 0.45, 0.07, 0.03),
        GOLD,
        Transform::from_xyz(0., 2.85, -5.5)
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)),
    ));
    // Recessed walls and broken parapets give the courtyard depth without crowding the actor.
    for side in [-1., 1.] {
        for i in 0..4 {
            let x = side * (4.1 + i as f32 * 1.5);
            block(
                &mut p,
                Vec3::new(x, 1.05, -5.5),
                Vec3::new(1.47, 2.1, 0.6),
                DARK,
            );
            block(
                &mut p,
                Vec3::new(x, 2.16, -5.5),
                Vec3::new(1.54, 0.16, 0.8),
                EDGE,
            );
            shrub(&mut p, Vec3::new(x, 0., -3.4), i + 12);
        }
        for i in 0..6 {
            let x = side * (3.8 + (i % 2) as f32 * 0.6);
            part(
                &mut p,
                Rock(0.30 + i as f32 * 0.035, i + 4),
                Vec3::new(x, 0.16, 1.6 - i as f32 * 0.75),
                STONE,
            );
        }
    }
    // A distant hillside closes the horizon with the same faceted canopy vocabulary as Forest.
    for i in 0..13 {
        let x = i as f32 * 1.7 - 10.;
        part(
            &mut p,
            Rock(2.3, i + 40),
            Vec3::new(x, 0.1, -9.),
            Color::srgb_u8(63, 96, 96),
        );
        let variation = 0.75 + (i % 4) as f32 * 0.11;
        for (r, h, y) in [(0.85, 1.8, 1.7), (0.65, 1.5, 2.5)] {
            part(
                &mut p,
                Cone(r * variation, h * variation),
                Vec3::new(x, y * variation, -8.7 - (i % 3) as f32 * 0.7),
                Color::srgb_u8(43, 85, 69),
            );
        }
    }
    p
}
fn column(p: &mut Vec<Part>, at: Vec3, h: f32) {
    for (y, size, color) in [
        (0.15, Vec3::new(1.05, 0.3, 1.05), DARK),
        (0.40, Vec3::new(0.85, 0.2, 0.85), EDGE),
        (h * 0.5, Vec3::new(0.64, h - 0.8, 0.64), STONE),
        (h - 0.30, Vec3::new(0.85, 0.22, 0.85), GOLD),
        (h - 0.10, Vec3::new(1.05, 0.2, 1.05), EDGE),
    ] {
        block(p, at + Vec3::Y * y, size, color);
    }
}
fn banner(p: &mut Vec<Part>, at: Vec3) {
    block(p, at + Vec3::Y * 2.7, Vec3::new(0.75, 2.4, 0.04), BLUE);
    for x in [-0.35, 0.35] {
        block(
            p,
            at + Vec3::new(x, 2.7, 0.03),
            Vec3::new(0.025, 2.35, 0.025),
            GOLD,
        );
    }
    block(p, at + Vec3::Y * 3.95, Vec3::new(1., 0.07, 0.10), GOLD);
    p.push((
        Bevel(0.31, 0.31, 0.04, 0.02),
        GOLD,
        Transform::from_translation(at + Vec3::new(0., 3.25, 0.06))
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)),
    ));
}
fn brazier(p: &mut Vec<Part>, at: Vec3) {
    part(p, Frustum(0.24, 0.40, 0.65), at + Vec3::Y * 0.35, DARK);
    part(p, Frustum(0.46, 0.22, 0.24), at + Vec3::Y * 0.78, GOLD);
    part(
        p,
        Cyl(0.30, 0.05),
        at + Vec3::Y * 0.91,
        Color::srgb_u8(70, 45, 29),
    );
}
fn shrub(p: &mut Vec<Part>, at: Vec3, seed: u32) {
    for (i, dx) in [-0.25, 0., 0.28].into_iter().enumerate() {
        part(
            p,
            Rock(0.35, seed + i as u32),
            at + Vec3::new(dx, 0.24, 0.),
            Color::srgb_u8(70, 105, 62),
        );
    }
}
fn flames() -> Vec<Part> {
    let mut p = Vec::new();
    for x in [-2.275, 2.275] {
        part(
            &mut p,
            Cone(0.22, 0.52),
            Vec3::new(x, 1.15, -1.45),
            Color::srgb_u8(238, 93, 24),
        );
        part(
            &mut p,
            Cone(0.13, 0.35),
            Vec3::new(x, 1.13, -1.32),
            Color::srgb_u8(255, 201, 72),
        );
    }
    p
}
