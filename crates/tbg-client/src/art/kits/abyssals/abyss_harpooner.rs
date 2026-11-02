//! AbyssHarpooner equipment and five authored shell Evolution forms.
use super::common::*;
use crate::art::rig::{Node::Secondary, Spring};
pub fn spec(t: u8, b: u8) -> FigureSpec {
    let p = plan(AbyssHarpooner, t);
    let f = Form::of(t, b);
    let mut v = marine(AbyssHarpooner, &p, f);

    let n = Prop(Socket::WeaponR);
    let c = if f.b { PLUM } else { IVORY };
    v.push(part(
        n,
        Bevel(0.13, 0.10, 0.42, 0.025),
        OAK,
        [0.0, 0.0, 0.10],
    ));
    v.push(
        part(
            n,
            Cyl(0.015, if f.a { 0.62 } else { 0.52 }),
            IVORY,
            [0.0, 0.065, 0.24],
        )
        .rot([90.0, 0.0, 0.0]),
    );
    v.push(
        part(
            n,
            Cone(0.044, 0.18),
            c,
            [0.0, 0.065, if f.a { 0.61 } else { 0.51 }],
        )
        .rot([90.0, 0.0, 0.0]),
    );
    for s in [-1.0f32, 1.0] {
        v.push(part(n, Prism(0.055, 0.035), c, [s * 0.04, 0.06, 0.47]).rot([90.0, 0.0, s * 25.0]));
    }
    if f.b {
        for x in [-0.08, 0.08] {
            v.push(
                part(
                    Secondary(Spring::Back),
                    Cone(0.03, 0.22),
                    IVORY,
                    [x, 0.27, -0.04],
                )
                .rot([0.0, 0.0, x * 90.0]),
            );
        }
    }
    body::cape(&mut v, PLUM, 0.25, 0.37);
    finish(&mut v, f, n, [0.0, 0.08, 0.27]);
    FigureSpec { plan: p, parts: v }
}
