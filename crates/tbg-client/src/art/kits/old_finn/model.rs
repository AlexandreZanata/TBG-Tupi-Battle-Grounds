//! Finn's rounded oilskin, forked beard, sou'wester, readable reel and branch-specific catch.

use bevy::prelude::*;

use crate::art::{
    figure::{part, FigureSpec, PartSpec},
    kits::common::{body, gear, Form},
    mesh::Shape::*,
    palette::*,
    rig::{Joint::*, Node::Body, Node::Prop, Node::Secondary, Proportions, Socket, Spring},
};

/// The original rod endpoint remains the projectile's release socket.
pub const ROD: f32 = 1.1;
pub const TIP: Vec3 = Vec3::new(0.0, ROD * 0.82, 0.0);

pub fn plan(tier: u8) -> Proportions {
    let mut p = body::plan(false, 0.95, tier);
    p.torso.x = 0.40;
    p.torso.z = 0.27;
    p.leg = 0.36;
    p.arm = 0.21;
    p
}

pub fn spec(tier: u8, branch: u8) -> FigureSpec {
    let f = Form::of(tier, branch);
    let p = plan(tier);
    let mut v = Vec::with_capacity(130);
    let coat = if f.a { OILSKIN_SHADE } else { OILSKIN_YELLOW };
    body::humanoid(
        &mut v,
        &p,
        &body::Look {
            skin: SKIN,
            cloth: coat,
            legs: NAVY,
            boots: CHARCOAL,
        },
    );
    body::tunic(&mut v, &p, coat, OILSKIN_SHADE, DARK_LEATHER);
    clothing(&mut v, &p, f, coat);
    head(&mut v, &p, f);
    creel(&mut v, f);
    rod(&mut v, f);
    // Buttons down the oilskin and a leather neckerchief.
    for y in [0.12, 0.22, 0.32] {
        v.push(part(
            Body(Chest),
            Box(0.25, 0.23, 0.045),
            NAVY,
            [0.0, 0.15, 0.145],
        ));
        for side in [-1.0, 1.0] {
            v.push(part(
                Body(Chest),
                Box(0.032, 0.13, 0.032),
                LINEN,
                [side * 0.09, 0.29, 0.15],
            ));
        }
    }
    for y in [0.10, 0.19, 0.28] {
        v.push(part(Body(Chest), Ball(0.019), BRASS, [0.0, y, 0.174]));
    }
    // A fitted neckerchief knot and tails preserve the team marker as clothing.
    v.extend([
        part(
            Body(Chest),
            Box(0.16, 0.045, 0.05),
            LINEN,
            [0.0, p.torso.y, 0.11],
        ),
        part(
            Body(Chest),
            Ball(0.032),
            LINEN,
            [0.0, p.torso.y - 0.025, 0.15],
        ),
        part(
            Body(Chest),
            Box(0.035, 0.085, 0.018),
            LINEN,
            [-0.025, p.torso.y - 0.065, 0.16],
        )
        .rot([0.0, 0.0, -18.0]),
        part(
            Body(Chest),
            Box(0.035, 0.07, 0.018),
            LINEN,
            [0.025, p.torso.y - 0.065, 0.16],
        )
        .rot([0.0, 0.0, 18.0]),
    ]);
    for (arm, fore, shin, foot) in [
        (UpperArmL, ForearmL, ShinL, FootL),
        (UpperArmR, ForearmR, ShinR, FootR),
    ] {
        v.push(part(Body(arm), Ball(0.075), coat, [0.0, -0.015, 0.0]).scale([1.0, 0.75, 1.05]));
        v.push(part(
            Body(fore),
            Cyl(0.062, 0.05),
            OILSKIN_SHADE,
            [0.0, -p.arm * 0.80, 0.0],
        ));
        v.push(part(
            Body(shin),
            Cyl(0.077, 0.045),
            DARK,
            [0.0, -0.045, 0.0],
        ));
        v.push(part(Body(foot), Ball(0.08), CHARCOAL, [0.0, -0.02, 0.11]).scale([0.9, 0.52, 1.15]));
    }

    if f.a {
        for hand in [HandL, HandR] {
            v.push(part(
                Body(hand),
                Box(0.11, 0.08, 0.125),
                RED_CLOTH,
                [0.0, -0.01, 0.0],
            ));
        }
    }
}

