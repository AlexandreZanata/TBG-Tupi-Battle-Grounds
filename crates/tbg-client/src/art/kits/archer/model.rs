//! Archer figure: a hooded yeoman archer of the Humans faction with an upright
//! longbow, a quiver on the back, linen tunic, and team tabard.
//! Branch A (Longbow -> Marksman) extends range and accuracy: green ranger hood
//! with feather, yew longbow with horn nocks, then a gilded recurve greatbow,
//! team half-cape, gold trim, and glowing eye glint.
//! Branch B (Volley -> Arrow Rain) fires rapid storms of arrows: dark leather
//! coif, dual quivers on back, composite war-bow, then an armored steel-brow
//! hood, twin gilded quivers, double-cable siege bow with glowing riser runes.

use bevy::prelude::*;

use crate::art::{
    figure::{part, FigureSpec, PartSpec},
    kits::common::{body, gear, Form},
    mesh::Shape::*,
    palette::*,
    rig::{Joint::*, Node::Body, Node::Prop, Node::Secondary, Proportions, Socket, Spring},
};

const HOT: Color = Color::srgb(1.0, 0.55, 0.18);

pub fn plan(tier: u8) -> Proportions {
    let mut p = body::plan(false, 1.0, tier);
    p.arm = 0.20;
    p
}

/// The arrow's point in `Socket::OffhandL` space: a braced arrow's head.
pub const MUZZLE: Vec3 = Vec3::new(0.0, 0.0, gear::BOW_TIP_Z + gear::ARROW_LEN);

/// Half the bow's length, grip to limb tip, by form.
pub fn half_len(f: Form) -> f32 {
    if f.a && f.t2 {
        0.70
    } else if f.a {
        0.74
    } else if f.b && f.t2 {
        0.68
    } else if f.b {
        0.65
    } else {
        0.64
    }
}

pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let f = Form::of(tier, branch);
    let p = plan(tier);
    let mut v = Vec::with_capacity(120);

    let look = body::Look {
        skin: SKIN,
        cloth: LINEN,
        legs: DARK_LEATHER,
        boots: DARK_LEATHER,
    };
    body::humanoid(&mut v, &p, &look);
    body::tunic(&mut v, &p, LINEN, DARK_LEATHER, LEATHER);
    body::bracers(&mut v, &p, if f.t2 { GOLD } else { DARK_LEATHER });

    head(&mut v, &p, f);
    dress(&mut v, &p, f);
    quivers(&mut v, f);
    bow(&mut v, f);

    FigureSpec { plan: p, parts: v }
}

fn head(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    let h = p.head;
    if f.b {
        // Branch B: Dark leather cowl
        body::hood(v, p, DARK_LEATHER, DARK);
        if f.t2 {
            // Arrow Rain: steel brow plate and glowing eye slit
            v.push(part(
                Body(Head),
                Box(h * 1.5, 0.035, 0.04),
                STEEL,
                [0.0, h * 0.25, h * 0.94],
            ));
            body::glowing_eyes(v, p, RED_CLOTH);
        }
    } else if f.a {
        // Branch A: Ranger green hood
        body::hood(v, p, HOOD_GREEN, DARK_LEATHER);
        // Feather plume on the side of the hood
        v.push(
            part(
                Body(Head),
                Cone(0.02, 0.16),
                LINEN,
                [-h * 0.85, h * 0.55, -0.05],
            )
            .rot([25.0, 0.0, 20.0]),
        );
        if f.t2 {
            // Marksman: gold hood trim & glowing eye glint
            v.push(part(
                Body(Head),
                Ring(0.015, h * 0.95, 360.0),
                GOLD,
                [0.0, h * 0.15, h * 0.2],
            ));
            body::glowing_eyes(v, p, GOLD);
        }
    } else {
        // Base form: classic oak hood
        body::hood(v, p, OAK, DARK_LEATHER);
    }
}

