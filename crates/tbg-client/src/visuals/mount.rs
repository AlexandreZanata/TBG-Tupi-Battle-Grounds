//! Cargo Crab passengers: each one sits on the carrier its view names and moves
//! with it. Boarding climbs an arc into the saddle (started by `Event::Boarded`,
//! so a reconnect shows passengers already seated, with no arc), and a drop
//! steps back down when the view stops carrying them. Presentation only: the
//! server times everything. Runs after unit placement, so it reads the carrier's
//! final position.

use std::collections::HashMap;
use std::f32::consts::PI;

use bevy::prelude::*;
use tbg_core::UnitId;

use crate::art::anim::Animator;
use crate::art::kits::saddle_height;
use crate::art::rig::{self, Joint, Node, Proportions};
use crate::sync::{LatestView, UnitEntities};

/// Seconds a passenger takes to climb into the saddle after `Event::Boarded`.
const BOARD_TIME: f32 = 0.5;
/// Seconds a dropped passenger takes to step down to the ground.
const DROP_TIME: f32 = 0.4;
/// How high a boarding passenger rises above the straight path.
const BOARD_ARC: f32 = 0.6;

/// A passenger climbing from `from` into its saddle.
#[derive(Component)]
pub struct Boarding {
    from: Vec3,
    age: f32,
}

impl Boarding {
    pub fn new(from: Vec3) -> Self {
        Self { from, age: 0.0 }
    }
}

/// Seated passengers and the drops in progress, between frames.
#[derive(Resource, Default)]
pub struct Mounts {
    /// Passenger → the saddle it sat on in the last view that carried it.
    seated: HashMap<UnitId, Vec3>,
    descents: Vec<Descent>,
}

impl Mounts {
    pub fn clear(&mut self) {
        self.seated.clear();
        self.descents.clear();
    }
}

/// A passenger stepping down from `from` to where its unit now stands.
struct Descent {
    entity: Entity,
    from: Vec3,
    to: Option<Vec3>,
    age: f32,
}

/// Smooth start and stop over `0..=1`.
fn ease(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// How far a seated figure's root sits below its hips, in world units.
fn hip_height(plan: &Proportions) -> f32 {
    rig::rest(Node::Body(Joint::Hips), plan).y * plan.scale
}

pub fn passengers(
    time: Res<Time>,
    latest: Res<LatestView>,
    map: Res<UnitEntities>,
    mut mounts: ResMut<Mounts>,
    mut commands: Commands,
    mut figures: Query<(&mut Transform, &Animator)>,
    mut boarding: Query<(Entity, &mut Boarding)>,
) {
    let dt = time.delta_secs();
    for (entity, mut b) in &mut boarding {
        b.age += dt;
        if b.age >= BOARD_TIME {
            commands.entity(entity).remove::<Boarding>();
        }
    }
    let Some(view) = latest.0.as_ref() else {
        return;
    };

    let mut carried = Vec::new();
    for u in &view.units {
        let Some(carrier) = u.carrier else {
            continue;
        };
        let (Some(&mount), Some(&entity)) = (map.0.get(&carrier), map.0.get(&u.id)) else {
            continue;
        };
        let Ok(base) = figures.get(mount).map(|(t, _)| t.translation) else {
            continue;
        };
        let tier = view
            .units
            .iter()
            .find(|c| c.id == carrier)
            .map_or(0, |c| c.tier);
        // The seat is the carrier's; the passenger's root sits one hip height below it.
        let seat = base + Vec3::Y * saddle_height(tier);
        let hips = figures
            .get(entity)
            .map_or(0.0, |(_, a)| hip_height(&a.plan));
        carried.push((u.id, entity, seat - Vec3::Y * hips));
    }

    for &(id, entity, saddle) in &carried {
        let Ok((mut t, _)) = figures.get_mut(entity) else {
            continue;
        };
        t.translation = match boarding.get(entity) {
            Ok((_, b)) => {
                let k = ease(b.age / BOARD_TIME);
                b.from.lerp(saddle, k) + Vec3::Y * BOARD_ARC * (k * PI).sin()
            }
            Err(_) => saddle,
        };
        mounts.seated.insert(id, saddle);
    }

    let lost: Vec<UnitId> = mounts
        .seated
        .keys()
        .filter(|id| carried.iter().all(|(c, _, _)| c != *id))
        .copied()
        .collect();
    for id in lost {
        let Some(from) = mounts.seated.remove(&id) else {
            continue;
        };
        if let Some(&entity) = map.0.get(&id) {
            mounts.descents.push(Descent {
                entity,
                from,
                to: None,
                age: 0.0,
            });
        }
    }

    mounts.descents.retain_mut(|d| {
        let Ok((mut t, _)) = figures.get_mut(d.entity) else {
            return false;
        };
        d.age += dt;
        // Place has put the unit back on the ground this frame: step to there.
        let to = *d.to.get_or_insert(t.translation);
        t.translation = d.from.lerp(to, ease(d.age / DROP_TIME));
        d.age < DROP_TIME
    });
}
