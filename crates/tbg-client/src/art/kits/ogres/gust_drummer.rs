//! Gust Drummer: five native forms with its own equipment and silhouette.
use super::common::*;
pub fn plan(t: u8) -> Proportions {
    super::common::plan(t, 1.14, 0.35, 0.46)
}
pub fn spec(t: u8, b: u8) -> FigureSpec {
    let p = plan(t);
    let f = Form::of(t, b);
    let mut v = ogre(&p, OGRE_SKIN, cloth(f, DARK_LEATHER));

    v.push(part(
        Body(Chest),
        Cyl(if f.t2 { 0.23 } else { 0.19 }, 0.36),
        if f.a { LINEN } else { OAK },
        [0.0, 0.0, 0.28],
    ));
    for y in [-0.17, 0.17] {
        v.push(part(
            Body(Chest),
            Ring(0.025, if f.t2 { 0.23 } else { 0.19 }, 360.0),
            BRASS,
            [0.0, y, 0.28],
        ));
    }
    v.push(part(
        Body(Chest),
        Cyl(0.18, 0.018),
        if f.b { CHARCOAL } else { LINEN },
        [0.0, 0.18, 0.28],
    ));
    for n in [Prop(Socket::WeaponR), Prop(Socket::OffhandL)] {
        v.push(part(n, Cyl(0.018, 0.20), OAK, [0.0, 0.0, 0.10]).rot([90.0, 0.0, 0.0]));
        v.push(part(n, Ball(0.055), BONE, [0.0, 0.0, 0.22]));
    }
    if f.a || f.b {
        for x in [-0.10, 0.10] {
            v.push(
                part(
                    Secondary(Spring::Back),
                    Cyl(0.055, if f.t2 { 0.47 } else { 0.30 }),
                    if f.a { IVORY } else { BRASS },
                    [x, 0.22, 0.0],
                )
                .rot([-15.0, 0.0, 0.0]),
            );
        }
    }
    v.push(part(Body(Head), Cone(0.12, 0.19), IVORY, [0.0, 0.14, -0.01]).rot([-40.0, 0.0, 0.0]));
    tier(&mut v, f, [0.0, 0.32, 0.14], Ring(0.015, 0.065, 360.0));
    FigureSpec { plan: p, parts: v }
}
