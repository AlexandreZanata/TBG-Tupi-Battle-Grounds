//! Parse gallery-only URL tools once; simulation time is never changed.

use crate::{
    art::anim::{Clip, CLIPS},
    config,
};
use bevy::prelude::*;
use tbg_core::{UnitKind, UNIT_KINDS};

#[derive(Resource)]
pub struct Gallery {
    pub mode: String,
    pub unit: Option<UnitKind>,
    pub slow: f32,
    pub silhouette: bool,
    pub turn: bool,
    pub frames: bool,
    pub stress: bool,
    pub clip: Option<Clip>,
    /// `&tier=1|2`: contact sheet of an evolution (branch A) instead of the base.
    pub tier: u8,
    /// `&lineup`: base, tier 1, and tier 2 side by side, idle and at the anchor.
    pub lineup: bool,
    /// Isolated base-form screenshot for a Shop card.
    pub portrait: bool,
    /// `&range`: the five forms attacking dummies on the BattleLine.
    pub range: bool,
}
impl Gallery {
    pub fn studio(&self) -> bool {
        (self.unit.is_some() && !self.range)
            || self.frames
            || self.mode == "blockout"
            || self.mode == "strongholds"
    }
    pub fn hide_arena(&self) -> bool {
        self.silhouette || self.studio()
    }
    pub fn floor(&self) -> bool {
        !self.portrait && !self.silhouette && !self.frames && self.studio()
    }
    pub fn read() -> Self {
        let mode = config::url_param("gallery")
            .unwrap_or_default()
            .to_lowercase();
        let unit = mode
            .parse::<usize>()
            .ok()
            .and_then(|i| UNIT_KINDS.get(i).copied())
            .or_else(|| {
                UNIT_KINDS
                    .into_iter()
                    .find(|kind| slug(&format!("{kind:?}")) == slug(&mode))
            })
            .or(match mode.as_str() {
                "sawknight" => Some(UnitKind::Sawbot),
                "ogrearcher" => Some(UnitKind::SiegeWalker),
                _ => None,
            });
        let clip = config::url_param("clip").and_then(|s| {
            CLIPS
                .into_iter()
                .find(|clip| slug(&format!("{clip:?}")) == slug(&s))
        });
        Self {
            mode,
            unit,
            slow: config::url_param("slow")
                .and_then(|s| s.parse::<f32>().ok())
                .filter(|v| v.is_finite() && *v > 0.0)
                .unwrap_or(1.0)
                .clamp(0.01, 4.0),
            silhouette: config::url_param("silhouette").is_some(),
            turn: config::url_param("turn").is_some(),
            frames: config::url_param("frames").is_some() || config::url_param("lineup").is_some(),
            stress: config::url_param("stress")
                .and_then(|s| s.parse::<usize>().ok())
                .is_some_and(|n| n > 0),
            clip,
            tier: config::url_param("tier")
                .and_then(|s| s.parse::<u8>().ok())
                .unwrap_or(0)
                .min(2),
            lineup: config::url_param("lineup").is_some(),
            portrait: config::url_param("portrait").is_some(),
            range: config::url_param("range").is_some(),
        }
    }
}
fn slug(s: &str) -> String {
    s.to_ascii_lowercase().replace(['-', '_', ' '], "")
}
