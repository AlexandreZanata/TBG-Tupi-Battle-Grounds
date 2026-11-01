//! Soft vertex-alpha squalls outside the Storm Valley's playable island.

use bevy::{asset::RenderAssetUsages, prelude::*, render::render_resource::PrimitiveTopology};

pub(super) fn mesh() -> Mesh {
    let mut positions = Vec::new();
    let mut colors = Vec::new();
    for layer in 0..2 {
        for i in 0..11 {
            let phase = i as f32 * 2.3 + layer as f32 * 1.7;
            let center = Vec3::new(
                -8.0 + i as f32 * 3.1 + phase.sin() * 0.6,
                0.15 + layer as f32 * 0.30,
                -10.7 - layer as f32 * 3.0 - phase.cos() * 0.7,
            );
            let radius = Vec2::new(3.1 + phase.sin() * 0.45, 1.8);
            for band in 0..3 {
                for segment in 0..20 {
                    let vertex = |j: usize, ring: usize| {
                        let angle = j as f32 * std::f32::consts::TAU / 20.0;
                        let r = [0.0, 0.38, 0.73, 1.0][ring];
                        let scallop = 1.0 + (angle * 5.0 + phase).sin() * 0.07;
                        let at = center
                            + Vec3::new(
                                angle.cos() * radius.x * r * scallop,
                                (1.0 - r * r) * 0.35,
                                angle.sin() * radius.y * r * scallop,
                            );
                        let shade = 0.10 + r * 0.10 + angle.sin() * r * 0.025;
                        let color = Color::srgba(
                            shade * 0.72,
                            shade,
                            shade * 1.45,
                            [0.84, 0.73, 0.32, 0.0][ring],
                        )
                        .to_linear()
                        .to_f32_array();
                        (at.to_array(), color)
                    };
                    for (j, ring) in [
                        (segment, band),
                        (segment + 1, band),
                        (segment + 1, band + 1),
                        (segment, band),
                        (segment + 1, band + 1),
                        (segment, band + 1),
                    ] {
                        let (at, color) = vertex(j, ring);
                        positions.push(at);
                        colors.push(color);
                    }
                }
            }
        }
    }
    let normals = vec![[0.0, 1.0, 0.0]; positions.len()];
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
}
