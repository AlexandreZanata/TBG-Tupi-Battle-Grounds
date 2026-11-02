//! Cinder Maul: five native forms with its own equipment and silhouette.
use super::common::*;
pub fn plan(t: u8) -> Proportions {
    super::common::plan(t, 1.13, 0.35, 0.44)
}
pub fn spec(t: u8, b: u8) -> FigureSpec {
    let p = plan(t);
    let f = Form::of(t, b);
    let mut v = ogre(&p, OGRE_SKIN, cloth(f, DARK_LEATHER));

    for n in [Prop(Socket::WeaponR), Prop(Socket::OffhandL)] {
        v.push(
            part(
                n,
                Cyl(0.025, if f.a { 0.43 } else { 0.25 }),
                OAK,
                [0.0, 0.0, 0.10],
            )
            .rot([90.0, 0.0, 0.0]),
        );
        v.push(part(
            n,
            Bevel(if f.a { 0.27 } else { 0.19 }, 0.13, 0.14, 0.025),
            if f.a { STEEL } else { GUNMETAL },
            [0.0, 0.0, if f.a { 0.34 } else { 0.21 }],
        ));
        if f.b {
            v.push(part(n, Box(0.025, 0.14, 0.16), ORANGE, [0.0, 0.0, 0.21]).glow());
        }
    }
    v.push(part(
        Secondary(Spring::Back),
        Box(0.22, 0.23, 0.13),
        OAK,
        [0.0, 0.06, -0.06],
    ));
    for x in [-0.06, 0.0, 0.06] {
        v.push(
            part(
                Secondary(Spring::Back),
                Ball(0.045),
                ORANGE,
                [x, 0.19, -0.06],
            )
            .glow(),
        );
    }
    v.push(part(
        Body(Head),
        Bevel(0.23, 0.08, 0.13, 0.02),
        CHARCOAL,
        [0.0, -0.08, 0.1],
    ));
    tier(&mut v, f, [0.0, 0.39, 0.0], Cone(0.07, 0.20));
    FigureSpec { plan: p, parts: v }
}
