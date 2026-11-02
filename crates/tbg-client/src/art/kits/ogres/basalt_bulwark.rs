//! Basalt Bulwark: five native forms with its own equipment and silhouette.
use super::common::*;
pub fn plan(t: u8) -> Proportions {
    super::common::plan(t, 1.20, 0.35, 0.48)
}
pub fn spec(t: u8, b: u8) -> FigureSpec {
    let p = plan(t);
    let f = Form::of(t, b);
    let mut v = ogre(&p, SKIN, cloth(f, DARK_LEATHER));

    v.push(part(
        Prop(Socket::OffhandL),
        Bevel(
            if f.a { 0.30 } else { 0.24 },
            if f.t2 { 0.76 } else { 0.60 },
            0.11,
            0.04,
        ),
        if f.b { CHARCOAL } else { ROCK },
        [0.0, 0.16, 0.07],
    ));
    for y in [0.0, 0.20, 0.40] {
        v.push(part(
            Prop(Socket::OffhandL),
            Box(0.25, 0.025, 0.13),
            if f.b { BRASS } else { IRON },
            [0.0, y, 0.07],
        ));
    }
    v.push(part(
        Prop(Socket::WeaponR),
        Bevel(0.16, 0.16, 0.19, 0.03),
        ROCK,
        [0.0, 0.0, 0.05],
    ));
    if f.a {
        v.push(part(
            Body(Head),
            Box(0.045, 0.17, 0.03),
            LINEN,
            [0.0, 0.0, 0.15],
        ));
    }
    if f.b {
        v.push(
            part(
                Prop(Socket::OffhandL),
                Box(0.025, 0.48, 0.015),
                AMBER,
                [0.0, 0.16, 0.135],
            )
            .glow(),
        );
    }
    tier(&mut v, f, [0.0, 0.34, 0.0], Cone(0.06, 0.14));
    FigureSpec { plan: p, parts: v }
}
