//! Siege Walker figure: a hulking ogre gunner of the Ironworks carrying a
//! shoulder bombard. Evolutions change the bombard (the silhouette piece),
//! the armor (large surfaces) and, at tier 2, add a glowing signature.

use bevy::prelude::*;

use crate::art::{
    figure::{part, FigureSpec, PartSpec},
    mesh::Shape::*,
    palette::*,
    rig::{
        Joint::*,
        Node::{Body, Prop, Secondary},
        Proportions, Socket, Spring,
    },
};

const BRONZE: Color = Color::srgb(0.70, 0.47, 0.24);
const TROUSERS: Color = Color::srgb(0.33, 0.27, 0.22);
const HOT: Color = Color::srgb(1.0, 0.55, 0.18);

/// Which Evolution pieces a form has.
#[derive(Clone, Copy)]
pub struct Form {
    pub a: bool,
    pub b: bool,
    pub t2: bool,
}

impl Form {
    pub fn of(tier: u8, branch: u8) -> Form {
        Form {
            a: tier >= 1 && branch == 0,
            b: tier >= 1 && branch == 1,
            t2: tier >= 2,
        }
    }
}

/// Ogre proportions: short legs, broad chest, long gorilla arms so both
/// hands reach the bombard on the right shoulder.
pub fn plan(tier: u8) -> Proportions {
    Proportions {
        scale: 1.45 * 1.2 * (1.0 + 0.08 * f32::from(tier.min(2))),
        leg: 0.34,
        torso: Vec3::new(0.48, 0.36, 0.3),
        head: 0.13,
        arm: 0.25,
    }
}

/// The bombard in its socket's space (the right hand's grip at the origin,
/// the barrel above it along +Z).
pub struct Bombard {
    pub muzzle: Vec3,
    /// Where the left hand holds it.
    pub fore_grip: Vec3,
    /// The breech, for the backblast.
    pub breech: Vec3,
}

/// Barrel axis height above the grip: the hand rides at chest height and
/// the barrel rests on the shoulder.
const BORE: f32 = 0.26;
/// Where the left hand turns the reload crank (centre of its circle).
pub const CRANK: Vec3 = Vec3::new(0.13, BORE - 0.05, 0.02);

pub fn bombard(f: Form) -> Bombard {
    let (len, back) = gun_length(f);
    let bell = if f.b && !f.t2 { 0.16 } else { 0.08 };
    Bombard {
        muzzle: Vec3::new(0.0, BORE, back + len + bell),
        fore_grip: Vec3::new(0.12, 0.1, 0.12),
        breech: Vec3::new(0.0, BORE, back - 0.08),
    }
}

fn gun_length(f: Form) -> (f32, f32) {
    let len = match (f.a, f.b, f.t2) {
        (true, _, false) => 1.08,
        (true, _, true) => 0.98,
        (_, true, false) => 0.8,
        (_, true, true) => 0.9,
        _ => 0.86,
    };
    (len, -0.32)
}

pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let f = Form::of(tier, branch);
    let p = plan(tier);
    let mut v = Vec::with_capacity(120);
    legs(&mut v, &p, f);
    hips(&mut v, f);
    chest(&mut v, &p, f);
    head(&mut v, f);
    arms(&mut v, &p, f);
    back(&mut v, f);
    gun(&mut v, f);
    FigureSpec { plan: p, parts: v }
}

fn legs(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    let l = p.leg;
    let plate = if f.a { STEEL } else { IRON };
    for (thigh, shin, foot) in [(ThighL, ShinL, FootL), (ThighR, ShinR, FootR)] {
        v.extend([
            part(
                Body(thigh),
                Frustum(0.095, 0.08, l * 0.5),
                TROUSERS,
                [0.0, -l * 0.25, 0.0],
            ),
            part(
                Body(shin),
                Frustum(0.08, 0.07, l * 0.5),
                DARK_LEATHER,
                [0.0, -l * 0.25, 0.0],
            ),
            part(Body(shin), Box(0.13, 0.1, 0.05), plate, [0.0, -0.01, 0.07]).rot([-8.0, 0.0, 0.0]),
            part(
                Body(shin),
                Box(0.15, 0.04, 0.15),
                LEATHER,
                [0.0, -l * 0.32, 0.0],
            ),
            part(
                Body(foot),
                Box(0.17, 0.09, 0.25),
                DARK_LEATHER,
                [0.0, -0.025, 0.05],
            ),
            part(
                Body(foot),
                Box(0.175, 0.045, 0.09),
                IRON,
                [0.0, -0.035, 0.15],
            ),
        ]);
    }
}

