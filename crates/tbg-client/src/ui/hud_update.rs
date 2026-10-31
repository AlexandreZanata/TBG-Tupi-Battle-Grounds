//! Per-frame HUD refresh from `LatestView`.

use bevy::prelude::*;
use tbg_core::content::{
    augment_def, def, AUGMENT_ROUND_TICKS, ENERGY_SCALE, INTEREST_BAND, INTEREST_MAX_BANDS, TICK_HZ,
};
use tbg_core::{MatchView, Phase, FACTIONS};

use super::hud::Hud;
use super::{PendingBuy, BUTTON, DISABLED};
use crate::i18n::{augment, content, Lang, Msg};
use crate::sync::LatestView;
use crate::visuals::assets::FACTION_COLORS;

pub(super) fn update(
    view: Res<LatestView>,
    lang: Res<Lang>,
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
    let lang = *lang;
    for (hud, mut node) in &mut nodes {
        let fraction = match hud {
            // Kinds and Fortification give each seat its own max HP (BR-SK07).
            Hud::Hp(0) => hp_fraction(v, me),
            Hud::Hp(_) => hp_fraction(v, 1 - me),
            Hud::EnergyFill => {
                let band = (INTEREST_BAND * INTEREST_MAX_BANDS) as f32;
                (v.energy as f32 / band.max(1.0)).min(1.0)
            }
            // The War Council row exists only while the battle is paused.
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
            Hud::Clock => match v.phase {
                Phase::Countdown => lang.t(Msg::HudStarting).into(),
                _ => clock(v),
            },
            Hud::Info => info(v, lang),
            Hud::CardText(i) => {
                let d = def(v.offers[*i]);
                [
                    content::unit_name(lang, d.kind).to_string(),
                    content::archetype(lang, d.archetype).to_string(),
                    lang.tf(Msg::HudCost, &[("cost", &(d.cost / ENERGY_SCALE))]),
                ]
                .join("\n")
            }
            Hud::Augments => augment_list(v, v.you.index(), lang),
            Hud::CouncilStatus => council_status(v, lang),
            Hud::CouncilOptionText(i) => v
                .augment_round
                .as_ref()
                .and_then(|r| {
                    let id = *r.options.get(*i)?;
                    let d = augment_def(id)?;
                    Some(
                        [
                            content::augment_name(lang, id)?.to_string(),
                            lang.tf(
                                Msg::CommonEnergyAmount,
                                &[("cost", &(r.costs[*i] / ENERGY_SCALE))],
                            ),
                            augment::text(lang, d),
                        ]
                        .join("\n"),
                    )
                })
                .unwrap_or_default(),
            Hud::ForgoText => {
                if v.augment_round
                    .as_ref()
                    .is_some_and(|r| r.resolved && r.ticks_left > 0)
                {
                    lang.t(Msg::CouncilChosen).into()
                } else {
                    lang.t(Msg::CouncilForgo).into()
                }
            }
            _ => continue,
        };
    }
}

/// `m:ss` for a tick count, rounded up to whole seconds.
fn mmss(ticks: u32) -> String {
    let s = ticks.div_ceil(TICK_HZ);
    format!("{}:{:02}", s / 60, s % 60)
}

fn council_status(v: &MatchView, lang: Lang) -> String {
    match &v.augment_round {
        Some(r) if r.resolved => {
            lang.tf(Msg::CouncilSelectedWait, &[("time", &mmss(r.ticks_left))])
        }
        Some(r) => [
            lang.tf(Msg::CouncilPaused, &[("time", &mmss(r.ticks_left))]),
            lang.t(Msg::CouncilChoose).to_string(),
        ]
        .join("\n"),
        None => lang.tf(Msg::CouncilEvery, &[("time", &mmss(AUGMENT_ROUND_TICKS))]),
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

pub(super) fn clock(v: &MatchView) -> String {
    if v.phase == Phase::Countdown {
        return "STARTING".into();
    }
    let s = v.elapsed / TICK_HZ;
    format!("{}:{:02}", s / 60, s % 60)
}

/// The seat's active Synergies, `Humans 1/2  Plants 2/2`, joined by `sep`.
pub(crate) fn synergy_list(v: &MatchView, lang: Lang, sep: &str) -> String {
    FACTIONS
        .iter()
        .filter(|f| v.synergy[v.you.index()][f.index()] > 0)
        .map(|f| content::synergy(lang, *f, v.synergy[v.you.index()][f.index()]))
        .collect::<Vec<_>>()
        .join(sep)
}

fn info(v: &MatchView, lang: Lang) -> String {
    let energy = f64::from(v.energy) / f64::from(ENERGY_SCALE);
    let regen = f64::from(v.regen_bp) / 10_000.0;
    lang.tf(
        Msg::HudInfo,
        &[
            ("energy", &lang.decimal(energy, 1)),
            ("regen", &lang.decimal(regen, 2)),
            ("army", &army_label(v, lang)),
            ("synergy", &synergy_list(v, lang, "  ")),
        ],
    )
}

/// Owned Augment names, `Sentry | Legacy x2`, in `lang`.
pub(crate) fn augment_names(counts: &tbg_core::augment::AugmentCounts, lang: Lang) -> String {
    counts
        .iter()
        .enumerate()
        .filter(|(_, copies)| **copies > 0)
        .filter_map(|(id, copies)| {
            let name = content::augment_name(lang, id as u8)?;
            Some(if *copies > 1 {
                format!("{name} x{copies}")
            } else {
                name.to_string()
            })
        })
        .collect::<Vec<_>>()
        .join(" | ")
}

fn augment_list(v: &MatchView, seat: usize, lang: Lang) -> String {
    let list = augment_names(&v.augments[seat], lang);
    if list.is_empty() {
        lang.t(Msg::HudAugmentsNone).into()
    } else {
        lang.tf(Msg::HudAugments, &[("list", &list)])
    }
}

/// Living conventional units and summons of the viewing seat (BR-U07, BR-U08).
pub(crate) fn army_counts(v: &MatchView) -> (usize, usize) {
    let army = tbg_core::bot::army(v);
    let brood = v
        .units
        .iter()
        .filter(|u| u.seat == v.you && u.is_summon())
        .count();
    (army, brood)
}

/// `Army N`, plus `Brood M` while the seat has summons.
pub(crate) fn army_label(v: &MatchView, lang: Lang) -> String {
    let (army, brood) = army_counts(v);
    let label = lang.tf(Msg::HudArmy, &[("army", &army)]);
    if brood > 0 {
        format!("{label} | {}", lang.tf(Msg::HudBrood, &[("brood", &brood)]))
    } else {
        label
    }
}
