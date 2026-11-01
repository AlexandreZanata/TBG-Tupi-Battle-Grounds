//! Low-poly mesh builders. Every part ends up flat-shaded with a vertex
//! color (darker toward its bottom: cheap fake ambient occlusion) and only
//! POSITION/NORMAL/COLOR, so parts merge into one mesh per joint.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, VertexAttributeValues};
use bevy::prelude::*;
use bevy::render::render_resource::PrimitiveTopology;

mod scenery;

/// A part's shape in its own space (Y up, centered unless noted).
#[derive(Clone, Copy, Debug)]
pub enum Shape {
    Box(f32, f32, f32),
    Ball(f32),
    /// Beveled scenery block: dimensions and edge width.
    Bevel(f32, f32, f32, f32),
    /// Irregular mineral silhouette and its fitted snow shell, sharing a seed.
    Rock(f32, u32),
    SnowCap(f32, u32),
    /// Faceted conifer crown with uneven, drooping lower edges.
    Canopy(f32, f32, u32),
    /// Radius, cylinder length (total height = length + 2 * radius).
    Pill(f32, f32),
    Cyl(f32, f32),
    Cone(f32, f32),
    /// Top radius, bottom radius, height.
    Frustum(f32, f32, f32),
    /// Tube radius, ring radius, swept angle in degrees (360 = full ring).
    Ring(f32, f32, f32),
    /// Revolved profile of (radius, y) points, bottom to top.
    Lathe(&'static [[f32; 2]]),
}

pub fn build(shape: Shape) -> Mesh {
    match shape {
        Shape::Box(x, y, z) => Cuboid::new(x, y, z).into(),
        Shape::Bevel(x, y, z, b) => scenery::bevel(Vec3::new(x, y, z), b),
        Shape::Rock(r, seed) => scenery::rock(r, seed, false),
        Shape::SnowCap(r, seed) => scenery::rock(r, seed, true),
        Shape::Canopy(r, h, seed) => scenery::canopy(r, h, seed),
        Shape::Ball(r) => Sphere::new(r)
            .mesh()
            .ico(1)
            .unwrap_or_else(|_| Sphere::new(r).into()),
        Shape::Pill(r, l) => Capsule3d::new(r, l)
            .mesh()
            .longitudes(8)
            .latitudes(4)
            .into(),
        Shape::Cyl(r, h) => Cylinder::new(r, h).mesh().resolution(8).into(),
        Shape::Cone(r, h) => Cone::new(r, h).mesh().resolution(8).into(),
        Shape::Frustum(top, bottom, h) => ConicalFrustum {
            radius_top: top,
            radius_bottom: bottom,
            height: h,
        }
        .mesh()
        .resolution(8)
        .into(),
        Shape::Ring(minor, major, deg) => Torus::new(major - minor, major + minor)
            .mesh()
            .minor_resolution(5)
            .major_resolution(12)
            .angle_range(0.0..=deg.to_radians())
            .into(),
        Shape::Lathe(profile) => lathe(profile, 8),
    }
}

/// Revolve a (radius, y) profile around Y.
fn lathe(profile: &[[f32; 2]], segments: u32) -> Mesh {
    let mut pos: Vec<[f32; 3]> = Vec::new();
    let ring = |[r, y]: [f32; 2], i: u32| {
        let a = i as f32 / segments as f32 * std::f32::consts::TAU;
        [r * a.cos(), y, r * a.sin()]
    };
    for w in profile.windows(2) {
        for i in 0..segments {
            let (a0, a1) = (ring(w[0], i), ring(w[0], i + 1));
            let (b0, b1) = (ring(w[1], i), ring(w[1], i + 1));
            pos.extend([a0, b0, b1, a0, b1, a1]);
        }
    }
    let mut m = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    m.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);
    m
}

/// Strip to positions, flat normals, and a vertex color with bottom shading.
pub fn paint(mut mesh: Mesh, color: Color) -> Mesh {
    if mesh.indices().is_some() {
        mesh.duplicate_vertices();
        mesh.remove_indices();
    }
    for attr in [
        Mesh::ATTRIBUTE_UV_0,
        Mesh::ATTRIBUTE_TANGENT,
        Mesh::ATTRIBUTE_NORMAL,
        Mesh::ATTRIBUTE_COLOR,
    ] {
        mesh.remove_attribute(attr);
    }
    mesh.compute_flat_normals();
    let ys: Vec<f32> = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(p)) => p.iter().map(|v| v[1]).collect(),
        _ => Vec::new(),
    };
    let (lo, hi) = ys
        .iter()
        .fold((f32::MAX, f32::MIN), |(lo, hi), y| (lo.min(*y), hi.max(*y)));
    let base = color.to_linear();
    let normals = match mesh.attribute(Mesh::ATTRIBUTE_NORMAL) {
        Some(VertexAttributeValues::Float32x3(n)) => n.clone(),
        _ => vec![[0.0, 1.0, 0.0]; ys.len()],
    };
    let sky = super::palette::AMBIENT_SKY.to_linear();
    let ground = super::palette::AMBIENT_GROUND.to_linear();
    let colors: Vec<[f32; 4]> = ys
        .iter()
        .zip(normals)
        .map(|(y, n)| {
            let h = (y - lo) / (hi - lo).max(1e-4);
            let k = if h < 0.33 {
                0.82
            } else if h > 0.67 {
                1.12
            } else {
                1.0
            };
            let a = if n[1] >= 0.0 { sky } else { ground };
            // Shader-free hemispherical fill and a subtle silhouette rim.
            let rim = 0.04 * (1.0 - n[1].abs());
            [
                base.red * k * (0.9 + 0.1 * a.red) + rim,
                base.green * k * (0.9 + 0.1 * a.green) + rim,
                base.blue * k * (0.9 + 0.1 * a.blue) + rim,
                1.0,
            ]
        })
        .collect();
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh
}

/// Merge already-painted parts placed by their transforms.
pub fn merge(parts: Vec<(Mesh, Transform)>) -> Option<Mesh> {
    let mut iter = parts.into_iter();
    let (first, t) = iter.next()?;
    let mut out = first.transformed_by(t);
    for (m, t) in iter {
        if out.merge(&m.transformed_by(t)).is_err() {
            warn!("mesh merge failed for a part; skipped");
        }
    }
    Some(out)
}

pub fn triangles(mesh: &Mesh) -> usize {
    match mesh.indices() {
        Some(Indices::U16(i)) => i.len() / 3,
        Some(Indices::U32(i)) => i.len() / 3,
        None => mesh.count_vertices() / 3,
    }
}
