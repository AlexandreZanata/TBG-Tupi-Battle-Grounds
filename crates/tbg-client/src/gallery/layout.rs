//! Reflow review figures after the browser's first resize and orientation changes.

use bevy::prelude::*;
#[derive(Component)]
pub enum Layout {
    Blockout(usize),
    Form(usize),
    Frame(usize, usize),
    /// `&lineup`: (pose row, form column), the same in either orientation.
    Lineup(usize, usize),
}
impl Layout {
    pub fn position(&self, portrait: bool) -> Vec3 {
        match *self {
            Self::Blockout(i) if portrait => {
                Vec3::new(6.5 + (i % 2) as f32 * 3.0, 0.0, -3.0 + (i / 2) as f32 * 2.0)
            }
            Self::Blockout(i) => Vec3::new(0.6 + i as f32 * 2.1, 0.0, 0.0),
            Self::Form(i) => {
                let cols = if portrait { 2 } else { 5 };
                Vec3::new((i % cols) as f32 * 2.0, 0.0, -((i / cols) as f32) * 2.2)
            }
            Self::Lineup(row, col) => Vec3::new(col as f32 * 1.7, -(row as f32) * 2.0, 0.0),
            Self::Frame(i, quarter) => {
                let cols = if portrait { 1 } else { 2 };
                Vec3::new(
                    (i % cols * 4 + quarter) as f32 * 1.6,
                    -((i / cols) as f32) * 2.0,
                    0.0,
                )
            }
        }
    }
}
pub fn place(window: Single<&Window>, mut roots: Query<(&Layout, &mut Transform)>) {
    for (layout, mut t) in &mut roots {
        t.translation = layout.position(window.height() > window.width());
    }
}
