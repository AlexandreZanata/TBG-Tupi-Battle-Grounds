//! Clay Binder: five native forms with its own equipment and silhouette.
use super::common::*;
pub fn plan(t: u8) -> Proportions {
    super::common::plan(t, 1.13, 0.37, 0.43)
}
pub fn spec(t: u8, b: u8) -> FigureSpec {
    let p = plan(t);
    let f = Form::of(t, b);
    let mut v = ogre(&p, OGRE_SKIN, cloth(f, LINEN));

    v.push(
        part(
            Body(Head),
            Cyl(0.14, 0.035),
            if f.a {
                IVORY
            } else if f.b {
                CHARCOAL
            } else {
                AMBER
            },
            [0.0, 0.0, 0.14],
        )
        .rot([90.0, 0.0, 0.0]),
    );
    for x in [-0.045, 0.045] {
        v.push(part(
            Body(Head),
            Box(0.025, 0.04, 0.014),
            DARK,
            [x, 0.02, 0.163],
        ));
    }
    v.push(part(
        Secondary(Spring::Back),
        Frustum(0.12, 0.16, if f.t2 { 0.55 } else { 0.38 }),
        if f.b { AMBER } else { RUST },
        [0.0, 0.20, -0.06],
    ));
    v.push(part(
        Secondary(Spring::Back),
        Ring(0.022, 0.12, 360.0),
        BRASS,
        [0.0, if f.t2 { 0.48 } else { 0.40 }, -0.06],
    ));
    v.push(
        part(
            Prop(Socket::WeaponR),
            Cyl(0.035, 0.34),
            ROCK,
            [0.0, 0.0, 0.14],
        )
        .rot([90.0, 0.0, 12.0]),
    );
    v.push(part(
        Prop(Socket::OffhandL),
        Ball(if f.t2 { 0.10 } else { 0.07 }),
        if f.b { AMBER } else { RUST },
        [0.0, 0.15, 0.12],
    ));
    tier(&mut v, f, [0.0, 0.28, 0.15], Cone(0.065, 0.14));
    FigureSpec { plan: p, parts: v }
}
