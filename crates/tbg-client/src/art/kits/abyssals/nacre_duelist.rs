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
        v.push(
            part(
                n,
                Ring(0.032, if f.a { 0.18 } else { 0.14 }, 155.0),
                if f.b { PLUM } else { IVORY },
                [0.015, -0.19, 0.05],
            )
            .scale([0.5, 1.0, 1.0])
            .rot([90.0, 0.0, -35.0]),
        );
        if f.b {
            v.push(part(n, Prism(0.045, 0.20), PLUM, [0.0, -0.12, -0.045]).scale([1.0, 1.0, 0.3]));
        }
        finish(&mut v, f, n, [0.0, -0.12, 0.08]);
    }
    FigureSpec { plan: p, parts: v }
}
