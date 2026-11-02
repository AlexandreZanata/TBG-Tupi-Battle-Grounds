//! Hook Brawler's bandana, split coat, hanging hooks, rope and chain Evolution families.
use super::common::*;
pub fn plan(t: u8) -> Proportions {
    super::common::plan(t, 1.12, 0.43, 0.43)
}
pub fn spec(t: u8, b: u8) -> FigureSpec {
    let p = plan(t);
    let f = Form::of(t, b);
    let coat = if f.b { CHARCOAL } else { FIELD_SLATE };
    let mut v = ogre(&p, OGRE_SKIN, coat);
    v.push(
        part(
            Body(Head),
            Frustum(0.13, 0.15, 0.055),
            LINEN,
            [0.0, 0.12, 0.0],
        )
        .scale([1.0, 1.0, 0.88]),
    );
    for s in [-1.0f32, 1.0] {
        v.push(
            part(
                Secondary(Spring::HeadGear),
                Prism(0.036, 0.25),
                LINEN,
                [s * 0.10, 0.11, -0.17],
            )
            .scale([1.0, 1.0, 0.26])
            .rot([-65.0, s * 20.0, s * 12.0]),
        );
        v.push(
            part(
                Body(Chest),
                Bevel(0.085, 0.16, 0.055, 0.012),
                coat,
                [s * 0.17, 0.29, 0.02],
            )
            .rot([-20.0, 0.0, -s * 22.0]),
        );
        v.push(
            part(
                Body(Hips),
                Prism(0.12, 0.30),
                coat,
                [s * 0.15, -0.13, -0.01],
            )
            .scale([1.0, 1.0, 0.28])
            .rot([0.0, 0.0, -s * 8.0]),
        );
    }
    for (n, left) in [
        (Prop(Socket::WeaponR), false),
        (Prop(Socket::OffhandL), true),
    ] {
        if !left || f.b {
            v.push(part(
                n,
                Bevel(0.075, 0.21, 0.075, 0.02),
                if f.a { STEEL } else { GUNMETAL },
                [0.0, -0.07, 0.015],
            ));
            v.push(part(
                n,
                Bevel(0.045, 0.24, 0.038, 0.012),
                STEEL,
                [0.0, -0.24, 0.04],
            ));
            v.push(
                part(
                    n,
                    Ring(0.024, if f.t2 { 0.135 } else { 0.105 }, 225.0),
                    STEEL,
                    [0.08, -0.34, 0.04],
                )
                .scale([0.80, 1.0, 1.0])
                .rot([90.0, 0.0, -30.0]),
            );
            if f.t2 {
                v.push(
                    part(n, Prism(0.026, 0.24), TURQUOISE, [0.014, -0.24, 0.065])
                        .scale([1.0, 1.0, 0.4])
                        .glow(),
                );
            }
        }
    }
    for y in [-0.02, -0.075, -0.13] {
        v.push(part(
            Body(ForearmL),
            Ring(0.012, 0.082, 360.0),
            IRON,
            [0.0, y, 0.0],
        ));
    }
    if f.a || f.b {
        for i in 0..4 {
            let x = -0.14 + i as f32 * 0.085;
            v.push(
                part(
                    Body(Chest),
                    Ring(0.014, 0.038, 360.0),
                    if f.a { LINEN } else { IRON },
                    [x, 0.08 + i as f32 * 0.058, 0.125],
                )
                .scale([1.0, 1.0, 0.65])
                .rot([90.0, 0.0, -25.0]),
            );
        }
        v.push(part(
            Body(UpperArmL),
            Bevel(0.22, 0.10, 0.21, 0.03),
            LEATHER,
            [0.0, 0.01, 0.0],
        ));
    }
    if f.b {
        for x in [-0.10, 0.0, 0.10] {
            v.push(
                part(
                    Body(Hips),
                    Ring(0.012, 0.045, 360.0),
                    IRON,
                    [x, -0.045, 0.15],
                )
                .rot([90.0, 0.0, 0.0]),
            );
        }
    }
    if f.t2 {
        v.push(
            part(
                Body(UpperArmL),
                Prism(0.07, 0.13),
                TURQUOISE,
                [0.0, 0.09, 0.035],
            )
            .rot([0.0, 30.0, 0.0])
            .glow(),
        );
    }
    FigureSpec { plan: p, parts: v }
}
