//! Shared weapons and props for kit champions. Hand-held weapons sit at a
//! hand socket's origin (the fist): blades and hafts run along +Z (forward
//! when the arm hangs), staffs and poles along +Y. Each returns parts so a
//! kit can recolor or extend them per form.

use bevy::prelude::*;

use crate::art::{
    anim::Pose,
    figure::{part, PartSpec},
    ik,
    mesh::Shape::*,
    palette::*,
    rig::{Joint, Node, Socket},
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

// Bow space: the grip sits on the origin, the long axis runs along +Y and the
// shot along +Z. The string lies behind the grip, on the archer's side.
/// Z of the limb tips and of the string when braced.
pub const BOW_TIP_Z: f32 = -0.10;
/// How far the nock travels back from brace to full draw.
pub const BOW_DRAW: f32 = 0.20;
/// Nock to arrowhead, so the head sits at the bow's muzzle when braced.
pub const ARROW_LEN: f32 = 0.42;
const LIMB_SEGMENTS: usize = 4;
/// How far the limb arcs forward of its chord: the curve that makes a bow.
const LIMB_BULGE: f32 = 0.22;
const LIMB_RADIUS: f32 = 0.018;
const STRING_RADIUS: f32 = 0.005;
/// String wobble after Release, as (degrees, decay per second, hertz).
const WOBBLE: (f32, f32, f32) = (7.0, 9.0, 16.0);

/// The colors a bow takes from its kit: staves, tips, grip and string, and
/// the arrow it holds.
pub struct BowStyle {
    pub wood: Color,
    pub tip: Color,
    pub grip: Color,
    pub string: Color,
    pub shaft: Color,
    pub head: Color,
    pub fletch: Color,
}

/// The string's half length: from a limb tip to the nock at full draw.
pub fn string_half(half: f32) -> f32 {
    half.hypot(BOW_DRAW)
}

/// Points along one limb from the grip to its tip (`sign` picks the upper or
/// lower limb). A quadratic curve bulging toward the target: the limb is an
/// arc, not a stick, and it ends exactly on its tip.
pub fn limb_points(half: f32, sign: f32) -> [Vec3; LIMB_SEGMENTS + 1] {
    let tip = Vec3::new(0.0, sign * half, BOW_TIP_Z);
    let control = tip * 0.5 + Vec3::Z * LIMB_BULGE;
    std::array::from_fn(|i| {
        let s = i as f32 / LIMB_SEGMENTS as f32;
        control * (2.0 * s * (1.0 - s)) + tip * (s * s)
    })
}

/// A strung bow on the bow socket: grip, two curved limbs with their tip
/// nocks, and the arrow on its own socket. The string halves and the arrow
/// are sockets of their own, so `bow_string` can draw them.
pub fn bow(half: f32, style: BowStyle) -> Vec<PartSpec> {
    let bow = Node::Prop(Socket::OffhandL);
    let mut v = vec![part(bow, Box(0.045, 0.14, 0.05), style.grip, [0.0; 3])];
    for sign in [1.0, -1.0] {
        let pts = limb_points(half, sign);
        for pair in pts.windows(2) {
            v.push(strut(
                bow,
                pair[0].to_array(),
                pair[1].to_array(),
                LIMB_RADIUS,
                style.wood,
            ));
        }
        for p in &pts[1..] {
            v.push(part(bow, Ball(LIMB_RADIUS), style.wood, p.to_array()));
        }
        v.push(part(
            bow,
            Box(0.026, 0.05, 0.026),
            style.tip,
            pts[LIMB_SEGMENTS].to_array(),
        ));
    }

    // Each string half hangs from its limb tip: the mesh runs from the pivot.
    let string = string_half(half);
    v.push(part(
        Node::Prop(Socket::StringTop),
        Cyl(STRING_RADIUS, string),
        style.string,
        [0.0, -string * 0.5, 0.0],
    ));
    v.push(part(
        Node::Prop(Socket::StringBottom),
        Cyl(STRING_RADIUS, string),
        style.string,
        [0.0, string * 0.5, 0.0],
    ));

    // The nocked arrow: its nock on the string, its head ahead along +Z.
    let nock = Node::Prop(Socket::Nock);
    v.push(
        part(
            nock,
            Cyl(0.006, ARROW_LEN),
            style.shaft,
            [0.0, 0.0, ARROW_LEN * 0.5],
        )
        .rot(X90),
    );
    v.push(
        part(
            nock,
            Cone(0.018, 0.05),
            style.head,
            [0.0, 0.0, ARROW_LEN - 0.025],
        )
        .rot(X90),
    );
    v.push(part(
        nock,
        Box(0.045, 0.006, 0.06),
        style.fletch,
        [0.0, 0.0, 0.03],
    ));
    v
}

/// The bow and string at rest: the string runs straight from tip to tip.
pub fn bow_rest(pose: &mut Pose, half: f32) {
    bow_string(pose, half, 0.0, 0.0);
}

/// Draws the string: `draw` (0..1) brings the nock back to full draw, and
/// `since` seconds after Release the string wobbles and settles. Each half
/// pivots on its limb tip and turns toward the nock.
pub fn bow_string(pose: &mut Pose, half: f32, draw: f32, since: f32) {
    let (amp, decay, hz) = WOBBLE;
    let wobble = if since > 0.0 {
        amp * (-decay * since).exp() * (std::f32::consts::TAU * hz * since).sin()
    } else {
        0.0
    };
    let pull = (BOW_DRAW * draw).atan2(half).to_degrees();
    pose[Node::Prop(Socket::StringTop).index()] = (
        Vec3::new(pull + wobble, 0.0, 0.0),
        Vec3::new(0.0, half, BOW_TIP_Z),
    );
    pose[Node::Prop(Socket::StringBottom).index()] = (
        Vec3::new(-pull - wobble, 0.0, 0.0),
        Vec3::new(0.0, -half, BOW_TIP_Z),
    );
    pose[Node::Prop(Socket::Nock).index()] =
        (Vec3::ZERO, Vec3::new(0.0, 0.0, BOW_TIP_Z - BOW_DRAW * draw));
}

/// Holds the bow upright whatever the arm does: its rotation cancels the
/// rotation of the arm and hand, so the long axis stays vertical and the shot
/// runs level, tilted up by `lift` degrees for high targets.
pub fn bow_upright(pose: &mut Pose, lift: f32) {
    let held = [
        Joint::Hips,
        Joint::Spine,
        Joint::Chest,
        Joint::UpperArmL,
        Joint::ForearmL,
        Joint::HandL,
    ]
    .into_iter()
    .fold(Quat::IDENTITY, |q, j| {
        q * ik::quat(pose[Node::Body(j).index()].0)
    });
    let want = Quat::from_rotation_x((-lift).to_radians());
    let (x, y, z) = (held.inverse() * want).to_euler(EulerRot::XYZ);
    pose[Node::Prop(Socket::OffhandL).index()].0 =
        Vec3::new(x.to_degrees(), y.to_degrees(), z.to_degrees());
}
