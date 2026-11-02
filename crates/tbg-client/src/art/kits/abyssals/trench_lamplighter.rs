//! TrenchLamplighter equipment and five authored shell Evolution forms.
use super::common::*;
pub fn spec(t: u8, b: u8) -> FigureSpec {
    let p = plan(TrenchLamplighter, t);
    let f = Form::of(t, b);
    let mut v = marine(TrenchLamplighter, &p, f);

    let n = Prop(Socket::WeaponR);
    let c = if f.b { PLUM } else { IVORY };
    v.push(part(n, Cyl(0.022, 0.67), OAK, [0.0, 0.14, 0.0]));
    v.push(
        part(
            n,
            Ball(0.095),
            if f.t2 && f.b { FOAM } else { AMBER },
            [0.0, 0.43, 0.0],
        )
        .scale([0.75, 1.25, 0.75])
        .glow(),
    );
    for x in [-0.08, 0.08] {
        v.push(part(n, Cyl(0.012, 0.23), c, [x, 0.43, 0.0]).rot([0.0, 0.0, x * 70.0]));
    }
    for z in [-0.08, 0.08] {
        v.push(part(n, Cyl(0.012, 0.23), c, [0.0, 0.43, z]).rot([z * 70.0, 0.0, 0.0]));
    }
    for y in [0.31, 0.55] {
        v.push(part(n, Ring(0.014, 0.075, 360.0), c, [0.0, y, 0.0]));
    }
    if f.b {
        v.push(
            part(n, Prism(0.085, 0.23), PLUM, [-0.11, 0.49, 0.0])
                .scale([0.4, 1.0, 0.7])
                .rot([0.0, 0.0, -30.0]),
        );
    }
    if f.a {
        v.push(part(n, Cone(0.08, 0.14), IVORY, [0.0, 0.60, 0.0]));
    }
    body::cape(&mut v, if f.a { IVORY } else { PLUM }, 0.23, 0.47);
    FigureSpec { plan: p, parts: v }
}