fn head(v: &mut Vec<PartSpec>, p: &Proportions, f: Form) {
    let h = p.head;
    v.push(
        part(
            Body(Head),
            Ball(h * 0.72),
            IVORY,
            [0.0, -h * 0.52, h * 0.66],
        )
        .scale([1.0, 0.7, 0.65]),
    );
    for side in [-1.0, 1.0] {
        v.push(
            part(
                Body(Head),
                Cone(h * 0.39, 0.18),
                IVORY,
                [side * 0.04, -0.15, h * 0.69],
            )
            .rot([172.0, 0.0, side * 12.0]),
        );
        v.push(
            part(
                Body(Head),
                Ball(h * 0.38),
                LINEN,
                [side * h * 0.43, -h * 0.30, h * 0.95],
            )
            .scale([1.15, 0.40, 0.55]),
        );
        v.push(
            part(
                Body(Head),
                Box(h * 0.65, h * 0.16, h * 0.25),
                IVORY,
                [side * h * 0.40, h * 0.28, h * 0.88],
            )
            .rot([0.0, 0.0, side * -9.0]),
        );
    }
    v.extend([
        part(
            Body(Head),
            Ball(h * 0.25),
            MUSHROOM,
            [0.0, -h * 0.02, h * 1.02],
        ),
        part(
            Body(Head),
            Cyl(0.012, 0.13),
            LINEN,
            [h * 0.66, -h * 0.38, h * 1.10],
        )
        .rot([80.0, 0.0, 0.0]),
        part(
            Body(Head),
            Cyl(0.031, 0.055),
            WOOD,
            [h * 0.66, -h * 0.31, h * 1.67],
        ),
        part(
            Body(Head),
            Ball(h * 1.10),
            OILSKIN_YELLOW,
            [0.0, h * 0.32, -h * 0.10],
        )
        .scale([1.08, 0.65, 1.06]),
        part(
            Body(Head),
            Cyl(h * 1.64, 0.025),
            OILSKIN_SHADE,
            [0.0, h * 0.24, -h * 0.25],
        )
        .rot([-9.0, 0.0, 0.0]),
        part(
            Body(Head),
            Cyl(h * 1.61, 0.025),
            OILSKIN_YELLOW,
            [0.0, h * 0.31, -h * 0.25],
        )
        .rot([-9.0, 0.0, 0.0]),
        part(
            Body(Head),
            Box(h * 1.85, 0.09, 0.035),
            OILSKIN_YELLOW,
            [0.0, -h * 0.08, -h * 1.38],
        )
        .rot([18.0, 0.0, 0.0]),
    ]);
    if f.b {
        v.push(part(
            Body(Head),
            Cyl(h * 1.13, 0.025),
            NAVY,
            [0.0, h * 0.43, -h * 0.10],
        ));
    }
    if f.b && f.t2 {
        v.push(
            part(
                Body(Head),
                Ring(0.009, 0.04, 270.0),
                GOLD,
                [h * 0.65, h * 0.58, 0.0],
            )
            .rot([0.0, 0.0, 90.0]),
        );
    }
}

/// Upright catches show a broad silver body and forked tail above the basket rim.
fn catch(v: &mut Vec<PartSpec>, at: [f32; 3], size: f32) {
    let node = Secondary(Spring::Back);
    let at = Vec3::from(at);
    for (shape, color, offset, scale, roll) in [
        (Ball(0.065), FISH_SILVER, Vec3::ZERO, [0.7, 1.5, 0.75], 0.0),
        (
            Ball(0.05),
            FISH_BACK,
            Vec3::new(0.0, 0.01, -0.028),
            [0.8, 1.65, 0.45],
            0.0,
        ),
        (
            Cone(0.04, 0.08),
            FISH_BACK,
            Vec3::new(-0.025, 0.11, 0.0),
            [0.8, 1.0, 0.25],
            -25.0,
        ),
        (
            Cone(0.04, 0.08),
            FISH_BACK,
            Vec3::new(0.025, 0.11, 0.0),
            [0.8, 1.0, 0.25],
            25.0,
        ),
    ] {
        v.push(
            part(node, shape, color, (at + offset * size).to_array())
                .scale([scale[0] * size, scale[1] * size, scale[2] * size])
                .rot([0.0, 0.0, roll]),
        );
    }
}

