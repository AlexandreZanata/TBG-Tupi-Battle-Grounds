//! Per-frame HUD refresh from `LatestView`.

use bevy::prelude::*;
use tbg_core::content::{
    def, ENERGY_CAP, ENERGY_SCALE, REGULATION, ROTATION_TICKS, STRONGHOLD_HP, TICK_HZ,
};
use tbg_core::{MatchView, Phase, FACTIONS};

use super::hud::Hud;
use super::{PendingBuy, BUTTON, DISABLED};
use crate::sync::LatestView;
use crate::visuals::assets::FACTION_COLORS;

pub(super) fn update(
    view: Res<LatestView>,
    mut pending: ResMut<PendingBuy>,
    mut nodes: Query<(&Hud, &mut Node)>,
    mut colors: Query<(&Hud, &mut BackgroundColor)>,
    mut texts: Query<(&Hud, &mut Text)>,
) {
    let Some(v) = &view.0 else { return };
    if pending.0.is_some_and(|(_, seq)| v.last_seq >= seq) {
        pending.0 = None;
    }
    let me = v.you.index();
    for (hud, mut node) in &mut nodes {
        let fraction = match hud {
            Hud::Hp(0) => v.stronghold_hp[me] as f32 / STRONGHOLD_HP as f32,
            Hud::Hp(_) => v.stronghold_hp[1 - me] as f32 / STRONGHOLD_HP as f32,
            Hud::EnergyFill => v.energy as f32 / ENERGY_CAP as f32,
            Hud::Rotation => 1.0 - v.next_rotation_in as f32 / ROTATION_TICKS as f32,
            _ => continue,
        };
        node.width = percent(100.0 * fraction.clamp(0.0, 1.0));
    }
    for (hud, mut color) in &mut colors {
        match hud {
            Hud::Card(i) => {
                color.0 = if buyable(v, *i) && pending.0.is_none_or(|(o, _)| usize::from(o) != *i) {
                    BUTTON
                } else {
                    DISABLED
                }
            }
            Hud::CardStrip(i) => {
                let c = FACTION_COLORS[def(v.offers[*i]).faction.index()];
                color.0 = Color::srgb_u8(c[0], c[1], c[2]);
            }
            _ => {}
        }
    }
    for (hud, mut text) in &mut texts {
        text.0 = match hud {
            Hud::Clock => clock(v),
            Hud::Info => info(v),
            Hud::CardText(i) => {
                let d = def(v.offers[*i]);
                format!(
                    "{}\n{:?}\nCost {}",
                    d.name,
                    d.archetype,
                    d.cost / ENERGY_SCALE
                )
            }
            _ => continue,
        };
    }
}

pub(super) fn buyable(v: &MatchView, i: usize) -> bool {
    let d = def(v.offers[i]);
    // Energy is the only purchase condition (BR-U07).
    v.energy >= d.cost && matches!(v.phase, Phase::Running | Phase::Overtime)
}

pub(super) fn clock(v: &MatchView) -> String {
    let s = v.phase_ticks_left / TICK_HZ;
    match v.phase {
        Phase::Countdown => format!(
            "{}:{:02}",
            REGULATION / TICK_HZ / 60,
            REGULATION / TICK_HZ % 60
        ),
        Phase::Overtime => format!("OVERTIME {}:{:02}", s / 60, s % 60),
        _ => format!("{}:{:02}", s / 60, s % 60),
    }
}

fn info(v: &MatchView) -> String {
    let synergies: Vec<String> = FACTIONS
        .iter()
        .filter(|f| v.synergy[v.you.index()][f.index()] > 0)
        .map(|f| format!("{f:?} {}/2", v.synergy[v.you.index()][f.index()]))
        .collect();
    let energy = v.energy as f32 / ENERGY_SCALE as f32;
    format!(
        "Energy {energy:.1} | {} | {}",
        army_label(v),
        synergies.join("  ")
    )
}

/// `Army N`, plus `Brood M` while the seat has summons (BR-U07, BR-U08).
pub(crate) fn army_label(v: &MatchView) -> String {
    let army = tbg_core::bot::army(v);
    let brood = v
        .units
        .iter()
        .filter(|u| u.seat == v.you && u.kind.is_summon())
        .count();
    if brood > 0 {
        format!("Army {army} | Brood {brood}")
    } else {
        format!("Army {army}")
    }
}
