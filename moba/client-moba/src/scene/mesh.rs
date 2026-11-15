//! Low-poly vertex-coloured meshes: primitives, flat shading, merging, noise.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{PrimitiveTopology, VertexAttributeValues};
use bevy::prelude::*;

#[derive(Clone, Copy, Debug)]
pub enum Shape {
    /// Read-only 1v1 scenery primitive.
    Shared(crate::art::mesh::Shape),
    Box(f32, f32, f32),
    Ball(f32),
    Cyl(f32, f32),
    Cone(f32, f32),
    Frustum(f32, f32, f32),
    /// Flat disc on the ground plane.
    Disc(f32),
}

pub type Part = (Shape, Color, Transform);

fn build(shape: Shape) -> Mesh {
    match shape {
        Shape::Shared(shape) => crate::art::mesh::build(shape),
        Shape::Box(x, y, z) => Cuboid::new(x, y, z).into(),
        Shape::Ball(r) => Sphere::new(r)
            .mesh()
            .ico(1)
            .unwrap_or_else(|_| Sphere::new(r).into()),
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
        Shape::Disc(r) => Circle::new(r)
            .mesh()
            .resolution(20)
            .build()
            .rotated_by(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    }
}

/// A non-indexed triangle list with per-vertex colours and flat normals.
pub fn triangles(vertices: Vec<(Vec3, Color)>) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    let positions: Vec<[f32; 3]> = vertices.iter().map(|(p, _)| p.to_array()).collect();
    let colors: Vec<[f32; 4]> = vertices
        .iter()
        .map(|(_, c)| c.to_linear().to_f32_array())
        .collect();
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.compute_flat_normals();
    mesh
}

/// Flat-shaded copy of `mesh` in one colour, lit a little from above.
pub fn paint(mut mesh: Mesh, color: Color) -> Mesh {
    if mesh.indices().is_some() {
        mesh.duplicate_vertices();
        mesh.remove_indices();
    }
    for attribute in [
        Mesh::ATTRIBUTE_UV_0,
        Mesh::ATTRIBUTE_TANGENT,
        Mesh::ATTRIBUTE_NORMAL,
        Mesh::ATTRIBUTE_COLOR,
    ] {
        mesh.remove_attribute(attribute);
    }
    mesh.compute_flat_normals();
    let base = color.to_linear();
    let colors: Vec<[f32; 4]> = match mesh.attribute(Mesh::ATTRIBUTE_NORMAL) {
        Some(VertexAttributeValues::Float32x3(normals)) => normals
            .iter()
            .map(|n| {
                // Baked occlusion: undersides darker, tops slightly brighter.
                let k = 0.82 + 0.22 * (n[1] * 0.5 + 0.5);
                [base.red * k, base.green * k, base.blue * k, base.alpha]
            })
            .collect(),
        _ => Vec::new(),
    };
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh
}

/// Paints and merges parts into one mesh (one draw call).
pub fn merge(parts: &[Part]) -> Mesh {
    let mut out: Option<Mesh> = None;
    for (shape, color, transform) in parts {
        let piece = paint(build(*shape), *color).transformed_by(*transform);
        match &mut out {
            None => out = Some(piece),
            Some(mesh) => {
                if mesh.merge(&piece).is_err() {
                    warn!("mesh merge skipped one part");
                }
            }
        }
    }
    out.unwrap_or_else(|| triangles(Vec::new()))
}

/// Deterministic 2D hash in `[0, 1)`.
pub fn hash(x: i32, y: i32, salt: u32) -> f32 {
    let mut h = (x as u32).wrapping_mul(0x8da6_b343)
        ^ (y as u32).wrapping_mul(0xd816_3841)
        ^ salt.wrapping_mul(0xcb1a_b31f);
    h ^= h >> 13;
    h = h.wrapping_mul(0x5bd1_e995);
    h ^= h >> 15;
    (h & 0x00ff_ffff) as f32 / 16_777_216.0
}

/// Smooth value noise in `[0, 1)` with feature size `scale`.
pub fn noise(x: f32, y: f32, scale: f32, salt: u32) -> f32 {
    let (fx, fy) = (x / scale, y / scale);
    let (ix, iy) = (fx.floor() as i32, fy.floor() as i32);
    let (tx, ty) = (fx - fx.floor(), fy - fy.floor());
    let (sx, sy) = (tx * tx * (3.0 - 2.0 * tx), ty * ty * (3.0 - 2.0 * ty));
    let a = hash(ix, iy, salt) + (hash(ix + 1, iy, salt) - hash(ix, iy, salt)) * sx;
    let b = hash(ix, iy + 1, salt) + (hash(ix + 1, iy + 1, salt) - hash(ix, iy + 1, salt)) * sx;
    a + (b - a) * sy
}

pub fn mix(a: Color, b: Color, t: f32) -> Color {
    let (a, b) = (a.to_linear(), b.to_linear());
    let t = t.clamp(0.0, 1.0);
    Color::LinearRgba(LinearRgba::new(
        a.red + (b.red - a.red) * t,
        a.green + (b.green - a.green) * t,
        a.blue + (b.blue - a.blue) * t,
        a.alpha + (b.alpha - a.alpha) * t,
    ))
}
