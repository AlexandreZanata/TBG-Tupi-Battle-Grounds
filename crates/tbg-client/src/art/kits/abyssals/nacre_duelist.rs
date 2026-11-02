//! NacreDuelist equipment and five authored shell Evolution forms.
use super::common::*;
pub fn spec(t: u8, b: u8) -> FigureSpec {
    let p = plan(NacreDuelist, t);
    let f = Form::of(t, b);
    let mut v = marine(NacreDuelist, &p, f);

    for n in [Prop(Socket::WeaponR), Prop(Socket::OffhandL)] {
        v.push(part(
            n,
            Bevel(0.07, 0.13, 0.08, 0.02),
            LEATHER,
            [0.0, -0.02, 0.0],
        ));
        // Forward shell fins keep this duelist distinct from the Ogre's hanging hooks.
        let edge = if f.b { PLUM } else { IVORY };
        v.push(
            part(
                n,
                Prism(0.11, if f.a { 0.28 } else { 0.22 }),
                edge,
                [0.0, 0.08, 0.13],
            )
            .scale([0.28, 1.0, 1.0])
            .rot([52.0, 0.0, 0.0]),
        );
        v.push(
            part(
                n,
                Cone(0.075, if f.a { 0.19 } else { 0.15 }),
                edge,
                [0.0, 0.17, 0.26],
            )
            .scale([0.28, 1.0, 1.0])
            .rot([65.0, 0.0, 0.0]),
        );
        if f.b {
            v.push(part(n, Prism(0.045, 0.20), PLUM, [0.0, 0.04, -0.045]).scale([1.0, 1.0, 0.3]));
        }
        finish(&mut v, f, n, [0.0, 0.08, 0.11]);
    }
    FigureSpec { plan: p, parts: v }
}
