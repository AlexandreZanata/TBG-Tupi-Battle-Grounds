//! Event-free signature dances, one short authored gesture per champion.
use super::*;
pub(super) fn apply(k: UnitKind, m: &Moment, p: &mut Pose, b: &crate::art::rig::Proportions) {
    match k {
        Gravecutter => gravecutter(m, p, b),
        LastlightCantor => lastlight_cantor(m, p, b),
        Cragclaw => cragclaw(m, p, b),
        Spinehurler => spinehurler(m, p, b),
        MireTender => mire_tender(m, p, b),
        Gorgehowl => gorgehowl(m, p, b),
        Sunlancer => sunlancer(m, p, b),
        EclipseDancer => eclipse_dancer(m, p, b),
        Starbreaker => starbreaker(m, p, b),
        Dawnmender => dawnmender(m, p, b),
        Prismancer => prismancer(m, p, b),
        Leafcutter => leafcutter(m, p, b),
        Blastbeetle => blastbeetle(m, p, b),
        Needlewing => needlewing(m, p, b),
        NectarNurse => nectar_nurse(m, p, b),
        Stridulator => stridulator(m, p, b),
        _ => {}
    }
}
fn gravecutter(m: &Moment, p: &mut Pose, _b: &crate::art::rig::Proportions) {
    let t = m.t * std::f32::consts::TAU / 3.0;
    let s = t.sin();
    let c = t.cos();
    let beat = (t * 3.0).sin();

    tools::set(p, HandR, [0.0, 30.0 * s, 20.0 * c]);
    tools::set(
        p,
        Chest,
        [10.0 * tools::hold(m.t, [1.7, 2.0, 2.3, 2.8]), 0.0, 0.0],
    );
    tools::set(p, FootL, [-10.0 * beat.max(0.0), 0.0, 0.0]);
}
fn lastlight_cantor(m: &Moment, p: &mut Pose, _b: &crate::art::rig::Proportions) {
    let t = m.t * std::f32::consts::TAU / 3.0;
    let s = t.sin();
    let c = t.cos();
    let beat = (t * 3.0).sin();

    tools::set(p, UpperArmR, [-35.0, 0.0, -10.0 + 18.0 * beat]);
    tools::set(p, HandR, [15.0 * beat, 0.0, 0.0]);
    tools::set(p, Chest, [8.0 * s.max(0.0), 0.0, 3.0 * c]);
}
fn cragclaw(m: &Moment, p: &mut Pose, _b: &crate::art::rig::Proportions) {
    let t = m.t * std::f32::consts::TAU / 3.0;
    let s = t.sin();
    let c = t.cos();
    let beat = (t * 3.0).sin();

    tools::set(p, UpperArmR, [-25.0 - 15.0 * beat.max(0.0), 0.0, -8.0]);
    tools::set(p, Chest, [0.0, 0.0, 5.0 * s]);
    tools::set(p, Head, [6.0 * c, 0.0, 0.0]);
}
fn spinehurler(m: &Moment, p: &mut Pose, b: &crate::art::rig::Proportions) {
    let t = m.t * std::f32::consts::TAU / 3.0;
    let s = t.sin();
    let c = t.cos();
    let beat = (t * 3.0).sin();

    tools::reach(
        p,
        false,
        Vec3::new(-0.12, 0.20, 0.17),
        Some(Quat::from_rotation_x(-1.5)),
        (b, Vec3::NEG_X),
    );
    tools::set(p, HandR, [-8.0 * s, 0.0, 6.0 * c]);
    p[Secondary(Spring::Back).index()].0.z = 5.0 * beat;
}
fn mire_tender(m: &Moment, p: &mut Pose, _b: &crate::art::rig::Proportions) {
    let t = m.t * std::f32::consts::TAU / 3.0;
    let s = t.sin();
    let beat = (t * 3.0).sin();

    tools::set(p, HandR, [0.0, 18.0 * beat, 8.0 * (t * 3.0).cos()]);
    tools::set(p, Head, [6.0 * beat, 0.0, 0.0]);
    tools::set(p, Chest, [0.0, 0.0, 3.0 * s]);
}
fn gorgehowl(m: &Moment, p: &mut Pose, _b: &crate::art::rig::Proportions) {
    let t = m.t * std::f32::consts::TAU / 3.0;
    let s = t.sin();
    let beat = (t * 3.0).sin();

    p[Secondary(Spring::HeadGear).index()].0.x = 10.0 * beat;
    tools::set(p, Head, [-5.0 * s, 0.0, 0.0]);
    tools::set(p, HandL, [0.0, 0.0, 18.0 * beat]);
}
fn sunlancer(m: &Moment, p: &mut Pose, b: &crate::art::rig::Proportions) {
    let t = m.t * std::f32::consts::TAU / 3.0;
    let s = t.sin();
    let beat = (t * 3.0).sin();
    shaft_pose(
        p,
        b,
        Vec3::new(0.0, 0.14 + 0.02 * s, 0.13),
        Quat::from_euler(EulerRot::XYZ, -1.05, 0.14 * s, 0.0),
        0.20,
    );
    tools::set(p, Head, [8.0 * s.max(0.0), 0.0, 0.0]);
    tools::set(p, FootR, [-8.0 * beat.max(0.0), 0.0, 0.0]);
}

