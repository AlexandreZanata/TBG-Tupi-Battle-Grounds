//! Read-only adapters for the 1v1 scenery primitives, preserving their authored shape and colours.

use crate::art::mesh::{self, Shape};
use bevy::prelude::*;
pub type Part = (Shape, Color, Transform);
#[path = "../../../../crates/tbg-client/src/arena/rocks.rs"]
mod rocks;
#[path = "../../../../crates/tbg-client/src/arena/trees.rs"]
mod trees;

pub fn pine(parts: &mut Vec<Part>, at: Vec3, scale: f32) {
    trees::pine(parts, at, scale);
}
pub fn bush(parts: &mut Vec<Part>, at: Vec3, scale: f32) {
    trees::bush(parts, at, scale);
}
pub fn grass(parts: &mut Vec<Part>, at: Vec3, angle: f32) {
    trees::grass(parts, at, angle);
}
pub fn flowers(parts: &mut Vec<Part>, at: Vec3) {
    trees::flowers(parts, at);
}
pub fn boulder(parts: &mut Vec<Part>, at: Vec3, scale: f32, angle: f32) {
    rocks::boulder(parts, at, scale, angle);
}
pub fn merge(parts: Vec<Part>) -> Mesh {
    mesh::merge(
        parts
            .into_iter()
            .map(|(s, c, t)| (mesh::paint(mesh::build(s), c), t))
            .collect(),
    )
    .unwrap_or_else(|| super::mesh::triangles(Vec::new()))
}
