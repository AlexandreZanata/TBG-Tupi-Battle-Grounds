//! ReefLancer equipment and five authored shell Evolution forms.
use super::common::*;
pub fn spec(t: u8, b: u8) -> FigureSpec {
    let p = plan(ReefLancer, t);
    let f = Form::of(t, b);
    let mut v = marine(ReefLancer, &p, f);

    let n = Prop(Socket::WeaponR);
    let color = if f.b { PLUM } else { IVORY };
    v.push(part(
        n,
        Cyl(0.021, if f.a { 0.93 } else { 0.78 }),
        OAK,
        [0.0, 0.16, 0.03],
    ));
    for x in [-0.10, 0.0, 0.10] {
        v.push(
            part(n, Cone(0.035, 0.21), color, [x, 0.58 + x.abs() * 0.4, 0.03]).rot([
                0.0,
                0.0,
                -x * 110.0,
            ]),
        );
    }
    v.push(part(
        n,
        Bevel(0.23, 0.035, 0.05, 0.012),
        color,
        [0.0, 0.49, 0.03],
    ));
    shell(
        &mut v,
        Body(UpperArmL),
        [0.0, 0.0, 0.025],
        0.13,
        if f.b { PLUM } else { RUST },
    );
    finish(&mut v, f, n, [0.0, 0.49, 0.07]);
    FigureSpec { plan: p, parts: v }
}
