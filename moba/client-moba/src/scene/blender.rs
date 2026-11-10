//! Embedded vertex-colored pieces baked from the editable Tupi Valley collections.

use bevy::prelude::*;
use tbg_moba_core::{types::CampKind, Team};

use super::mesh::triangles;

macro_rules! asset {
    ($slug:literal) => {
        include_bytes!(concat!("../../assets/valley/", $slug, ".tbgpart"))
    };
}

fn piece(bytes: &[u8]) -> Mesh {
    // Blender writes local Bevy-space i16 coordinates (1/256 unit), then RGB8.
    let (vertices, tail) = bytes.as_chunks::<9>();
    assert!(tail.is_empty() && !vertices.is_empty() && vertices.len().is_multiple_of(3));
    triangles(
        vertices
            .iter()
            .map(|&[x0, x1, y0, y1, z0, z1, r, g, b]| {
                let axis = |lo, hi| f32::from(i16::from_le_bytes([lo, hi])) / 256.0;
                (
                    Vec3::new(axis(x0, x1), axis(y0, y1), axis(z0, z1)),
                    Color::srgb_u8(r, g, b),
                )
            })
            .collect(),
    )
}

pub fn bridge() -> Mesh {
    piece(asset!("bridge"))
}

pub fn plaza() -> Mesh {
    piece(asset!("plaza"))
}

pub fn plaza_crystal() -> Mesh {
    piece(asset!("plaza-crystal"))
}

pub fn nexus_plinth() -> Mesh {
    piece(asset!("nexus-plinth"))
}

pub fn camp(kind: CampKind) -> Mesh {
    piece(match kind {
        CampKind::Wolves => asset!("camp-wolves").as_slice(),
        CampKind::Golems => asset!("camp-golems").as_slice(),
        CampKind::Jaguar => asset!("camp-jaguar").as_slice(),
        CampKind::Frogs => asset!("camp-frogs").as_slice(),
    })
}

pub fn tower(team: Team, damage: u8) -> Mesh {
    let variants: [[&[u8]; 3]; 2] = [
        [
            asset!("tower-blue-0"),
            asset!("tower-blue-1"),
            asset!("tower-blue-2"),
        ],
        [
            asset!("tower-red-0"),
            asset!("tower-red-1"),
            asset!("tower-red-2"),
        ],
    ];
    piece(variants[team.index()][usize::from(damage.min(2))])
}

pub fn tower_crystal() -> Mesh {
    piece(asset!("tower-crystal"))
}