fn hips(v: &mut Vec<PartSpec>, f: Form) {
    let cloth = if f.b { RUST } else { ALLY };
    v.extend([
        part(Body(Hips), Box(0.42, 0.17, 0.29), TROUSERS, [0.0, 0.0, 0.0]),
        part(
            Body(Hips),
            Box(0.47, 0.065, 0.32),
            LEATHER,
            [0.0, 0.08, 0.0],
        ),
        part(Body(Hips), Box(0.09, 0.08, 0.02), BRASS, [0.0, 0.08, 0.165]),
        part(
            Body(Hips),
            Box(0.08, 0.1, 0.07),
            LEATHER,
            [0.19, 0.02, 0.12],
        ),
        part(
            Body(Hips),
            Box(0.08, 0.1, 0.07),
            LEATHER,
            [-0.19, 0.02, 0.12],
        ),
        // Team loincloth on the tabard spring, iron-weighted hem.
        part(
            Secondary(Spring::Tabard),
            Box(0.2, 0.27, 0.02),
            cloth,
            [0.0, -0.08, 0.0],
        ),
        part(
            Secondary(Spring::Tabard),
            Box(0.2, 0.035, 0.026),
            IRON,
            [0.0, -0.2, 0.0],
        ),
        part(
            Secondary(Spring::Tabard),
            Box(0.06, 0.06, 0.025),
            ALLY,
            [0.0, -0.06, 0.004],
        )
        .rot([0.0, 0.0, 45.0]),
    ]);
}

fn chest(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    let h = p.torso.y;
    v.extend([
        part(
            Body(Chest),
            Frustum(0.28, 0.22, h),
            OGRE_SKIN,
            [0.0, h * 0.52, 0.0],
        )
        .scale([1.0, 1.0, 0.74]),
        part(Body(Chest), Ball(0.19), OGRE_SKIN, [0.0, 0.1, 0.08]).scale([1.15, 0.9, 0.85]),
        part(Body(Chest), Ball(0.13), OGRE_SKIN, [0.0, h * 0.94, -0.04]).scale([1.7, 0.6, 1.0]),
    ]);
    // Leather harness crossing the chest, a brass ring at the crossing.
    for s in [-1.0f32, 1.0] {
        v.push(
            part(
                Body(Chest),
                Box(0.065, 0.5, 0.025),
                DARK_LEATHER,
                [0.07 * s, h * 0.55, 0.16],
            )
            .rot([-6.0, 0.0, 33.0 * s]),
        );
    }
    v.push(part(Body(Chest), Cyl(0.04, 0.03), BRASS, [0.0, h * 0.55, 0.18]).rot([90.0, 0.0, 0.0]));
    if f.a {
        // Longshot / Artillery: a riveted breastplate (large-surface change).
        let (plate, trim) = if f.t2 { (STEEL, GOLD) } else { (STEEL, BRASS) };
        v.push(
            part(
                Body(Chest),
                Box(0.4, 0.25, 0.06),
                plate,
                [0.0, h * 0.62, 0.15],
            )
            .rot([-8.0, 0.0, 0.0]),
        );
        v.push(
            part(
                Body(Chest),
                Box(0.42, 0.035, 0.07),
                trim,
                [0.0, h * 0.78, 0.14],
            )
            .rot([-8.0, 0.0, 0.0]),
        );
        for x in [-0.15, 0.15] {
            v.push(part(Body(Chest), Ball(0.022), trim, [x, h * 0.5, 0.19]));
        }
    }
    if f.b {
        // Shrapnel / Concussion: a bandolier of shells across the belly.
        v.push(
            part(
                Body(Chest),
                Box(0.08, 0.56, 0.04),
                RUST,
                [0.0, h * 0.42, 0.19],
            )
            .rot([0.0, 0.0, -38.0]),
        );
        for i in 0..5 {
            let t = i as f32 / 4.0 - 0.5;
            v.push(
                part(
                    Body(Chest),
                    Cyl(0.028, 0.09),
                    BRASS,
                    [t * 0.3, h * 0.42 + t * 0.24, 0.215],
                )
                .rot([0.0, 0.0, -38.0]),
            );
        }
    }
    if f.b && f.t2 {
        v.push(part(
            Body(Chest),
            Box(0.36, 0.09, 0.3),
            IRON,
            [0.0, h * 0.95, 0.02],
        ));
        v.push(
            part(
                Body(Chest),
                Box(0.37, 0.025, 0.31),
                HOT,
                [0.0, h * 0.95, 0.02],
            )
            .glow(),
        );
    }
}