fn creel(v: &mut Vec<PartSpec>, f: Form) {
    let b = Secondary(Spring::Back);
    let size = if f.a { 1.25 } else { 1.0 };
    v.extend([
        part(
            b,
            Box(0.27 * size, 0.25 * size, 0.17),
            TIMBER,
            [0.06, -0.05, -0.09],
        ),
        part(
            b,
            Box(0.29 * size, 0.035, 0.185),
            TRIM,
            [0.06, 0.08 * size, -0.09],
        ),
        part(
            b,
            Box(0.035, 0.27 * size, 0.18),
            DARK_LEATHER,
            [0.06, -0.05, -0.09],
        ),
    ]);
    for y in [-0.12, -0.04, 0.04] {
        v.push(part(
            b,
            Box(0.28 * size, 0.018, 0.185),
            TRIM,
            [0.06, y * size, -0.09],
        ));
    }
    catch(v, [0.13, 0.12, -0.09], 1.0);
    if f.a {
        catch(v, [-0.06, 0.16, -0.09], 1.25);
    }
    if f.a && f.t2 {
        v.push(
            part(b, Ball(0.13), FISH_BACK, [0.1, 0.20, -0.04])
                .scale([0.8, 0.8, 2.6])
                .rot([0.0, 30.0, 65.0]),
        );
        v.push(part(b, Ball(0.095), FISH_SILVER, [0.19, 0.29, 0.06]).scale([0.85, 0.7, 1.4]));
        for side in [-1.0, 1.0] {
            v.push(
                part(
                    b,
                    Box(0.012, 0.012, 0.17),
                    LINEN,
                    [0.19 + side * 0.04, 0.29, 0.16],
                )
                .rot([12.0, side * 28.0, 0.0]),
            );
        }
    }
    if f.b {
        v.push(part(b, Ball(0.13), LINEN, [0.0, -0.23, -0.12]).scale([1.4, 0.75, 0.8]));
        for side in [-1.0, 1.0] {
            v.push(
                part(
                    b,
                    Box(0.018, 0.19, 0.018),
                    DARK_LEATHER,
                    [side * 0.075, -0.23, -0.22],
                )
                .rot([0.0, 0.0, side * 25.0]),
            );
        }
    }
    if f.b && f.t2 {
        for (x, y) in [(-0.09, -0.18), (0.07, -0.18), (0.0, -0.28)] {
            catch(v, [x, y, -0.20], 0.65);
        }
        v.extend(gear::lantern(b, [-0.18, 0.04, -0.02], 1.35));
    }
}

fn rod(v: &mut Vec<PartSpec>, f: Form) {
    let w = Prop(Socket::WeaponR);
    let wood = if f.a && f.t2 { OAK } else { WOOD };
    v.extend([
        part(w, Cyl(0.022, ROD * 0.55), wood, [0.0, ROD * 0.2, 0.0]),
        part(w, Cyl(0.014, ROD * 0.45), wood, [0.0, ROD * 0.6, 0.0]),
        part(w, Cyl(0.032, 0.18), TRIM, [0.0, -0.015, 0.0]),
        part(w, Cyl(0.014, 0.14), BRASS, [0.075, 0.045, 0.0]).rot([0.0, 0.0, 90.0]),
        part(w, Cyl(0.063, 0.035), BRASS, [0.16, 0.045, 0.0]).rot([0.0, 0.0, 90.0]),
        part(w, Cyl(0.043, 0.04), DARK_LEATHER, [0.162, 0.045, 0.0]).rot([0.0, 0.0, 90.0]),
        part(w, Ball(0.022), WOOD, [0.192, 0.085, 0.0]),
        part(
            w,
            Box(0.005, 0.25, 0.005),
            LINEN,
            [0.0, TIP.y - 0.12, 0.025],
        ),
        part(w, Ball(0.035), RED_CLOTH, [0.0, TIP.y - 0.26, 0.025]),
    ]);
    if f.a && f.t2 {
        v.push(
            part(w, Ring(0.009, 0.04, 270.0), GOLD, [0.0, TIP.y - 0.3, 0.025])
                .rot([0.0, 0.0, 90.0])
                .glow(),
        );
    }
    if f.b {
        v.push(
            part(
                Secondary(Spring::Back),
                Cyl(0.015, 0.8),
                WOOD,
                [0.18, 0.1, -0.14],
            )
            .rot([0.0, 0.0, -30.0]),
        );
    }
}
