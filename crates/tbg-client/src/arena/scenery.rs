//! A single offshore terrain batch gives the arena a surrounding maritime world.

use super::{geometry::Surface, maritime, painted, trees, water, Part};
use crate::art::{mesh, mesh::Shape::*, palette::*};
use bevy::{light::NotShadowCaster, prelude::*};
use tbg_core::Pcg32;

struct Islet {
    at: Vec2,
    radius: Vec2,
    height: f32,
    seed: u64,
}

const ISLETS: [Islet; 2] = [
    Islet {
        at: Vec2::new(-4.0, -6.7),
        radius: Vec2::new(2.0, 1.5),
        height: 0.20,
        seed: 0xfa10,
    },
    Islet {
        at: Vec2::new(20.8, -7.6),
        radius: Vec2::new(2.35, 1.75),
        height: 0.95,
        seed: 0xc11f,
    },
];

fn island(def: &Islet) -> Mesh {
    const SIDES: usize = 14;
    let mut rng = Pcg32::new(def.seed, 13);
    let radii: [f32; SIDES] = std::array::from_fn(|_| 0.93 + rng.below(180) as f32 * 0.001);
    let levels = [
        (0.86, def.height),
        (1.02, def.height - 0.42),
        (1.06, water::LEVEL - 0.02),
        (0.82, water::LEVEL - 0.65),
    ];
    let point = |i: usize, band: usize| {
        let angle = i as f32 * std::f32::consts::TAU / SIDES as f32;
        Vec3::new(
            def.at.x + angle.cos() * def.radius.x * radii[i] * levels[band].0,
            levels[band].1,
            def.at.y + angle.sin() * def.radius.y * radii[i] * levels[band].0,
        )
    };
    let mut surface = Surface::default();
    for i in 0..SIDES {
        let j = (i + 1) % SIDES;
        let green = if i % 3 == 0 {
            Color::srgb(0.44, 0.61, 0.25)
        } else {
            Color::srgb(0.50, 0.66, 0.29)
        };
        surface.triangle(
            Vec3::new(def.at.x, def.height, def.at.y),
            point(j, 0),
            point(i, 0),
            green,
        );
        for band in 0..3 {
            let value = [0.66, 0.54, 0.44][band] + (i % 4) as f32 * 0.025;
            let color = Color::srgb(value * 0.96, value, value * 0.98);
            surface.quad(
                [
                    point(i, band),
                    point(j, band),
                    point(j, band + 1),
                    point(i, band + 1),
                ],
                color,
            );
        }
    }
    surface.finish()
}

fn dressing(parts: &mut Vec<Part>, def: &Islet, index: usize) {
    for (x, z, scale) in [
        (-0.48, -0.27, 0.47),
        (0.28, -0.30, 0.61),
        (0.57, 0.12, 0.42),
        (-0.42, 0.30, 0.36),
    ] {
        let at = Vec3::new(
            def.at.x + x * def.radius.x,
            def.height,
            def.at.y + z * def.radius.y,
        );
        trees::pine(parts, at, scale);
    }
    if index == 0 {
        maritime::lighthouse(
            parts,
            Vec3::new(def.at.x - 0.14, def.height, def.at.y + 0.10),
        );
        maritime::pier(parts, Vec3::new(def.at.x, -0.46, def.at.y + 1.35));
    }
    for i in 0..5 {
        let angle = (i as f32 + 0.25) * 1.2;
        parts.push((
            Ball(1.0),
            ROCK,
            Transform::from_xyz(
                def.at.x + angle.cos() * def.radius.x * 0.87,
                water::LEVEL + 0.18,
                def.at.y + angle.sin() * def.radius.y * 0.88,
            )
            .with_scale(Vec3::new(0.40, 0.64 + (i % 2) as f32 * 0.2, 0.44)),
        ));
    }
}

pub(super) fn spawn(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: &Handle<StandardMaterial>,
) -> usize {
    let mut parts = Vec::new();
    let mut chunks = Vec::new();
    for (index, def) in ISLETS.iter().enumerate() {
        chunks.push((island(def), Transform::default()));
        dressing(&mut parts, def, index);
    }
    for (x, z, scale) in [(-0.4, -8.5, 0.48), (17.2, -8.8, 0.60)] {
        parts.push((
            Ball(1.0),
            ROCK,
            Transform::from_xyz(x, water::LEVEL + 0.22, z).with_scale(Vec3::new(
                scale,
                scale * 1.7,
                scale * 0.8,
            )),
        ));
    }
    chunks.push((painted(parts), Transform::default()));
    let scenery = mesh::merge(chunks).unwrap_or_else(|| Cuboid::new(0.01, 0.01, 0.01).into());
    let triangles = mesh::triangles(&scenery);
    // Distant details never join the shadow pass; only the original arena casts shadows.
    commands.spawn((
        Name::new("offshore islets and landmarks"),
        Mesh3d(meshes.add(scenery)),
        MeshMaterial3d(material.clone()),
        Transform::default(),
        NotShadowCaster,
    ));
    let boat = maritime::spawn_boat(
        commands,
        meshes,
        material,
        Vec3::new(-2.2, water::LEVEL + 0.12, -6.2),
    );
    info!("arena offshore scenery: {triangles} triangles, boat: {boat} triangles; 2 draw batches");
    triangles + boat
}

pub(super) fn foam() -> Mesh {
    let mut surface = Surface::default();
    for def in &ISLETS {
        water::foam_ring(
            &mut surface,
            Vec3::new(def.at.x, water::LEVEL, def.at.y),
            def.radius,
        );
    }
    for (x, z, scale) in [(-0.4, -8.5, 0.48), (17.2, -8.8, 0.60)] {
        water::foam_ring(
            &mut surface,
            Vec3::new(x, water::LEVEL, z),
            Vec2::new(scale, scale * 0.8),
        );
    }
    waterfall(&mut surface);
    surface.finish()
}

fn waterfall(surface: &mut Surface) {
    let upper = Vec3::new(21.15, 0.98, -6.08);
    let lower = Vec3::new(21.15, water::LEVEL + 0.08, -5.48);
    for (offset, width, color) in [
        (0.0, 0.46, Color::srgb(0.27, 0.76, 0.85)),
        (-0.11, 0.085, Color::srgb(0.75, 0.94, 0.96)),
        (0.10, 0.06, Color::srgb(0.57, 0.86, 0.94)),
    ] {
        let shift = Vec3::new(offset, 0.0, if offset == 0.0 { 0.0 } else { 0.012 });
        let u = upper + shift;
        let l = lower + shift;
        surface.quad(
            [
                l - Vec3::X * width * 0.5,
                l + Vec3::X * width * 0.5,
                u + Vec3::X * width * 0.5,
                u - Vec3::X * width * 0.5,
            ],
            color,
        );
    }
    water::foam_ring(surface, lower, Vec2::new(0.46, 0.30));
}
