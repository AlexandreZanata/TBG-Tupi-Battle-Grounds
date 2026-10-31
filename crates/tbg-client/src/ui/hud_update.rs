//! Per-frame HUD refresh from `LatestView`.

use bevy::prelude::*;
use tbg_core::content::{
    augment_def, def, ENERGY_SCALE, INTEREST_BAND, INTEREST_MAX_BANDS, TICK_HZ,
};
use tbg_core::FACTIONS;
use tbg_core::{MatchView, Phase};

use super::hud::Hud;
use super::{PendingBuy, BUTTON, DISABLED};
use crate::sync::LatestView;
use crate::visuals::assets::FACTION_COLORS;

pub(super) fn update(
    view: Res<LatestView>,
    info_state: Res<crate::sync::MatchInfo>,
    mut pending: ResMut<PendingBuy>,
    mut nodes: Query<(&Hud, &mut Node)>,
    mut colors: Query<(&Hud, &mut BackgroundColor)>,
    mut texts: Query<(&Hud, &mut Text)>,
    #[cfg(target_os = "linux")] tower_open: Res<super::fortify::TowerOpen>,
) {
    let Some(v) = &view.0 else { return };
    if pending.0.is_some_and(|(_, seq)| v.last_seq >= seq) {
        pending.0 = None;
    }
    let me = v.you.index();
    for (hud, mut node) in &mut nodes {
        #[cfg(target_os = "linux")]
        if let Some(show) = super::native_battle::auxiliary_visible(hud, v, tower_open.0) {
            node.display = if show { Display::Flex } else { Display::None };
            continue;
        }
        let fraction = match hud {
            // Kinds and Fortification give each seat its own max HP (BR-SK07).
            Hud::Hp(0) => hp_fraction(v, me),
            Hud::Hp(_) => hp_fraction(v, 1 - me),
            Hud::EnergyFill => {
                let band = (INTEREST_BAND * INTEREST_MAX_BANDS) as f32;
                (v.energy as f32 / band.max(1.0)).min(1.0)
            }
            // The War Council row exists only while the battle is paused.
            Hud::PrivateControls | Hud::WatchControls => {
                let show = matches!(hud, Hud::WatchControls) == info_state.spectating;
                node.display = if show { Display::Flex } else { Display::None };
                continue;
            }
            Hud::CouncilRow => {
                node.display = if v.augment_round.is_some() {
                    Display::Flex
                } else {
                    Display::None
                };
                continue;
            }
            _ => continue,
        };
        node.width = percent(100.0 * fraction.clamp(0.0, 1.0));
    }
    for (hud, mut color) in &mut colors {
        match hud {
            Hud::Card(i) => {
                color.0 = if buyable(v, *i)
                    && pending
                        .0
                        .is_none_or(|(o, _)| o != u8::MAX && usize::from(o) != *i)
                {
                    BUTTON
                } else {
                    DISABLED
                }
            }
            Hud::CardStrip(i) => {
                let c = FACTION_COLORS[def(v.offers[*i]).faction.index()];
                color.0 = Color::srgb_u8(c[0], c[1], c[2]);
            }
            Hud::CouncilOption(i) => {
                let open = v
                    .augment_round
                    .as_ref()
                    .is_some_and(|r| !r.resolved && r.ticks_left > 0 && v.energy >= r.costs[*i]);
                color.0 = if open { BUTTON } else { DISABLED };
            }
            Hud::Forgo => {
                let open = v
                    .augment_round
                    .as_ref()
                    .is_some_and(|r| !r.resolved && r.ticks_left > 0);
                color.0 = if open { BUTTON } else { DISABLED };
            }
            _ => {}
        }
    }
    for (hud, mut text) in &mut texts {
        text.0 = match hud {
            #[cfg(not(target_os = "linux"))]
            Hud::Clock => clock(v),
            #[cfg(not(target_os = "linux"))]
            Hud::HpLabel(relative) => {
                let seat = if *relative == 0 { me } else { 1 - me };
                format!(
                    "{} / {} HP",
                    v.stronghold_hp[seat].max(0),
                    v.strongholds[seat].max_hp
                )
            }
            Hud::Spectators => format!(
                "{} vs {} · {} spectators",
                info_state.you_name, info_state.opponent, v.spectators
            ),
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
            Hud::Augments => augment_list(v, v.you.index()),
            Hud::CouncilStatus => match &v.augment_round {
                Some(r) => {
                    let s = r.ticks_left.div_ceil(TICK_HZ);
                    if r.resolved {
                        format!(
                            "Augment selected! Waiting for opponent... {}:{:02}",
                            s / 60,
                            s % 60
                        )
                    } else {
                        format!(
                            "BATTLE PAUSED - War Council {}:{:02} left\nChoose one Augment",
                            s / 60,
                            s % 60
                        )
                    }
                }
                None => "War Council every 1:00 of battle".into(),
            },
            Hud::CouncilOptionText(i) => v
                .augment_round
                .as_ref()
                .and_then(|r| {
                    let id = *r.options.get(*i)?;
                    let d = augment_def(id)?;
                    Some(format!(
                        "{}\n{} Energy\n{}",
                        d.name,
                        r.costs[*i] / ENERGY_SCALE,
                        d.text
                    ))
                })
                .unwrap_or_default(),
            Hud::ForgoText => {
                if v.augment_round
                    .as_ref()
                    .is_some_and(|r| r.resolved && r.ticks_left > 0)
                {
                    "Chosen".into()
                } else {
                    "Forgo".into()
                }
            }
            _ => continue,
        };
    }
}

