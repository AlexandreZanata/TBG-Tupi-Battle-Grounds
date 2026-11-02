//! Pebble Slinger: five native forms with its own equipment and silhouette.
use super::common::*;
pub fn plan(t: u8) -> Proportions {
    super::common::plan(t, 1.02, 0.34, 0.36)
}
pub fn spec(t: u8, b: u8) -> FigureSpec {
    let p = plan(t);
    let f = Form::of(t, b);
    let mut v = ogre(&p, Color::srgb(0.48, 0.51, 0.26), cloth(f, OILSKIN_YELLOW));

    for x in [-0.09, 0.09] {
        v.push(
            part(
                Prop(Socket::WeaponR),
                Cyl(0.022, if f.a { 0.30 } else { 0.22 }),
                if f.a { IVORY } else { OAK },
                [x, 0.13, 0.0],
            )
            .rot([0.0, 0.0, x * 160.0]),
        );
    }
    v.push(part(
        Prop(Socket::WeaponR),
        Cyl(0.03, 0.15),
        OAK,
        [0.0, -0.06, 0.0],
    ));
    v.push(part(
        Prop(Socket::WeaponR),
        Box(0.23, 0.015, 0.025),
        DARK_LEATHER,
        [0.0, 0.25, 0.0],
    ));
    v.push(part(
        Prop(Socket::OffhandL),
        Ball(0.09),
        LEATHER,
        [0.0, 0.0, 0.04],
    ));
    v.push(part(
        Secondary(Spring::Back),
        Bevel(0.21, 0.24, 0.13, 0.025),
        LINEN,
        [0.0, 0.03, -0.05],
    ));
    if f.b {
        for x in [-0.055, 0.055] {
            v.push(part(
                Prop(Socket::WeaponR),
                Ball(0.04),
                CHARCOAL,
                [x, 0.24, 0.03],
            ));
        }
    }
    tier(&mut v, f, [0.0, 0.29, 0.0], Cone(0.06, 0.15));
    FigureSpec { plan: p, parts: v }
}
