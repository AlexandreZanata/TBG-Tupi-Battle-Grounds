//! FoamWeaver equipment and five authored shell Evolution forms.
use super::common::*;
pub fn spec(t: u8, b: u8) -> FigureSpec {
    let p = plan(FoamWeaver, t);
    let f = Form::of(t, b);
    let mut v = marine(FoamWeaver, &p, f);

    let n = Prop(Socket::WeaponR);
    let c = if f.b { PLUM } else { IVORY };
    v.push(part(n, Frustum(0.15, 0.055, 0.085), c, [0.0, 0.0, 0.08]));
    for s in [-1.0f32, 1.0] {
        v.push(
            part(
                n,
                Prism(0.055, if f.a { 0.28 } else { 0.20 }),
                c,
                [s * 0.10, 0.09, 0.08],
            )
            .scale([0.65, 1.0, 0.4])
            .rot([0.0, 0.0, -s * 24.0]),
        );
    }
    v.push(
        part(
            n,
            Ball(if f.t2 { 0.065 } else { 0.045 }),
            if f.t2 && f.a { AMBER } else { PALE },
            [0.0, 0.15, 0.08],
        )
        .glow(),
    );
    body::cape(&mut v, PLUM, 0.24, 0.32);
    FigureSpec { plan: p, parts: v }
}
