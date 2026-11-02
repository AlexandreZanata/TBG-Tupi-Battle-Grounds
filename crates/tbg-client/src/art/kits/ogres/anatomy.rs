//! Faceted Ogre muscle, face and fitted clothing shared by the eight new organic kits.
use super::common::*;

pub fn parts(p: &Proportions, skin: Color, cloth: Color) -> Vec<PartSpec> {
    let t = p.torso;
    let mut v = vec![
        part(
            Body(Hips),
            Bevel(t.x * 0.70, 0.15, t.z * 0.95, 0.025),
            DARK_LEATHER,
            [0.0, 0.0, 0.0],
        ),
        part(
            Body(Chest),
            Frustum(t.x * 0.53, t.x * 0.33, t.y),
            skin,
            [0.0, t.y * 0.5, 0.0],
        )
        .scale([1.0, 1.0, 0.69]),
        part(Body(Head), Ball(p.head), skin, [0.0, 0.01, -0.005]).scale([1.0, 1.12, 0.92]),
        part(
            Body(Head),
            Bevel(0.255, 0.145, 0.20, 0.04),
            skin,
            [0.0, -0.065, 0.065],
        ),
        part(
            Body(Head),
            Bevel(0.07, 0.07, 0.08, 0.02),
            skin,
            [0.0, 0.025, 0.14],
        ),
        part(
            Body(Head),
            Bevel(0.15, 0.018, 0.018, 0.006),
            DARK,
            [0.0, -0.065, 0.172],
        ),
        part(
            Body(Hips),
            Bevel(t.x * 0.83, 0.045, t.z * 1.08, 0.014),
            LEATHER,
            [0.0, 0.025, 0.0],
        ),
        part(
            Body(Hips),
            Bevel(0.05, 0.05, 0.022, 0.01),
            BRASS,
            [0.0, 0.025, t.z * 0.56],
        ),
    ];
    for s in [-1.0f32, 1.0] {
        v.push(
            part(
                Body(Chest),
                Ball(t.x * 0.265),
                skin,
                [s * t.x * 0.24, t.y * 0.71, 0.065],
            )
            .scale([1.0, 0.70, 0.66]),
        );
        v.push(
            part(
                Body(Chest),
                Bevel(t.x * 0.22, t.y * 0.79, 0.045, 0.012),
                cloth,
                [s * t.x * 0.40, t.y * 0.50, 0.09],
            )
            .rot([0.0, 0.0, s * 12.0]),
        );
        v.push(
            part(
                Body(Hips),
                Prism(0.10, 0.23),
                cloth,
                [s * 0.105, -0.12, 0.10],
            )
            .scale([1.0, 1.0, 0.27])
            .rot([0.0, 0.0, s * 12.0]),
        );
        v.push(
            part(
                Body(Head),
                Bevel(0.085, 0.035, 0.065, 0.012),
                skin,
                [s * 0.065, 0.063, 0.125],
            )
            .rot([0.0, 0.0, s * 14.0]),
        );
        v.push(part(
            Body(Head),
            Bevel(0.032, 0.021, 0.014, 0.004),
            AMBER,
            [s * 0.062, 0.035, 0.153],
        ));
        v.push(part(
            Body(Head),
            Box(0.012, 0.019, 0.008),
            DARK,
            [s * 0.063, 0.035, 0.162],
        ));
        v.push(
            part(
                Body(Head),
                Cone(0.024, 0.09),
                IVORY,
                [s * 0.083, -0.045, 0.19],
            )
            .rot([0.0, 0.0, s * 12.0]),
        );
        v.push(
            part(
                Body(Head),
                Cone(0.049, 0.14),
                skin,
                [s * 0.175, 0.02, -0.01],
            )
            .rot([0.0, 0.0, -s * 72.0]),
        );
    }
    for (arm, fore, hand, thigh, shin, foot) in [
        (UpperArmL, ForearmL, HandL, ThighL, ShinL, FootL),
        (UpperArmR, ForearmR, HandR, ThighR, ShinR, FootR),
    ] {
        v.extend([
            part(Body(arm), Ball(0.105), skin, [0.0, -0.015, 0.0]).scale([1.0, 0.85, 1.0]),
            part(
                Body(arm),
                Frustum(0.105, 0.075, p.arm),
                skin,
                [0.0, -p.arm * 0.5, 0.0],
            ),
            part(
                Body(fore),
                Frustum(0.084, 0.061, p.arm * 0.9),
                skin,
                [0.0, -p.arm * 0.45, 0.0],
            ),
            part(
                Body(hand),
                Bevel(0.115, 0.09, 0.12, 0.025),
                skin,
                [0.0, -0.025, 0.015],
            ),
            part(
                Body(thigh),
                Frustum(0.095, 0.075, p.leg * 0.5),
                cloth,
                [0.0, -p.leg * 0.25, 0.0],
            )
            .scale([1.0, 1.0, 0.88]),
            part(
                Body(shin),
                Frustum(0.07, 0.048, p.leg * 0.5),
                skin,
                [0.0, -p.leg * 0.25, 0.0],
            ),
            part(
                Body(foot),
                Bevel(0.15, 0.075, 0.23, 0.024),
                DARK_LEATHER,
                [0.0, -0.02, 0.045],
            ),
        ]);
        for y in [-0.04, -0.09] {
            v.push(part(Body(fore), Cyl(0.085, 0.018), LEATHER, [0.0, y, 0.0]));
            v.push(part(Body(shin), Cyl(0.066, 0.018), LINEN, [0.0, y, 0.0]));
        }
        for x in [-0.04, 0.0, 0.04] {
            v.push(part(
                Body(foot),
                Bevel(0.035, 0.039, 0.052, 0.008),
                skin,
                [x, -0.024, 0.16],
            ));
            v.push(part(
                Body(hand),
                Bevel(0.025, 0.035, 0.055, 0.007),
                skin,
                [x * 0.8, -0.06, 0.033],
            ));
        }
    }
    v
}
