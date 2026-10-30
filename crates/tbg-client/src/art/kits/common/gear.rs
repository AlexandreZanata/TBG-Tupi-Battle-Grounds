//! Shared weapons and props for kit champions. Hand-held weapons sit at a
//! hand socket's origin (the fist): blades and hafts run along +Z (forward
//! when the arm hangs), staffs and poles along +Y. Each returns parts so a
//! kit can recolor or extend them per form.

use bevy::prelude::*;

use crate::art::{
    figure::{part, PartSpec},
    mesh::Shape::*,
    palette::*,
    rig::Node,
};

const X90: [f32; 3] = [90.0, 0.0, 0.0];

/// A straight sword: grip, guard, blade and point along +Z.
pub fn sword(at: Node, len: f32, blade: Color, guard: Color) -> Vec<PartSpec> {
    vec![
        part(at, Cyl(0.022, 0.16), DARK_LEATHER, [0.0, 0.0, 0.0]).rot(X90),
        part(at, Ball(0.035), guard, [0.0, 0.0, -0.1]),
        part(at, Box(0.2, 0.03, 0.04), guard, [0.0, 0.0, 0.09]),
        part(
            at,
            Box(0.07, 0.018, len),
            blade,
            [0.0, 0.0, 0.1 + len * 0.5],
        ),
        part(at, Cone(0.035, 0.1), blade, [0.0, 0.0, 0.15 + len])
            .rot(X90)
            .scale([1.0, 1.0, 0.4]),
    ]
}

/// A war hammer along +Z.
pub fn hammer(at: Node, len: f32, head: Color) -> Vec<PartSpec> {
    vec![
        part(at, Cyl(0.024, len), WOOD, [0.0, 0.0, len * 0.35]).rot(X90),
        part(at, Box(0.12, 0.2, 0.12), head, [0.0, 0.0, len * 0.85]),
        part(at, Box(0.13, 0.04, 0.13), GOLD, [0.0, 0.0, len * 0.85]),
    ]
}

/// A kite shield on the forearm's outside (+X), point down, team face.
pub fn kite_shield(at: Node, size: f32, rim: Color) -> Vec<PartSpec> {
    vec![
        part(at, Box(0.04, size, size * 0.8), ALLY, [0.08, 0.0, 0.0]),
        part(
            at,
            Box(0.04, size * 0.55, size * 0.55),
            ALLY,
            [0.08, -size * 0.5, 0.0],
        )
        .rot([45.0, 0.0, 0.0]),
        part(
            at,
            Box(0.045, size * 0.08, size * 0.82),
            rim,
            [0.08, size * 0.48, 0.0],
        ),
        part(
            at,
            Box(0.046, size * 0.3, size * 0.06),
            GOLD,
            [0.085, 0.0, 0.0],
        ),
        part(
            at,
            Box(0.046, size * 0.06, size * 0.3),
            GOLD,
            [0.085, 0.05, 0.0],
        ),
    ]
}

/// A staff held upright (+Y) with a head shape on top.
pub fn staff(at: Node, len: f32, wood: Color) -> Vec<PartSpec> {
    vec![
        part(at, Cyl(0.022, len), wood, [0.0, len * 0.3, 0.03]),
        part(at, Ball(0.03), wood, [0.0, -len * 0.2, 0.03]),
    ]
}

/// The top of a staff made with `staff` (where to put its head).
pub fn staff_top(len: f32) -> [f32; 3] {
    [0.0, len * 0.8, 0.03]
}

/// A quiver of arrows on the back spring.
pub fn quiver(color: Color, fletch: Color) -> Vec<PartSpec> {
    let b = Node::Secondary(crate::art::rig::Spring::Back);
    let mut v = vec![part(b, Cyl(0.06, 0.32), color, [0.08, 0.02, -0.06]).rot([0.0, 0.0, 18.0])];
    for (x, z) in [(0.05, -0.04), (0.1, -0.07), (0.08, -0.02)] {
        v.push(part(b, Box(0.012, 0.18, 0.03), fletch, [x + 0.05, 0.24, z]).rot([0.0, 0.0, 18.0]));
    }
    v
}

