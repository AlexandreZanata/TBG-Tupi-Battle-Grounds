//! Unit art kits: a character's own figure (five forms), signature motion
//! over the shared clip library, gait, aim and attack style (own
//! projectile or melee impact), registered by `UnitKind`. Plain data and
//! function pointers; units without a kit keep the shared library.

pub mod aim;
mod antidote_brewer;
mod archer;
mod beetling;
mod blood_furnace;
mod bloodweaver;
mod bloomkeeper;
mod bombard;
mod bone_brute;
mod cargo_crab;
mod cargo_crate;
pub mod common;
mod drumguard;
mod farwatch;
pub mod gait;
mod hexer;
mod leechroot;
mod longaxe;
mod nester;
mod oilpress;
mod old_finn;
mod puppetmaster;
mod rampart;
mod rootcaller;
mod rotcap;
mod sawbot;
pub(crate) mod shatterback;
mod shieldbearer;
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

/// A prop that exists only while a shot is loaded (the archer's nocked
/// arrow): its node, and whether it shows at a clip time.
pub type Loaded = (Node, fn(Clip, f32) -> bool);

/// How a kit's attack reaches its target.
pub enum Attack {
    /// A projectile leaves `muzzle` (a point in `from`'s space) on Release,
    /// with an optional muzzle effect and backblast point (same space).
    Shot {
        from: Node,
        muzzle: fn(u8, u8) -> Vec3,
        shot: fn(u8, u8) -> ShotKind,
        flash: fn(u8, u8) -> Flash,
        loaded: Option<Loaded>,
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
    pub gesture: f32,
    pub growth: f32,
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
static BOMBARD: Kit = bombard::KIT;
static OILPRESS: Kit = oilpress::KIT;
static CARGO_CRAB: Kit = cargo_crab::KIT;
static ROTCAP: Kit = rotcap::KIT;
static LEECHROOT: Kit = leechroot::KIT;
static ANTIDOTE_BREWER: Kit = antidote_brewer::KIT;
static SHATTERBACK: Kit = shatterback::KIT;
static BLOOD_FURNACE: Kit = blood_furnace::KIT;
static BLOODWEAVER: Kit = bloodweaver::KIT;
static PUPPETMASTER: Kit = puppetmaster::KIT;
static SHIELDBEARER: Kit = shieldbearer::KIT;
static HEXER: Kit = hexer::KIT;
static SAWBOT: Kit = sawbot::KIT;
static BLOOMKEEPER: Kit = bloomkeeper::KIT;
static NESTER: Kit = nester::KIT;
static BEETLING: Kit = beetling::KIT;
static ARCHER: Kit = archer::KIT;
static RAMPART: Kit = rampart::KIT;

/// The passenger seat's height above a Cargo Crab's ground, in world units,
/// at its tier: where a rider's hips sit (see `visuals::mount`).
pub fn saddle_height(tier: u8) -> f32 {
    cargo_crab::saddle_height(tier)
}

/// The kit of `kind`, if it has one.
pub fn kit(kind: UnitKind) -> Option<&'static Kit> {
    match kind {
        UnitKind::Archer => Some(&ARCHER),
        UnitKind::SiegeWalker => Some(&SIEGE_WALKER),
        UnitKind::BoneBrute => Some(&BONE_BRUTE),
        UnitKind::Rootcaller => Some(&ROOTCALLER),
        UnitKind::Longaxe => Some(&LONGAXE),
        UnitKind::Farwatch => Some(&FARWATCH),
        UnitKind::OldFinn => Some(&OLD_FINN),
        UnitKind::Drumguard => Some(&DRUMGUARD),
        UnitKind::Smuggler => Some(&SMUGGLER),
        UnitKind::Crate => Some(&CRATE),
        UnitKind::Bombard => Some(&BOMBARD),
        UnitKind::Oilpress => Some(&OILPRESS),
        UnitKind::CargoCrab => Some(&CARGO_CRAB),
        UnitKind::Rotcap => Some(&ROTCAP),
        UnitKind::Leechroot => Some(&LEECHROOT),
        UnitKind::AntidoteBrewer => Some(&ANTIDOTE_BREWER),
        UnitKind::Shatterback => Some(&SHATTERBACK),
        UnitKind::BloodFurnace => Some(&BLOOD_FURNACE),
        UnitKind::Bloodweaver => Some(&BLOODWEAVER),
        UnitKind::Puppetmaster => Some(&PUPPETMASTER),
        UnitKind::Shieldbearer => Some(&SHIELDBEARER),
        UnitKind::Hexer => Some(&HEXER),
        UnitKind::Sawbot => Some(&SAWBOT),
        UnitKind::Bloomkeeper => Some(&BLOOMKEEPER),
        UnitKind::Nester => Some(&NESTER),
        UnitKind::Beetling => Some(&BEETLING),
        UnitKind::Rampart => Some(&RAMPART),
        #[allow(unreachable_patterns)]
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