fn hp_fraction(v: &MatchView, seat: usize) -> f32 {
    v.stronghold_hp[seat] as f32 / v.strongholds[seat].max_hp.max(1) as f32
}

pub(super) fn buyable(v: &MatchView, i: usize) -> bool {
    let d = def(v.offers[i]);
    // Energy is the only purchase condition (BR-U07); the Shop is frozen
    // while an Augment selection pauses the battle (BR-AG12).
    v.energy >= d.cost && v.phase == Phase::Running && v.augment_round.is_none()
}

#[cfg(not(target_os = "linux"))]
pub(super) fn clock(v: &MatchView) -> String {
    if v.phase == Phase::Countdown {
        return "STARTING".into();
    }
    let s = v.elapsed / TICK_HZ;
    format!("{}:{:02}", s / 60, s % 60)
}

fn info(v: &MatchView) -> String {
    let synergies: Vec<String> = FACTIONS
        .iter()
        .filter(|f| v.synergy[v.you.index()][f.index()] > 0)
        .map(|f| format!("{f:?} {}/2", v.synergy[v.you.index()][f.index()]))
        .collect();
    #[cfg(target_os = "linux")]
    {
        synergies.join("  ")
    }
    #[cfg(not(target_os = "linux"))]
    {
        let energy = v.energy as f32 / ENERGY_SCALE as f32;
        let regen = v.regen_bp as f32 / 10_000.0;
        format!(
            "Energy {energy:.1} | Regen x{regen:.2} | {} | {}",
            army_label(v),
            synergies.join("  ")
        )
    }
}

fn augment_list(v: &MatchView, seat: usize) -> String {
    let list = v.augments[seat]
        .iter()
        .enumerate()
        .filter(|(_, copies)| **copies > 0)
        .filter_map(|(id, copies)| {
            let def = augment_def(id as u8)?;
            Some(if *copies > 1 {
                format!("{} x{copies}", def.name)
            } else {
                def.name.to_string()
            })
        })
        .collect::<Vec<_>>()
        .join(" | ");
    if list.is_empty() {
        "Augments: none".into()
    } else {
        format!("Augments: {list}")
    }
}

/// `Army N`, plus `Brood M` while the seat has summons (BR-U07, BR-U08).
pub(crate) fn army_label(v: &MatchView) -> String {
    let army = tbg_core::bot::army(v);
    let brood = v
        .units
        .iter()
        .filter(|u| u.seat == v.you && u.is_summon())
        .count();
    if brood > 0 {
        format!("Army {army} | Brood {brood}")
    } else {
        format!("Army {army}")
    }
}
