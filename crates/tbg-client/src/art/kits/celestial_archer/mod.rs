//! Celestial Archer's upright draw/release, rigid wing settling and confirmed purification halo.
mod model;
use super::{
    common::{gear, moves, moves::*},
    tools, Anatomy, Attack, Gait, Kit, Moment,
};
use crate::art::{
    anim::{Clip, Pose},
    rig::{
        Joint::*,
        Node::{Prop, Secondary},
        Socket, Spring,
    },
};
use crate::vfx::{impacts::Burst, shots::ShotKind};
use bevy::prelude::*;
static ANATOMY: Anatomy = Anatomy {
    rest: model::rest,
    parent: model::parent,
    springs: false,
    planted_legs: true,
};
pub const KIT: Kit = Kit {
    anatomy: Some(&ANATOMY),
    figure: model::spec,
    pose,
    gait: Gait::LIGHT,
    attack: Attack::CountedShot {
        from: Prop(Socket::OffhandL),
        muzzle: |_, _| model::MUZZLE,
        normal: ShotKind::CelestialArrow,
        powered: ShotKind::PurifyingArrow,
        every: |t, b| if t > 0 && b == 1 { 3 } else { 4 },
        loaded: Some((Prop(Socket::Nock), nocked)),
    },
    evolve: Burst::Valor,
    durations: moves::DURATIONS,
};
fn nocked(c: Clip, t: f32) -> bool {
    c == Clip::AttackRanged && t < 0.18
}
fn pose(m: &Moment, p: &mut Pose) {
    let body = model::plan(m.tier);
    let half = model::half(m.tier, m.branch);
    gear::bow_rest(p, half);
    match m.clip {
        Clip::Idle => {
            moves::idle(m, p, &body, 0.25);
            tools::set(p, UpperArmL, [-25.0, 0.0, 8.0]);
            tools::set(p, ForearmL, [-65.0, 0.0, 0.0]);
        }
        Clip::Walk => {
            moves::walk(m, p, 3.0, 1.5);
            tools::set(p, UpperArmL, [-30.0, 0.0, 8.0]);
            tools::set(p, ForearmL, [-60.0, 0.0, 0.0]);
        }
        Clip::AttackRanged | Clip::Cast | Clip::AttackMelee => draw(m, p, half),
        Clip::Hit => {
            moves::hit(m, p, 0.75);
            tools::set(p, UpperArmL, [-40.0, 0.0, 12.0]);
        }
        Clip::Death => {
            moves::death(m, p, &body, Fall::Kneel);
            p[Root.index()].0.z = 70.0 * tools::window(m.t, 0.35, 0.9);
        }
        Clip::Spawn => moves::spawn(m, p, &body, Entry::Rise),
        Clip::Evolve => {
            moves::evolve(m, p, &body, Power::Raise);
            tools::set(p, UpperArmL, [-100.0, 0.0, 25.0]);
        }
        Clip::Victory => moves::victory(m, p, &body, Cheer::Raise),
    }
    let flare = m
        .special
        .map_or(0.0, |t| tools::hold(t, [0.25, 0.30, 0.45, 0.75]));
    p[Secondary(Spring::HeadGear).index()].1.y += 0.035 * flare;
    for (s, n) in [(1.0, Spring::Back), (-1.0, Spring::Tabard)] {
        p[Secondary(n).index()].0.z = s * ((m.clock * 1.2).sin() * 2.0 + 8.0 * flare);
    }
    tools::face(p, m.aim, Vec3::new(0.0, 0.9, 0.3));
    gear::bow_upright(p, 0.0);
}
fn draw(m: &Moment, p: &mut Pose, half: f32) {
    let draw = tools::window(m.t, 0.0, 0.12);
    let recoil = tools::hold(m.t, [0.18, 0.21, 0.25, 0.40]);
    let recovery = tools::window(m.t, 0.42, 0.62);
    let pull = draw * (1.0 - tools::snap(m.t, 0.18, 0.21));
    gear::bow_string(p, half, pull, m.t - 0.18);
    tools::set(p, Chest, [0.0, -12.0 * draw * (1.0 - recovery), 0.0]);
    tools::set(
        p,
        UpperArmL,
        [-85.0 * draw + 55.0 * recovery, 0.0, 25.0 * draw],
    );
    tools::set(p, ForearmL, [-10.0 * draw - 45.0 * recovery, 0.0, 0.0]);
    tools::set(
        p,
        UpperArmR,
        [-65.0 * draw, 0.0, -55.0 * draw - 10.0 * recoil],
    );
    tools::set(p, ForearmR, [-110.0 * pull - 65.0 * recoil, 0.0, 0.0]);
}
