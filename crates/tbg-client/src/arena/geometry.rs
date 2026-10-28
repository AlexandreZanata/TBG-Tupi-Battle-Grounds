//! Deterministic island outline and flat-shaded vertex-colored surfaces.

use bevy::{asset::RenderAssetUsages, prelude::*, render::render_resource::PrimitiveTopology};
use tbg_core::Pcg32;

pub(super) const SEGMENTS: usize = 64;

pub(super) fn outline() -> Vec<Vec3> {
    let mut rng = Pcg32::new(0x16a_c11ff, 7);
    (0..SEGMENTS)
        .map(|i| {
            let a = i as f32 * std::f32::consts::TAU / SEGMENTS as f32;
            let r = 0.97 + rng.below(700) as f32 / 10_000.0;
            Vec3::new(8.0 + 12.0 * a.cos() * r, -0.04, 5.7 * a.sin() * r)
        })
        .collect()
}

#[derive(Default)]
pub(super) struct Surface {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    colors: Vec<[f32; 4]>,
}

impl Surface {
    pub fn triangle(&mut self, a: Vec3, b: Vec3, c: Vec3, color: Color) {
        let n = (b - a).cross(c - a).normalize_or_zero().to_array();
        self.positions
            .extend([a.to_array(), b.to_array(), c.to_array()]);
        self.normals.extend([n; 3]);
        self.colors.extend([color.to_linear().to_f32_array(); 3]);
    }

    pub fn quad(&mut self, points: [Vec3; 4], color: Color) {
        let [a, b, c, d] = points;
        self.triangle(a, b, c, color);
        self.triangle(a, c, d, color);
    }

    pub fn finish(self) -> Mesh {
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, self.colors)
    }
}