fn dress(v: &mut Vec<PartSpec>, _p: &Proportions, f: Form) {
    // Team tabard is visible from match camera
    body::tabard(v, 0.18, 0.30);

    if f.a && f.t2 {
        // Marksman: team half-cape with gold hem
        body::cape(v, ALLY, 0.34, 0.44);
        v.push(part(
            Secondary(Spring::Back),
            Box(0.34, 0.03, 0.02),
            GOLD,
            [0.0, 0.09 - 0.44 * 0.5, -0.04],
        ));
        body::pauldrons(v, GOLD, 0.14);
    } else if f.b && f.t2 {
        // Arrow Rain: armored steel pauldrons and battle mantle
        body::cape(v, ALLY, 0.36, 0.40);
        body::pauldrons(v, IRON, 0.15);
    }
}

fn quivers(v: &mut Vec<PartSpec>, f: Form) {
    let b = Secondary(Spring::Back);
    if f.b {
        // Branch B: Dual quivers on back
        for (x, rot) in [(-0.10, -12.0), (0.10, 12.0)] {
            let rim = if f.t2 { GOLD } else { DARK_LEATHER };
            v.push(part(b, Cyl(0.055, 0.34), LEATHER, [x, 0.04, -0.08]).rot([0.0, 0.0, rot]));
            v.push(part(b, Ring(0.008, 0.056, 360.0), rim, [x, 0.18, -0.08]).rot([0.0, 0.0, rot]));
            // Arrows bristling from both quivers
            for (dx, dz) in [(-0.02, -0.02), (0.02, 0.01), (0.0, -0.03)] {
                let fletch = if f.t2 { ORANGE } else { RED_CLOTH };
                v.push(
                    part(
                        b,
                        Box(0.01, 0.16, 0.025),
                        fletch,
                        [x + dx, 0.28, -0.08 + dz],
                    )
                    .rot([0.0, 0.0, rot]),
                );
            }
        }
    } else {
        // Single quiver on back
        let (color, fletch) = if f.a && f.t2 {
            (GOLD, GOLD)
        } else if f.a {
            (DARK_LEATHER, ALLY)
        } else {
            (DARK_LEATHER, LINEN)
        };
        v.extend(gear::quiver(color, fletch));
    }
}

/// The bow on the left hand: a curved, strung longbow (see `gear::bow`), with
/// each form's own trim on top of it.
fn bow(v: &mut Vec<PartSpec>, f: Form) {
    let (wood, tip) = if f.a && f.t2 {
        (RUST, GOLD)
    } else if f.a {
        (TIMBER, BONE)
    } else if f.b && f.t2 {
        (GUNMETAL, STEEL)
    } else if f.b {
        (IRON, DARK_LEATHER)
    } else {
        (OAK, TRIM)
    };
    let style = gear::BowStyle {
        wood,
        tip,
        grip: DARK_LEATHER,
        string: if f.t2 { GOLD } else { LINEN },
        shaft: TIMBER,
        head: STEEL,
        fletch: if f.a { ALLY } else { LINEN },
    };
    let half = half_len(f);
    v.extend(gear::bow(half, style));

    let at = Prop(Socket::OffhandL);
    if f.a && f.t2 {
        // Marksman: recurve tips curling away, brass sight pin
        for s in [1.0f32, -1.0] {
            v.push(
                part(
                    at,
                    Box(0.02, 0.10, 0.02),
                    GOLD,
                    [0.0, s * (half + 0.04), gear::BOW_TIP_Z + 0.03],
                )
                .rot([-s * 35.0, 0.0, 0.0]),
            );
        }
        v.push(part(at, Cone(0.012, 0.06), BRASS, [0.03, 0.04, 0.02]).rot([0.0, 0.0, -90.0]));
    }
    if f.b && f.t2 {
        // Arrow Rain: heavy reinforced riser plates and glowing center core
        v.push(part(at, Box(0.055, 0.18, 0.065), STEEL, [0.0, 0.0, 0.0]));
        v.push(part(at, Ball(0.025), HOT, [0.0, 0.0, 0.04]).glow());
    }
}
