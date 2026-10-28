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
}
impl Gallery {
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
            frames: config::url_param("frames").is_some(),
            stress: config::url_param("stress")
                .and_then(|s| s.parse::<usize>().ok())
                .is_some_and(|n| n > 0),
            clip,
        }
    }
}
fn slug(s: &str) -> String {
    s.to_ascii_lowercase().replace(['-', '_', ' '], "")
}