/// A small bottle: round belly, neck, cork; `liquid` glows.
pub fn flask(at: Node, at_xyz: [f32; 3], liquid: Color, size: f32) -> Vec<PartSpec> {
    let [x, y, z] = at_xyz;
    vec![
        part(at, Ball(0.06 * size), liquid, [x, y, z]).glow(),
        part(
            at,
            Cyl(0.022 * size, 0.06 * size),
            LINEN,
            [x, y + 0.07 * size, z],
        ),
        part(
            at,
            Cyl(0.026 * size, 0.025 * size),
            WOOD,
            [x, y + 0.1 * size, z],
        ),
    ]
}

/// A sack, tied at the top.
pub fn sack(at: Node, at_xyz: [f32; 3], color: Color, size: f32) -> Vec<PartSpec> {
    let [x, y, z] = at_xyz;
    vec![
        part(at, Ball(0.11 * size), color, [x, y, z]).scale([1.0, 1.15, 0.9]),
        part(
            at,
            Cyl(0.035 * size, 0.04 * size),
            DARK_LEATHER,
            [x, y + 0.13 * size, z],
        ),
    ]
}

/// A small wooden crate with iron bands.
pub fn crate_box(at: Node, at_xyz: [f32; 3], size: f32) -> Vec<PartSpec> {
    let [x, y, z] = at_xyz;
    let s = size;
    vec![
        part(at, Box(s, s * 0.8, s), OAK, [x, y, z]),
        part(
            at,
            Box(s * 1.03, s * 0.12, s * 1.03),
            IRON,
            [x, y + s * 0.25, z],
        ),
        part(
            at,
            Box(s * 1.03, s * 0.12, s * 1.03),
            IRON,
            [x, y - s * 0.25, z],
        ),
        part(at, Box(s * 0.2, s * 0.2, s * 1.04), GOLD, [x, y, z]),
    ]
}

/// A small lantern: brass cap and base around a glowing flame, with a
/// carrying ring on top.
pub fn lantern(at: Node, at_xyz: [f32; 3], size: f32) -> Vec<PartSpec> {
    let [x, y, z] = at_xyz;
    let s = size;
    vec![
        part(at, Ball(0.045 * s), SAP, [x, y, z]).glow(),
        part(
            at,
            Box(0.08 * s, 0.025 * s, 0.08 * s),
            BRASS,
            [x, y + 0.05 * s, z],
        ),
        part(
            at,
            Box(0.08 * s, 0.025 * s, 0.08 * s),
            BRASS,
            [x, y - 0.05 * s, z],
        ),
        part(
            at,
            Ring(0.008 * s, 0.03 * s, 360.0),
            BRASS,
            [x, y + 0.08 * s, z],
        )
        .rot(X90),
    ]
}

/// A rod from `a` to `b` (both in `at`'s space): struts, pistons, legs.
pub fn strut(at: Node, a: [f32; 3], b: [f32; 3], r: f32, color: Color) -> PartSpec {
    let (a, b) = (Vec3::from(a), Vec3::from(b));
    let d = b - a;
    let turn = Quat::from_rotation_arc(Vec3::Y, d.normalize_or(Vec3::Y));
    let (x, y, z) = turn.to_euler(EulerRot::XYZ);
    part(at, Cyl(r, d.length()), color, ((a + b) * 0.5).to_array()).rot([
        x.to_degrees(),
        y.to_degrees(),
        z.to_degrees(),
    ])
}

/// A round black bomb with a short fuse.
pub fn bomb(at: Node, at_xyz: [f32; 3], size: f32) -> Vec<PartSpec> {
    let [x, y, z] = at_xyz;
    vec![
        part(at, Ball(0.055 * size), DARK, [x, y, z]),
        part(
            at,
            Cyl(0.02 * size, 0.02 * size),
            IRON,
            [x, y + 0.055 * size, z],
        ),
        part(
            at,
            Cyl(0.006 * size, 0.04 * size),
            LINEN,
            [x, y + 0.08 * size, z],
        ),
    ]
}