fn head(v: &mut Vec<PartSpec>, f: Form) {
    let eye = if f.t2 { HOT } else { DARK };
    v.extend([
        part(Body(Head), Ball(0.13), OGRE_SKIN, [0.0, 0.0, 0.01]).scale([1.15, 0.95, 1.05]),
        part(
            Body(Head),
            Box(0.25, 0.05, 0.08),
            OGRE_SKIN,
            [0.0, 0.045, 0.09],
        )
        .rot([-12.0, 0.0, 0.0]),
        part(
            Body(Head),
            Box(0.22, 0.085, 0.15),
            OGRE_SKIN,
            [0.0, -0.08, 0.05],
        ),
        part(
            Body(Head),
            Box(0.05, 0.05, 0.05),
            OGRE_SKIN,
            [0.0, -0.005, 0.135],
        ),
    ]);
    let mut eyes = vec![
        part(
            Body(Head),
            Box(0.04, 0.026, 0.02),
            eye,
            [0.05, 0.012, 0.128],
        ),
        part(
            Body(Head),
            Box(0.04, 0.026, 0.02),
            eye,
            [-0.05, 0.012, 0.128],
        ),
    ];
    if f.t2 {
        eyes = eyes.into_iter().map(PartSpec::glow).collect();
    }
    v.extend(eyes);
    for s in [-1.0f32, 1.0] {
        v.push(
            part(
                Body(Head),
                Cone(0.022, 0.085),
                BONE,
                [0.075 * s, -0.035, 0.12],
            )
            .rot([12.0, 0.0, -10.0 * s]),
        );
        v.push(
            part(
                Body(Head),
                Cone(0.032, 0.09),
                OGRE_SKIN,
                [0.15 * s, 0.03, -0.01],
            )
            .rot([0.0, 0.0, -78.0 * s]),
        );
    }
    match (f.a, f.b, f.t2) {
        // Artillery: a plumed great helm in gunmetal and gold.
        (true, _, true) => {
            v.push(
                part(Body(Head), Ball(0.142), GUNMETAL, [0.0, 0.05, -0.01])
                    .scale([1.12, 0.62, 1.1]),
            );
            v.push(part(
                Body(Head),
                Box(0.3, 0.035, 0.29),
                GOLD,
                [0.0, 0.035, -0.01],
            ));
            v.push(part(
                Secondary(Spring::HeadGear),
                Box(0.04, 0.14, 0.26),
                ALLY,
                [0.0, 0.17, -0.03],
            ));
            v.push(part(
                Secondary(Spring::HeadGear),
                Box(0.045, 0.04, 0.27),
                GOLD,
                [0.0, 0.1, -0.03],
            ));
        }
        // Concussion: an iron face guard with glowing slits.
        (_, true, true) => {
            v.push(
                part(Body(Head), Ball(0.138), IRON, [0.0, 0.05, -0.01]).scale([1.12, 0.55, 1.1]),
            );
            v.push(part(
                Body(Head),
                Box(0.26, 0.12, 0.06),
                IRON,
                [0.0, -0.075, 0.12],
            ));
            v.push(part(Body(Head), Box(0.2, 0.018, 0.02), HOT, [0.0, -0.06, 0.152]).glow());
        }
        _ => {
            v.push(
                part(Body(Head), Ball(0.136), IRON, [0.0, 0.055, -0.01]).scale([1.12, 0.5, 1.08]),
            );
            v.push(part(
                Body(Head),
                Box(0.04, 0.04, 0.22),
                RUST,
                [0.0, 0.12, -0.01],
            ));
        }
    }
    if f.a && !f.t2 {
        // Longshot: a brass sighting monocle over the right eye.
        v.push(
            part(
                Body(Head),
                Ring(0.012, 0.035, 360.0),
                BRASS,
                [-0.05, 0.012, 0.14],
            )
            .rot([90.0, 0.0, 0.0]),
        );
        v.push(part(
            Body(Head),
            Box(0.12, 0.018, 0.018),
            BRASS,
            [-0.1, 0.03, 0.12],
        ));
    }
}

