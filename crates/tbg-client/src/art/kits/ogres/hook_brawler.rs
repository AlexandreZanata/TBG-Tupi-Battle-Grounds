//! Hook Brawler: five native forms with its own equipment and silhouette.
use super::common::*;
pub fn plan(t: u8) -> Proportions {
    super::common::plan(t, 1.10, 0.41, 0.39)
}
pub fn spec(t: u8, b: u8) -> FigureSpec {
    let p = plan(t);
    let f = Form::of(t, b);
    let mut v = ogre(&p, OGRE_SKIN, cloth(f, FIELD_SLATE));

    v.push(part(
        Body(Head),
        Box(0.29, 0.055, 0.26),
        LINEN,
        [0.0, 0.11, 0.0],
    ));
    for n in [Prop(Socket::WeaponR), Prop(Socket::OffhandL)] {
        v.push(part(
            n,
            Bevel(0.12, 0.11, 0.15, 0.02),
            IRON,
            [0.0, 0.0, 0.04],
        ));
        if matches!(n, Prop(Socket::WeaponR)) || f.b {
            v.push(
                part(
                    n,
                    Ring(0.025, if f.t2 { 0.15 } else { 0.11 }, 210.0),
                    if f.a { STEEL } else { GUNMETAL },
                    [0.0, 0.0, 0.15],
                )
                .rot([90.0, 0.0, 0.0]),
            );
        }
    }
    for y in [0.0, -0.06, -0.12] {
        v.push(part(
            Body(ForearmL),
            Ring(0.012, 0.07, 360.0),
            if f.a { LINEN } else { IRON },
            [0.0, y, 0.0],
        ));
    }
    tier(&mut v, f, [0.0, 0.23, 0.14], Ring(0.018, 0.065, 360.0));
    FigureSpec { plan: p, parts: v }
}
