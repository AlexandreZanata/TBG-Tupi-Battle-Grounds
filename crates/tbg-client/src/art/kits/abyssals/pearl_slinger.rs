//! PearlSlinger equipment and five authored shell Evolution forms.
use super::common::*;
pub fn spec(t: u8, b: u8) -> FigureSpec {
    let p = plan(PearlSlinger, t);
    let f = Form::of(t, b);
    let mut v = marine(PearlSlinger, &p, f);

    let n = Prop(Socket::WeaponR);
    v.push(part(n, Cyl(0.023, 0.19), OAK, [0.0, -0.05, 0.0]));
    v.push(
        part(
            n,
            Ring(0.028, if f.a { 0.16 } else { 0.13 }, 210.0),
            if f.b { PLUM } else { IVORY },
            [0.0, 0.15, 0.0],
        )
        .scale([0.65, 1.0, 1.0])
        .rot([90.0, 0.0, 165.0]),
    );
    v.push(part(n, Ball(0.052), PALE, [0.0, 0.13, 0.0]));
    for x in [-0.17, 0.17] {
        v.push(part(Body(Hips), Ball(0.085), LEATHER, [x, -0.03, 0.025]));
        v.push(part(Body(Hips), Ball(0.042), IVORY, [x, 0.035, 0.055]));
    }
    body::cape(&mut v, if f.b { PLUM } else { RUST }, 0.25, 0.24);
    finish(&mut v, f, n, [0.0, 0.13, 0.04]);
    FigureSpec { plan: p, parts: v }
}