fn arms(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    let a = p.arm;
    for (upper, fore, hand) in [(UpperArmL, ForearmL, HandL), (UpperArmR, ForearmR, HandR)] {
        v.extend([
            part(Body(upper), Ball(0.1), OGRE_SKIN, [0.0, -0.02, 0.0]),
            part(
                Body(upper),
                Frustum(0.09, 0.075, a),
                OGRE_SKIN,
                [0.0, -a * 0.5, 0.0],
            ),
            part(
                Body(fore),
                Frustum(0.09, 0.07, a * 0.9),
                OGRE_SKIN,
                [0.0, -a * 0.45, 0.0],
            ),
            part(
                Body(fore),
                Box(0.155, 0.11, 0.155),
                DARK_LEATHER,
                [0.0, -a * 0.6, 0.0],
            ),
            part(Body(hand), Ball(0.075), OGRE_SKIN, [0.0, -0.045, 0.0]).scale([1.0, 0.9, 1.1]),
        ]);
    }
    // Right pauldron: the bombard rests on it.
    let right = if f.a { STEEL } else { IRON };
    v.push(part(Body(UpperArmR), Ball(0.115), right, [0.0, 0.03, 0.0]).scale([1.3, 0.62, 1.2]));
    if f.b {
        // Shrapnel: a spiked pauldron on the left shoulder.
        v.push(part(Body(UpperArmL), Ball(0.115), IRON, [0.0, 0.03, 0.0]).scale([1.3, 0.62, 1.2]));
        for (x, z) in [(0.03, 0.05), (0.06, -0.05), (-0.02, 0.0)] {
            v.push(
                part(Body(UpperArmL), Cone(0.03, 0.11), STEEL, [x, 0.1, z]).rot([0.0, 0.0, -18.0]),
            );
        }
    }
    if f.t2 {
        // Tier 2: iron gauntlets on both hands.
        for hand in [HandL, HandR] {
            v.push(part(
                Body(hand),
                Box(0.13, 0.08, 0.14),
                IRON,
                [0.0, -0.02, 0.0],
            ));
        }
    }
}

fn back(v: &mut Vec<PartSpec>, f: Form) {
    let b = Secondary(Spring::Back);
    // A team pennant on a pole: the side reads from the match camera.
    v.extend([
        part(b, Cyl(0.014, 0.62), WOOD, [0.12, 0.22, -0.16]),
        part(b, Box(0.02, 0.15, 0.2), ALLY, [0.12, 0.44, -0.27]),
        part(b, Box(0.022, 0.03, 0.2), GOLD, [0.12, 0.36, -0.27]),
    ]);
    if f.t2 {
        // A furnace-boiler that feeds the great guns; glowing firebox.
        v.extend([
            part(b, Cyl(0.13, 0.32), IRON, [0.0, 0.0, -0.07]),
            part(b, Cyl(0.135, 0.03), BRASS, [0.0, 0.12, -0.07]),
            part(b, Cyl(0.135, 0.03), BRASS, [0.0, -0.12, -0.07]),
            part(b, Cyl(0.035, 0.22), DARK, [0.07, 0.26, -0.1]),
            part(b, Box(0.13, 0.09, 0.03), HOT, [0.0, -0.04, -0.2]).glow(),
        ]);
        return;
    }
    if f.a {
        // Longshot: a rack of long shells.
        for x in [-0.08, 0.0, 0.08] {
            v.push(part(b, Cyl(0.035, 0.32), BRASS, [x, 0.02, -0.06]));
            v.push(part(b, Cone(0.035, 0.07), IRON, [x, 0.21, -0.06]));
        }
        v.push(part(b, Box(0.26, 0.05, 0.08), OAK, [0.0, -0.08, -0.06]));
        return;
    }
    // A powder keg strapped to the back (B1 adds a crate of shot).
    v.extend([
        part(b, Cyl(0.11, 0.24), OAK, [0.0, 0.0, -0.07]),
        part(b, Cyl(0.115, 0.025), IRON, [0.0, 0.08, -0.07]),
        part(b, Cyl(0.115, 0.025), IRON, [0.0, -0.08, -0.07]),
        part(b, Cyl(0.012, 0.08), LINEN, [0.04, 0.15, -0.07]),
    ]);
    if f.b {
        v.push(part(b, Box(0.2, 0.14, 0.14), OAK, [0.0, -0.2, -0.08]));
        for x in [-0.05, 0.05] {
            v.push(part(b, Ball(0.045), DARK, [x, -0.12, -0.08]));
        }
    }
}

