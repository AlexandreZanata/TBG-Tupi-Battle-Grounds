//! Shared Evolution marker geometry and stable form-aware overhead placement.
use bevy::{asset::RenderAssetUsages, prelude::*, render::render_resource::PrimitiveTopology};

pub fn star() -> Mesh {
    let mut positions = Vec::<[f32; 3]>::new();
    let point = |i: usize| {
        let angle = std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::PI / 5.0;
        let radius = if i.is_multiple_of(2) { 0.105 } else { 0.047 };
        [angle.cos() * radius, angle.sin() * radius, 0.0]
    };
    for i in 0..10 {
        positions.extend([[0.0; 3], point(i), point(i + 1)]);
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 0.0, 1.0]; 30])
}

pub fn marker_x(tier: u8, index: usize) -> f32 {
    if tier == 1 {
        0.0
    } else if index == 0 {
        -0.12
    } else {
        0.12
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn p71_evolution_markers_are_centered_and_do_not_touch() {
        assert_eq!(marker_x(1, 0), 0.0);
        assert!(marker_x(2, 1) - marker_x(2, 0) > 0.21);
        assert_eq!(star().count_vertices(), 30);
    }
}
