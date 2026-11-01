//! Chipped floating ice slabs with irregular bevels and broad pale top faces.

use crate::arena::{geometry::Surface, theme::rgb};
use bevy::prelude::*;

pub(super) fn mesh(sites: Vec<(Vec3, f32)>) -> Mesh {
    let mut surface = Surface::default();
    for (i, (at, r)) in sites.into_iter().enumerate() {
        let point = |j: usize, scale: f32, y: f32| {
            let a = j as f32 * std::f32::consts::TAU / 9.0 + i as f32 * 0.7;
            let uneven = 0.90 + (j as f32 * 2.4 + i as f32).sin() * 0.12;
            at + Vec3::new(
                a.cos() * r * uneven * scale * 1.2,
                y,
                a.sin() * r * uneven * scale * 0.78,
            )
        };
        for j in 0..9 {
            let k = (j + 1) % 9;
            let a = point(j, 0.90, 0.29);
            let b = point(k, 0.90, 0.29);
            let c = point(k, 1.0, 0.11);
            let d = point(j, 1.0, 0.11);
            surface.triangle(
                at + Vec3::Y * 0.29,
                b,
                a,
                if j % 3 == 0 {
                    rgb(0xe9f7fd)
                } else {
                    rgb(0xddecf5)
                },
            );
            surface.quad([a, b, c, d], rgb(0x90d7ed));
            surface.quad(
                [d, c, point(k, 0.96, -0.15), point(j, 0.96, -0.15)],
                if j % 2 == 0 {
                    rgb(0x53afd8)
                } else {
                    rgb(0x71c1e0)
                },
            );
        }
    }
    surface.finish()
}
