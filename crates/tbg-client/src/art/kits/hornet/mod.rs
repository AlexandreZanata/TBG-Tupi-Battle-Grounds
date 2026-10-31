//! Hornet's planted wasp gait, slow opaque wing hum, stinger alignment and confirmed venom-sac pulse.
mod model;
use super::{
    common::{moves, moves::*},
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
        from: Prop(Socket::WeaponR),
        muzzle: model::muzzle,
        normal: ShotKind::Sting,
        powered: ShotKind::VenomSting,
        every: |_, _| 3,
        loaded: None,
    },
    evolve: Burst::Bloom,
    durations: moves::DURATIONS,
};
fn pose(m: &Moment, p: &mut Pose) {
    let body = model::plan(m.tier);
    match m.clip {
        Clip::Idle => moves::idle(m, p, &body, 0.3),
        Clip::Walk => moves::walk(m, p, 3.0, 2.0),
        Clip::AttackRanged | Clip::Cast | Clip::AttackMelee => {
            let coil = tools::hold(m.t, [0.0, 0.13, 0.18, 0.30]);
            let release = tools::hold(m.t, [0.15, 0.18, 0.25, 0.55]);
            tools::set(p, Chest, [-7.0 * coil + 5.0 * release, 0.0, 0.0]);
            p[Secondary(Spring::Back).index()].0.x = 12.0 * coil;
        }
        Clip::Hit => {
            moves::hit(m, p, 0.8);
            p[Secondary(Spring::HeadGear).index()].0.z = -20.0;
            p[Secondary(Spring::Tabard).index()].0.z = 20.0;
        }
        Clip::Death => {
            moves::death(m, p, &body, Fall::Kneel);
            p[Secondary(Spring::Back).index()].0.x = 60.0 * tools::window(m.t, 0.2, 0.8);
        }
        Clip::Spawn => moves::spawn(m, p, &body, Entry::Rise),
        Clip::Evolve => moves::evolve(m, p, &body, Power::Raise),
        Clip::Victory => moves::victory(m, p, &body, Cheer::Both),
    }
    if m.clip != Clip::Death {
        tools::reach(
            p,
            false,
            Vec3::new(-0.14, 0.12, 0.26),
            Some(Quat::IDENTITY),
            (&body, Vec3::new(-1.0, -0.2, 0.0)),
        );
        tools::set(p, UpperArmL, [-15.0, 0.0, 12.0]);
        for (s, n) in [(-1.0, Spring::Tabard), (1.0, Spring::HeadGear)] {
            p[Secondary(n).index()].0.z += s * (m.clock * 3.0).sin() * 5.0;
        }
    }
    if let Some(t) = m.special {
        let k = tools::hold(t, [0.25, 0.30, 0.45, 0.70]);
        tools::add(p, Chest, [-6.0 * k, 0.0, 0.0]);
    }
    tools::face(p, m.aim, Vec3::new(0.0, 0.8, 0.3));
}
