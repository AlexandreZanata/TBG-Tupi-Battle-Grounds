//! Rampart figure: an ogre of the Ironworks in riveted plate carrying a
//! mobile wall of oak and iron on the left arm, an iron gauntlet on the
//! right. Branch A (Bastion) grows the wall and its plating, then crowns
//! it with merlons and a team banner; branch B (Spiked) studs the wall
//! with spikes, then mounts a hot battering ram on the right arm.

use bevy::prelude::*;

use crate::art::{
    figure::{part, FigureSpec, PartSpec},
    kits::common::{body, Form},
    mesh::Shape::*,
    palette::*,
    rig::{Joint::*, Node::Body, Node::Prop, Node::Secondary, Proportions, Socket, Spring},
};

const HOT: Color = Color::srgb(1.0, 0.55, 0.18);

pub fn plan(tier: u8) -> Proportions {
    let mut p = body::plan(true, 0.95, tier);
    p.torso.x *= 1.1;
    p
}

/// The gauntlet (or ram) face (right hand space).
pub fn tip() -> Vec3 {
    Vec3::new(0.0, -0.15, 0.0)
}

pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let f = Form::of(tier, branch);
    let p = plan(tier);
    let mut v = Vec::with_capacity(140);
    let look = body::Look {
        skin: OGRE_SKIN,
        cloth: DARK_LEATHER,
        legs: GUNMETAL,
        boots: IRON,
    };
    body::humanoid(&mut v, &p, &look);
    let (t, h) = (p.torso, p.head);
    v.extend([
        part(
            Body(Chest),
            Box(t.x * 1.05, t.y * 0.7, t.z * 1.15),
            GUNMETAL,
            [0.0, t.y * 0.58, 0.0],
        ),
        part(
            Body(Chest),
            Box(t.x * 0.3, 0.05, 0.03),
            BRASS,
            [0.0, t.y * 0.85, t.z * 0.6],
        ),
        part(
            Body(Head),
            Cone(h * 0.15, h * 0.35),
            LINEN,
            [h * 0.35, -h * 0.45, h * 0.85],
        ),
        part(
            Body(Head),
            Cone(h * 0.15, h * 0.35),
            LINEN,
            [-h * 0.35, -h * 0.45, h * 0.85],
        ),
        part(
            Secondary(Spring::Back),
            Cyl(0.05, 0.3),
            IRON,
            [0.12, 0.2, -0.04],
        ),
        part(Body(HandR), Box(0.2, 0.2, 0.2), IRON, [0.0, -0.06, 0.0]),
    ]);
    body::helmet(&mut v, &p, body::Helm::Kettle(IRON));
    body::pauldrons(&mut v, IRON, 0.13);
    body::greaves(&mut v, IRON);
    body::bracers(&mut v, &p, BRASS);
    wall(&mut v, &p, f);
    if f.b && f.t2 {
        // Siegebreaker: a battering ram along the right forearm.
        let fore = Body(ForearmR);
        v.extend([
            part(fore, Cyl(0.09, p.arm * 1.3), IRON, [0.0, -p.arm * 0.6, 0.0]),
            part(
                fore,
                Frustum(0.13, 0.09, 0.12),
                GUNMETAL,
                [0.0, -p.arm * 1.25, 0.0],
            )
            .rot([180.0, 0.0, 0.0]),
            part(fore, Cyl(0.1, 0.04), HOT, [0.0, -p.arm * 0.9, 0.0]).glow(),
        ]);
    }
    FigureSpec { plan: p, parts: v }
}

/// The wall stands upright before the chest, over the left side, so the
/// right fist strikes round its edge; the left hand holds its grip.
pub fn grip(p: &Proportions) -> Vec3 {
    let t = p.torso;
    Vec3::new(t.x * 0.35, t.y * 0.3, t.z * 0.9)
}

/// The mobile wall of oak planks and iron bands; its face looks forward.
fn wall(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    let c = Body(Chest);
    let (tall, wide) = if f.a { (0.7, 0.56) } else { (0.62, 0.5) };
    let g = grip(p);
    let (x, y, z) = (g.x, g.y, g.z + 0.07);
    for k in [-1.0f32, 0.0, 1.0] {
        v.push(part(
            c,
            Box(wide / 3.0 - 0.01, tall, 0.05),
            OAK,
            [x + k * wide / 3.0, y, z],
        ));
    }
    v.extend([
        part(
            c,
            Box(wide + 0.02, 0.06, 0.06),
            IRON,
            [x, y + tall * 0.35, z + 0.005],
        ),
        part(
            c,
            Box(wide + 0.02, 0.06, 0.06),
            IRON,
            [x, y - tall * 0.35, z + 0.005],
        ),
        part(
            c,
            Box(0.14, 0.035, 0.065),
            DARK,
            [x, y + tall * 0.17, z + 0.005],
        ),
        part(
            c,
            Box(wide * 0.3, tall * 0.32, 0.06),
            ALLY,
            [x, y - tall * 0.05, z + 0.006],
        ),
    ]);
    if f.a {
        v.push(part(
            c,
            Box(0.05, tall * 0.9, 0.065),
            IRON,
            [x - wide * 0.3, y, z + 0.008],
        ));
    }
    if f.a && f.t2 {
        for k in [-0.375f32, -0.125, 0.125, 0.375] {
            v.push(part(
                c,
                Box(0.1, 0.1, 0.07),
                STONE,
                [x + k * wide, y + tall * 0.55, z],
            ));
        }
        v.extend([
            part(
                c,
                Cyl(0.012, 0.5),
                WOOD,
                [x + wide * 0.45, y + tall * 0.75, z - 0.02],
            ),
            part(
                c,
                Box(0.2, 0.14, 0.015),
                ALLY,
                [x + wide * 0.45 - 0.1, y + tall * 0.9, z - 0.02],
            ),
            part(
                c,
                Box(wide + 0.03, 0.03, 0.07),
                GOLD,
                [x, y + tall * 0.47, z],
            ),
        ]);
    }
    if f.b {
        for (dx, dy) in [(-0.3, 0.25), (0.3, 0.25), (-0.3, -0.25), (0.3, -0.25)] {
            v.push(
                part(
                    c,
                    Cone(0.03, 0.12),
                    STEEL,
                    [x + dx * wide, y + dy * tall, z + 0.08],
                )
                .rot([90.0, 0.0, 0.0]),
            );
        }
    }
}