/// The bombard on the right hand's socket.
fn gun(v: &mut Vec<PartSpec>, f: Form) {
    let w = Prop(Socket::WeaponR);
    let (len, back) = gun_length(f);
    let front = back + len;
    let along = |x: f32, z: f32| [x, BORE, z];
    let r = if f.t2 {
        0.088
    } else if f.a {
        0.064
    } else {
        0.078
    };
    let (metal, band) = match (f.a, f.t2) {
        (true, true) => (GUNMETAL, GOLD),
        (true, false) => (BRONZE, BRASS),
        _ if f.b && f.t2 => (GUNMETAL, IRON),
        _ => (BRONZE, IRON),
    };
    let x90 = [90.0, 0.0, 0.0];
    if f.a && f.t2 {
        // Artillery: twin bores side by side with gold muzzles.
        for x in [-0.068, 0.068] {
            v.push(part(w, Cyl(0.064, len), metal, along(x, back + len * 0.5)).rot(x90));
            v.push(part(w, Frustum(0.085, 0.07, 0.09), GOLD, along(x, front + 0.04)).rot(x90));
        }
        v.push(part(
            w,
            Box(0.27, 0.11, 0.44),
            GUNMETAL,
            along(0.0, back + 0.28),
        ));
        v.push(part(w, Box(0.28, 0.022, 0.05), HOT, along(0.0, back + 0.44)).glow());
    } else {
        v.push(part(w, Cyl(r, len), metal, along(0.0, back + len * 0.5)).rot(x90));
        let blunderbuss = f.b && !f.t2;
        // Shrapnel: a wide blunderbuss bell.
        let muzzle = if blunderbuss {
            Frustum(r * 2.3, r * 1.05, 0.16)
        } else {
            Frustum(r * 1.4, r * 1.05, 0.08)
        };
        let bell = if blunderbuss { 0.08 } else { 0.04 };
        v.push(part(w, muzzle, band, along(0.0, front + bell)).rot(x90));
    }
    for k in [0.12, 0.45, 0.82] {
        v.push(part(w, Cyl(r * 1.18, 0.035), band, along(0.0, back + len * k)).rot(x90));
    }
    // Breech block, exhaust flare, sight, mount and grip, fore-grip, crank.
    v.extend([
        part(w, Box(r * 2.5, r * 2.5, 0.13), IRON, along(0.0, back)),
        part(
            w,
            Frustum(r * 1.5, r * 1.1, 0.07),
            DARK,
            along(0.0, back - 0.09),
        )
        .rot([-90.0, 0.0, 0.0]),
        part(
            w,
            Box(0.02, 0.05, 0.03),
            IRON,
            [0.0, BORE + r + 0.02, front - 0.12],
        ),
        part(w, Box(0.07, 0.12, 0.12), IRON, [0.0, BORE - r - 0.04, 0.02]),
        part(w, Box(0.05, 0.15, 0.065), OAK, [0.0, 0.03, 0.0]).rot([12.0, 0.0, 0.0]),
        part(w, Box(0.12, 0.03, 0.03), OAK, [0.07, 0.15, 0.12]),
        part(w, Box(0.035, 0.1, 0.035), OAK, [0.12, 0.1, 0.12]),
        part(w, Cyl(0.04, 0.03), BRASS, [r + 0.02, BORE, CRANK.z]).rot([0.0, 0.0, 90.0]),
        part(
            w,
            Box(0.018, 0.06, 0.018),
            IRON,
            [r + 0.04, BORE - 0.03, CRANK.z],
        ),
        part(
            w,
            Box(0.02, 0.02, 0.05),
            OAK,
            [r + 0.05, BORE - 0.06, CRANK.z],
        ),
    ]);
    if f.a && !f.t2 {
        // Longshot: a long brass telescope along the barrel.
        let top = BORE + r + 0.035;
        v.push(part(w, Cyl(0.026, 0.42), BRASS, [0.06, top, back + 0.45]).rot(x90));
        v.push(part(w, Cyl(0.034, 0.04), BRASS, [0.06, top, back + 0.68]).rot(x90));
    }
    if f.b && f.t2 {
        // Concussion: a drum magazine of hammer shells with glowing bands.
        let z90 = [0.0, 0.0, 90.0];
        let drum = [0.0, BORE - 0.02, back + 0.3];
        v.push(part(w, Cyl(0.14, 0.14), IRON, drum).rot(z90));
        for x in [-0.06, 0.06] {
            let at = [x, BORE - 0.02, back + 0.3];
            v.push(part(w, Ring(0.014, 0.14, 360.0), HOT, at).rot(z90).glow());
        }
        v.push(part(
            w,
            Box(0.12, 0.08, 0.1),
            IRON,
            along(0.0, front - 0.05),
        ));
    }
}
