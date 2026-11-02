//! Flint Runner: five native forms with its own equipment and silhouette.
use super::common::*;
pub fn plan(t: u8) -> Proportions {
    super::common::plan(t, 1.08, 0.48, 0.34)
}
pub fn spec(t: u8, b: u8) -> FigureSpec {
    let p = plan(t);
    let f = Form::of(t, b);
    let mut v = ogre(&p, SKIN, cloth(f, RUST));

    for n in [Prop(Socket::WeaponR), Prop(Socket::OffhandL)] {
        v.push(part(n, Cyl(0.022, 0.12), OAK, [0.0, 0.0, 0.0]).rot([90.0, 0.0, 0.0]));
        v.push(
            part(
                n,
                Bevel(
                    if f.a { 0.09 } else { 0.07 },
                    0.035,
                    if f.a { 0.42 } else { 0.27 },
                    0.018,
                ),
                if f.a { IVORY } else { CHARCOAL },
                [0.0, 0.0, if f.a { 0.26 } else { 0.18 }],
            )
            .rot([0.0, 12.0, 0.0]),
        );
        if f.b {
            v.push(part(n, Box(0.015, 0.04, 0.27), AMBER, [0.03, 0.0, 0.18]).glow());
        }
    }
    for z in [-0.08, 0.0, 0.08] {
        v.push(
            part(Body(Head), Cone(0.065, 0.22), OLD_BONE, [0.0, 0.15, z]).rot([-35.0, 0.0, 0.0]),
        );
    }
    v.push(part(
        Body(Chest),
        Bevel(0.37, 0.07, 0.25, 0.02),
        if f.a { LINEN } else { RUST },
        [0.0, 0.28, 0.0],
    ));
    tier(&mut v, f, [0.0, 0.34, -0.10], Cone(0.06, 0.16));
    FigureSpec { plan: p, parts: v }
}