fn eclipse_dancer(m: &Moment, p: &mut Pose, _b: &crate::art::rig::Proportions) {
    let t = m.t * std::f32::consts::TAU / 3.0;
    let s = t.sin();

    tools::set(p, UpperArmR, [-55.0 - 15.0 * s, 0.0, -12.0]);
    tools::set(p, UpperArmL, [-35.0 + 15.0 * s, 0.0, 12.0]);
    tools::set(p, HandR, [0.0, 20.0 * s, 0.0]);
    tools::set(p, HandL, [0.0, -20.0 * s, 0.0]);
    tools::set(p, Chest, [0.0, 10.0 * s, 0.0]);
}
fn starbreaker(m: &Moment, p: &mut Pose, b: &crate::art::rig::Proportions) {
    let s = (m.t * std::f32::consts::TAU / 3.0).sin();
    shaft_pose(
        p,
        b,
        Vec3::new(0.0, 0.12 + 0.06 * s.max(0.0), 0.13),
        Quat::from_rotation_x(-1.05 - 0.20 * s.max(0.0)),
        0.20,
    );
    tools::set(p, FootL, [-10.0 * s.max(0.0), 0.0, 0.0]);
    tools::set(p, FootR, [-10.0 * (-s).max(0.0), 0.0, 0.0]);
}

fn dawnmender(m: &Moment, p: &mut Pose, b: &crate::art::rig::Proportions) {
    let t = m.t * std::f32::consts::TAU / 3.0;
    let s = t.sin();
    let c = t.cos();

    lantern(m, p, b, 0.17 + 0.025 * c, 0.25 + 0.035 * s);
    tools::set(p, Head, [6.0 * s, 0.0, 0.0]);
}

fn prismancer(m: &Moment, p: &mut Pose, _b: &crate::art::rig::Proportions) {
    let t = m.t * std::f32::consts::TAU / 3.0;
    let s = t.sin();
    let c = t.cos();
    let beat = (t * 3.0).sin();

    tools::set(p, HandR, [8.0 * s, 8.0 * c, 4.0 * beat]);
    tools::set(p, Chest, [0.0, 0.0, 4.0 * s]);
    tools::set(p, Head, [-3.0, 6.0 * s, 0.0]);
}
fn leafcutter(m: &Moment, p: &mut Pose, _b: &crate::art::rig::Proportions) {
    let t = m.t * std::f32::consts::TAU / 3.0;
    let s = t.sin();
    let beat = (t * 3.0).sin();

    tools::set(p, UpperArmL, [-95.0 - 12.0 * s, 0.0, 12.0]);
    tools::set(p, ForearmL, [-65.0, 0.0, 0.0]);
    tools::set(p, HandR, [0.0, -10.0 * beat, 0.0]);
    tools::set(p, FootL, [-9.0 * beat.max(0.0), 0.0, 0.0]);
}
fn blastbeetle(m: &Moment, p: &mut Pose, _b: &crate::art::rig::Proportions) {
    let t = m.t * std::f32::consts::TAU / 3.0;
    let s = t.sin();
    let beat = (t * 3.0).sin();

    tools::set(p, Chest, [-6.0 * beat.max(0.0), 0.0, 0.0]);
    tools::set(p, Hips, [0.0, 0.0, 5.0 * s]);
    tools::set(p, Head, [9.0 * s.max(0.0), 0.0, 0.0]);
}
fn needlewing(m: &Moment, p: &mut Pose, _b: &crate::art::rig::Proportions) {
    let t = m.t * std::f32::consts::TAU / 3.0;
    let s = t.sin();
    let beat = (t * 3.0).sin();

    tools::set(p, UpperArmR, [-60.0 - 15.0 * s.max(0.0), 0.0, -10.0]);
    tools::set(p, Chest, [0.0, 0.0, 4.0 * beat]);
    tools::set(p, FootR, [-8.0 * beat.max(0.0), 0.0, 0.0]);
}
fn nectar_nurse(m: &Moment, p: &mut Pose, _b: &crate::art::rig::Proportions) {
    let t = m.t * std::f32::consts::TAU / 3.0;
    let s = t.sin();
    let beat = (t * 3.0).sin();

    tools::set(p, HandR, [0.0, 20.0 * beat, 8.0 * (t * 3.0).cos()]);
    tools::set(p, Chest, [0.0, 0.0, 4.0 * s]);
    tools::set(p, FootR, [-8.0 * s.max(0.0), 0.0, 0.0]);
}
fn stridulator(m: &Moment, p: &mut Pose, _b: &crate::art::rig::Proportions) {
    let t = m.t * std::f32::consts::TAU / 3.0;
    let s = t.sin();
    let beat = (t * 3.0).sin();

    tools::set(p, UpperArmR, [-35.0, 0.0, -15.0 - 12.0 * beat]);
    tools::set(p, UpperArmL, [-35.0, 0.0, 15.0 + 12.0 * beat]);
    tools::set(p, Chest, [-4.0 * beat, 0.0, 0.0]);
    tools::set(p, Head, [6.0 * s.max(0.0), 0.0, 0.0]);
}
