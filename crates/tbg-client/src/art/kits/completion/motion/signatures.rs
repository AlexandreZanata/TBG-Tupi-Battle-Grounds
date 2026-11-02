//! Compact idle/walk tells and independent wing hinges, layered over distance-driven leg motion.
use super::*;
use crate::art::rig::{Node::Prop, Socket};
pub(super) fn apply(k: UnitKind, m: &Moment, p: &mut Pose) {
    wings(k, m, p);
    if !matches!(m.clip, Clip::Idle | Clip::Walk) {
        return;
    }
    let wave = (m.clock * 1.7).sin();
    let step = m.walk.map_or(0.0, |k| (k * std::f32::consts::TAU).sin());
    match k {
        Gravecutter => {
            tools::add(p, HandR, [0.0, 5.0 * wave, 2.0 * wave]);
            tools::add(p, Head, [0.0, 0.0, 2.0 * step]);
        }
        LastlightCantor => {
            tools::add(p, HandR, [4.0 * wave, 0.0, 0.0]);
            tools::add(p, ForearmL, [-5.0 * wave, 0.0, 0.0]);
        }
        Cragclaw => {
            tools::add(p, Chest, [2.0 * step, 0.0, -3.0 * step]);
            tools::add(p, UpperArmR, [-5.0 * step, 0.0, 0.0]);
        }
        Spinehurler => {
            p[Secondary(Spring::Back).index()].0.z = 2.5 * wave;
            tools::add(p, Head, [0.0, -4.0 * wave, 0.0]);
        }
        MireTender => {
            tools::add(p, Head, [2.0 * wave, 0.0, 0.0]);
            tools::add(p, HandR, [0.0, 5.0 * wave, 3.0 * (m.clock * 1.7).cos()]);
        }
        Gorgehowl => {
            p[Secondary(Spring::HeadGear).index()].0.y = 4.0 * wave;
            tools::add(p, Head, [0.0, 3.0 * wave, 0.0]);
        }
        Sunlancer => tools::add(p, Head, [-2.0 * wave, 0.0, 2.0 * step]),
        EclipseDancer => {
            tools::add(p, HandL, [0.0, -5.0 * wave, 0.0]);
            tools::add(p, HandR, [0.0, 5.0 * wave, 0.0]);
            tools::add(p, Chest, [0.0, 3.0 * step, 0.0]);
        }
        Starbreaker => tools::add(p, Head, [3.0 * step, 0.0, 0.0]),
        Dawnmender => tools::add(p, Head, [3.0 * wave, 0.0, 2.0 * step]),
        Prismancer => tools::add(p, HandL, [0.0, 5.0 * wave, 0.0]),
        Leafcutter => {
            tools::add(p, Head, [0.0, 0.0, 3.0 * step]);
            tools::add(p, ForearmL, [-5.0 * step, 0.0, 0.0]);
        }
        Blastbeetle => tools::add(p, Hips, [0.0, 0.0, 2.5 * step]),
        Needlewing => tools::add(p, Head, [-2.0 * wave, 3.0 * wave, 0.0]),
        NectarNurse => tools::add(p, HandR, [0.0, 6.0 * wave, 3.0 * (m.clock * 1.7).cos()]),
        Stridulator => {
            tools::add(p, Chest, [-2.0 * wave, 0.0, 0.0]);
            tools::add(p, ForearmL, [-4.0 * wave, 0.0, 0.0]);
        }
        _ => {}
    }
}
fn wings(k: UnitKind, m: &Moment, p: &mut Pose) {
    if m.clip == Clip::Death {
        return;
    }
    let phase = if m.clip == Clip::Victory {
        m.t
    } else {
        m.clock
    };
    match k {
        NectarNurse => {
            p[Secondary(Spring::HeadGear).index()].0.y = (phase
                * if m.clip == Clip::Victory {
                    std::f32::consts::TAU * 2.0
                } else {
                    14.0
                })
            .sin()
                * if m.clip == Clip::Victory { 9.0 } else { 2.0 }
        }
        Needlewing => {
            p[Prop(Socket::WeaponR).index()].0.x = (phase
                * if m.clip == Clip::Victory {
                    std::f32::consts::TAU * 5.0 / 3.0
                } else {
                    10.0
                })
            .sin()
                * if m.clip == Clip::Victory { 8.0 } else { 1.5 }
        }
        Stridulator => {
            p[Secondary(Spring::HeadGear).index()].0.z = (phase
                * if m.clip == Clip::Victory {
                    std::f32::consts::TAU / 3.0
                } else {
                    3.0
                })
            .sin()
                * 2.0
        }
        _ => {}
    }
}
