//! Unit art kits: a character's own figure (five forms), signature motion
//! over the shared clip library, gait, aim and attack style (own
//! projectile or melee impact), registered by `UnitKind`. Plain data and
//! function pointers; units without a kit keep the shared library.

pub mod aim;
mod bone_brute;
mod cargo_crate;
pub mod common;
mod drumguard;
mod farwatch;
pub mod gait;
mod longaxe;
mod old_finn;
mod rootcaller;
mod siege_walker;
mod smuggler;
mod tools;

use bevy::prelude::*;
use tbg_core::UnitKind;

use super::anim::{Clip, Pose};
use super::figure::FigureSpec;
use super::rig::Node;
use crate::vfx::{
    impacts::{Burst, Impact, Muzzle},
    shots::ShotKind,
};
pub use aim::Aim;
pub use gait::Gait;

pub struct Kit {
    /// The native rig v2 figure of (tier, branch).
    pub figure: fn(u8, u8) -> FigureSpec,
    /// Signature layer written over the shared clip pose.
    pub pose: fn(&Moment, &mut Pose),
    pub gait: Gait,
    pub attack: Attack,
    /// Its own Evolution burst, played with its own Evolve clip.
    pub evolve: Burst,
    /// Clip lengths that differ from the shared library. Event times never
    /// move (ART-DIRECTION §4); only recoveries get longer.
    pub durations: &'static [(Clip, f32)],
}

/// A muzzle effect and the backblast point (weapon space), if any.
pub type Flash = Option<(Muzzle, Option<Vec3>)>;

/// How a kit's attack reaches its target.
pub enum Attack {
    /// A projectile leaves `muzzle` (a point in `from`'s space) on Release,
    /// with an optional muzzle effect and backblast point (same space).
    Shot {
        from: Node,
        muzzle: fn(u8, u8) -> Vec3,
        shot: fn(u8, u8) -> ShotKind,
        flash: fn(u8, u8) -> Flash,
    },
    /// The weapon lands on Strike; `tip` is its point in `from`'s space.
    Melee {
        from: Node,
        tip: fn(u8, u8) -> Vec3,
        impact: fn(u8, u8) -> Impact,
    },
    /// No basic attack (passive supports); its actions are cued by their
    /// own events (`sync/events.rs`).
    None,
}

/// Everything a kit's pose layer may depend on, for one sampled clip.
#[derive(Clone, Copy, Debug)]
pub struct Moment {
    pub clip: Clip,
    /// Seconds into `clip`.
    pub t: f32,
    pub tier: u8,
    pub branch: u8,
    /// The engaged target in figure space, if any.
    pub aim: Option<Aim>,
    /// Seconds since the last attack started (reload loops).
    pub since_attack: f32,
    /// Alternates on every attack (A/B swings).
    pub variant: u8,
    /// Free-running seconds with this unit's phase offset (idle noise).
    pub clock: f32,
    /// Walk cycle phase (0..1 per two steps) while walking.
    pub walk: Option<f32>,
}

static SIEGE_WALKER: Kit = siege_walker::KIT;
static BONE_BRUTE: Kit = bone_brute::KIT;
static ROOTCALLER: Kit = rootcaller::KIT;
static LONGAXE: Kit = longaxe::KIT;
static FARWATCH: Kit = farwatch::KIT;
static OLD_FINN: Kit = old_finn::KIT;
static DRUMGUARD: Kit = drumguard::KIT;
static SMUGGLER: Kit = smuggler::KIT;
static CRATE: Kit = cargo_crate::KIT;

/// The kit of `kind`, if it has one.
pub fn kit(kind: UnitKind) -> Option<&'static Kit> {
    match kind {
        UnitKind::SiegeWalker => Some(&SIEGE_WALKER),
        UnitKind::BoneBrute => Some(&BONE_BRUTE),
        UnitKind::Rootcaller => Some(&ROOTCALLER),
        UnitKind::Longaxe => Some(&LONGAXE),
        UnitKind::Farwatch => Some(&FARWATCH),
        UnitKind::OldFinn => Some(&OLD_FINN),
        UnitKind::Drumguard => Some(&DRUMGUARD),
        UnitKind::Smuggler => Some(&SMUGGLER),
        UnitKind::Crate => Some(&CRATE),
        _ => None,
    }
}

impl Kit {
    pub fn duration(&self, clip: Clip) -> f32 {
        self.durations
            .iter()
            .find(|(c, _)| *c == clip)
            .map_or_else(|| clip.duration(), |(_, d)| *d)
    }
}

#[cfg(test)]
mod tests;
