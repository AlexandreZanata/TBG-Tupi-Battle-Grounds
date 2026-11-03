//! Bottom-right War Auxiliaries (`Mineiro` and `Mercador`) panel in the native battle HUD (P74).
//! Separated from the five conventional Shop cards, sharing the navy-and-gold visual language.

#[cfg(target_os = "android")]
mod mobile;

use super::{
    super::{hud::Hud, kit, PendingBuy, UiAction},
    chrome, SHOP_BOTTOM, SHOP_SCALE,
};
use crate::{i18n::Lang, sync::LatestView, GameState};
use bevy::prelude::*;
use tbg_core::{
    auxiliary::{
        AuxiliaryKind, AuxiliaryPhase, AuxiliarySlotView, AUXILIARY_RULE, AUXILIARY_SLOTS,
    },
    content::{ENERGY_SCALE, TICK_HZ},
    MatchView, Phase,
};

#[derive(Component)]
struct AuxCardSurface(usize);

#[derive(Component)]
struct AuxCoinIcon(usize);

#[derive(Component)]
enum AuxReadout {
    Header,
    #[cfg(target_os = "android")]
    Toggle,
    Name(usize),
    Subtitle(usize),
    BadgePrefix(usize),
    BadgeValue(usize),
}

pub fn plugin(app: &mut App) {
    #[cfg(target_os = "android")]
    mobile::plugin(app);
    app.add_systems(Update, update.run_if(in_state(GameState::InMatch)));
}

#[cfg(target_os = "android")]
pub(super) fn spawn_toggle(parent: &mut ChildSpawnerCommands) {
    mobile::button(parent);
}

pub fn spawn(parent: &mut ChildSpawnerCommands, assets: &AssetServer) {
    #[cfg(target_os = "android")]
    mobile::spawn(parent, assets);
    #[cfg(not(target_os = "android"))]
    desktop(parent, assets);
}

#[cfg(not(target_os = "android"))]
fn desktop(parent: &mut ChildSpawnerCommands, assets: &AssetServer) {
    parent
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: px(14.),
                top: if cfg!(target_os = "android") {
                    px(90)
                } else {
                    Val::Auto
                },
                bottom: if cfg!(target_os = "android") {
                    Val::Auto
                } else {
                    px(SHOP_BOTTOM)
                },
                padding: UiRect::axes(
                    px(if cfg!(target_os = "android") { 4. } else { 10. }),
                    px(if cfg!(target_os = "android") { 4. } else { 8. }),
                ),
                border: UiRect::all(px(2.)),
                border_radius: BorderRadius::all(px(10.)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(if cfg!(target_os = "android") { 2. } else { 6. }),
                ..default()
            },
            BackgroundColor(Color::srgba_u8(12, 22, 33, 236)),
            BorderColor::all(chrome::GOLD),
            UiTransform {
                scale: Vec2::splat(SHOP_SCALE),
                translation: Val2::percent(0., (1. - SHOP_SCALE) * 50.),
                ..default()
            },
            Hud::PrivateControls,
        ))
        .with_children(|panel| {
            panel
                .spawn((
                    Node {
                        width: percent(100),
                        height: px(if cfg!(target_os = "android") {
                            18.
                        } else {
                            24.
                        }),
                        padding: UiRect::horizontal(px(8.)),
                        border: UiRect::bottom(px(1.)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor::all(chrome::GOLD.with_alpha(0.45)),
                ))
                .with_children(|hdr| {
                    hdr.spawn((
                        chrome::title(
                            "",
                            if cfg!(target_os = "android") {
                                11.
                            } else {
                                15.
                            },
                        ),
                        AuxReadout::Header,
                    ));
                });

            panel
                .spawn(Node {
                    column_gap: px(if cfg!(target_os = "android") { 4. } else { 8. }),
                    ..default()
                })
                .with_children(|row| {
                    for slot in 0..AUXILIARY_SLOTS {
                        card(row, assets, slot);
                    }
                });
        });
}

fn card(parent: &mut ChildSpawnerCommands, assets: &AssetServer, slot: usize) {
    let portrait_path = match slot {
        0 => "battle/portraits/aux_miner.webp",
        _ => "battle/portraits/aux_merchant.webp",
    };
    parent
        .spawn((
            Node {
                width: px(if cfg!(target_os = "android") {
                    118.
                } else {
                    124.
                }),
                height: px(if cfg!(target_os = "android") {
                    80.
                } else {
                    176.
                }),
                flex_shrink: 0.,
                ..default()
            },
            ImageNode {
                image_mode: NodeImageMode::Stretch,
                ..ImageNode::new(assets.load("battle/native/shop-card.webp"))
            },
            UiAction::HireAuxiliary(slot as u8),
            AuxCardSurface(slot),
        ))
        .with_children(|card| {
            card.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: px(7.),
                    left: px(5.),
                    width: px(if cfg!(target_os = "android") {
                        32.
                    } else {
                        114.
                    }),
                    height: px(if cfg!(target_os = "android") {
                        32.
                    } else {
                        102.
                    }),
                    ..default()
                },
                ImageNode::new(assets.load(portrait_path)),
            ));
            card.spawn(Node {
                position_type: PositionType::Absolute,
                left: px(if cfg!(target_os = "android") { 42. } else { 9. }),
                right: px(8.),
                top: px(if cfg!(target_os = "android") {
                    8.
                } else {
                    112.
                }),
                height: px(20.),
                overflow: Overflow::clip(),
                ..default()
            })
            .with_children(|title| {
                title.spawn((
                    chrome::title(
                        "",
                        if cfg!(target_os = "android") {
                            11.
                        } else {
                            14.
                        },
                    ),
                    TextLayout::no_wrap(),
                    AuxReadout::Name(slot),
                ));
            });
            card.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(9.),
                    right: px(8.),
                    top: px(if cfg!(target_os = "android") {
                        41.
                    } else {
                        131.
                    }),
                    ..default()
                },
                kit::muted("", 10.),
                AuxReadout::Subtitle(slot),
            ));
            card.spawn(Node {
                position_type: PositionType::Absolute,
                left: px(7.),
                right: px(7.),
                bottom: px(6.),
                height: px(24.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: px(5.),
                ..default()
            })
            .with_children(|cost| {
                cost.spawn((
                    Node {
                        width: px(14.),
                        height: px(14.),
                        border: UiRect::all(px(2.)),
                        border_radius: BorderRadius::all(percent(50)),
                        ..default()
                    },
                    BorderColor::all(Color::srgb_u8(255, 228, 162)),
                    chrome::gradient(Color::srgb_u8(255, 233, 137), Color::srgb_u8(181, 115, 13)),
                    AuxCoinIcon(slot),
                ));
                cost.spawn((kit::muted("", 11.), AuxReadout::BadgePrefix(slot)));
                cost.spawn((chrome::title("", 16.), AuxReadout::BadgeValue(slot)));
            });
        });
}

pub(crate) fn can_hire(v: &MatchView, slot: usize) -> bool {
    if v.phase != Phase::Running || v.augment_round.is_some() || slot >= AUXILIARY_SLOTS {
        return false;
    }
    let me = v.you.index();
    let entry = v.auxiliaries[me][slot];
    entry.phase == AuxiliaryPhase::Available && v.energy >= AUXILIARY_RULE.cost_of(entry.kind)
}

pub(crate) fn header_text(slots: &[AuxiliarySlotView; AUXILIARY_SLOTS], lang: Lang) -> String {
    let hired = slots
        .iter()
        .filter(|s| s.phase != AuxiliaryPhase::Available)
        .count();
    match lang {
        Lang::PtBr => format!("Auxiliares ({hired}/{AUXILIARY_SLOTS})"),
        Lang::En => format!("Auxiliaries ({hired}/{AUXILIARY_SLOTS})"),
    }
}

pub(crate) fn slot_strings(
    entry: AuxiliarySlotView,
    lang: Lang,
) -> (&'static str, String, &'static str, String, bool) {
    let name = match (entry.kind, lang) {
        (AuxiliaryKind::Miner, Lang::PtBr) => "Mineiro",
        (AuxiliaryKind::Miner, Lang::En) => "Miner",
        (AuxiliaryKind::Merchant, Lang::PtBr) => "Mercador",
        (AuxiliaryKind::Merchant, Lang::En) => "Merchant",
    };
    let cost_num = (AUXILIARY_RULE.cost_of(entry.kind) / ENERGY_SCALE).to_string();
    match (entry.kind, entry.phase) {
        (AuxiliaryKind::Miner, AuxiliaryPhase::Available) => {
            let sub = match lang {
                Lang::PtBr => "+0.06/s · 180s".to_string(),
                Lang::En => "+0.06/s · 180s".to_string(),
            };
            let prefix = match lang {
                Lang::PtBr => "Custo",
                Lang::En => "Cost",
            };
            (name, sub, prefix, cost_num, true)
        }
        (AuxiliaryKind::Miner, AuxiliaryPhase::Preparing { ticks_left }) => {
            let secs = u32::from(ticks_left).div_ceil(TICK_HZ);
            let sub = match lang {
                Lang::PtBr => "Construindo mina".to_string(),
                Lang::En => "Building mine".to_string(),
            };
            let prefix = match lang {
                Lang::PtBr => "Prep",
                Lang::En => "Prep",
            };
            (name, sub, prefix, format!("{secs}s"), false)
        }
        (AuxiliaryKind::Miner, AuxiliaryPhase::Active { ticks_left, .. }) => {
            let secs = u32::from(ticks_left).div_ceil(TICK_HZ);
            let sub = "+0.06 Energia/s".to_string();
            let prefix = match lang {
                Lang::PtBr => "Ativo",
                Lang::En => "Active",
            };
            (name, sub, prefix, format!("{secs}s"), false)
        }
        (AuxiliaryKind::Merchant, AuxiliaryPhase::Available) => {
            let sub = match lang {
                Lang::PtBr => "3 compras → Reroll".to_string(),
                Lang::En => "3 buys → Reroll".to_string(),
            };
            let prefix = match lang {
                Lang::PtBr => "Custo",
                Lang::En => "Cost",
            };
            (name, sub, prefix, cost_num, true)
        }
        (
            AuxiliaryKind::Merchant,
            AuxiliaryPhase::Active {
                progress,
                activations_used,
                ..
            },
        ) => {
            let need = AUXILIARY_RULE.merchant_purchases_per_reroll;
            let max_act = AUXILIARY_RULE.merchant_max_activations;
            let rem_act = max_act.saturating_sub(activations_used);
            let sub = match lang {
                Lang::PtBr => format!("Restam {rem_act}/{max_act} cargas"),
                Lang::En => format!("{rem_act}/{max_act} charges left"),
            };
            let prefix = match lang {
                Lang::PtBr => "Compras",
                Lang::En => "Buys",
            };
            (name, sub, prefix, format!("{progress}/{need}"), false)
        }
        (_, AuxiliaryPhase::Completed) => {
            let sub = match lang {
                Lang::PtBr => "Turno encerrado".to_string(),
                Lang::En => "Shift complete".to_string(),
            };
            let val = match lang {
                Lang::PtBr => "Concluído".to_string(),
                Lang::En => "Done".to_string(),
            };
            (name, sub, "", val, false)
        }
        (AuxiliaryKind::Merchant, AuxiliaryPhase::Preparing { ticks_left }) => {
            let secs = u32::from(ticks_left).div_ceil(TICK_HZ);
            (name, String::new(), "Prep", format!("{secs}s"), false)
        }
    }
}

fn update(
    view: Res<LatestView>,
    pending: Res<PendingBuy>,
    lang: Res<Lang>,
    mut surfaces: Query<(&AuxCardSurface, &mut ImageNode, &mut UiAction)>,
    mut coins: Query<(&AuxCoinIcon, &mut Node)>,
    mut texts: Query<(&AuxReadout, &mut Text)>,
) {
    let Some(v) = &view.0 else { return };
    let me = v.you.index();
    let slots = &v.auxiliaries[me];

    for (surface, mut image, mut action) in &mut surfaces {
        let i = surface.0;
        let hireable = can_hire(v, i) && pending.0.is_none();
        *action = if hireable {
            UiAction::HireAuxiliary(i as u8)
        } else {
            UiAction::Disabled
        };
        image.color = match slots[i].phase {
            AuxiliaryPhase::Available if hireable => Color::WHITE,
            AuxiliaryPhase::Available => Color::srgb(0.65, 0.65, 0.65),
            AuxiliaryPhase::Preparing { .. } => Color::srgb(0.88, 0.84, 0.72),
            AuxiliaryPhase::Active { .. } => Color::srgb(0.86, 0.96, 0.88),
            AuxiliaryPhase::Completed => Color::srgb(0.52, 0.54, 0.58),
        };
    }

    for (coin, mut node) in &mut coins {
        let show_coin = slots[coin.0].phase == AuxiliaryPhase::Available;
        node.display = if show_coin {
            Display::Flex
        } else {
            Display::None
        };
    }

    for (readout, mut text) in &mut texts {
        let want = match readout {
            AuxReadout::Header => header_text(slots, *lang),
            #[cfg(target_os = "android")]
            AuxReadout::Toggle => format!(
                "Aux.\n{}/{}",
                slots
                    .iter()
                    .filter(|slot| slot.phase != AuxiliaryPhase::Available)
                    .count(),
                AUXILIARY_SLOTS
            ),
            AuxReadout::Name(i) => slot_strings(slots[*i], *lang).0.to_string(),
            AuxReadout::Subtitle(i) => slot_strings(slots[*i], *lang).1,
            AuxReadout::BadgePrefix(i) => slot_strings(slots[*i], *lang).2.to_string(),
            AuxReadout::BadgeValue(i) => slot_strings(slots[*i], *lang).3,
        };
        if text.0 != want {
            text.0 = want;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tbg_core::MatchState;

    #[test]
    fn auxiliary_hud_formats_both_slots_and_tracks_hire_eligibility() {
        let mut s = MatchState::new(7);
        s.phase = Phase::Running;
        s.players[0].energy = 5 * ENERGY_SCALE;
        let v = s.view_for(tbg_core::Seat::A, &[]);
        // With 5 Energy, Merchant (cost 4) is hireable and Miner (cost 6) is not.
        assert!(!can_hire(&v, 0));
        assert!(can_hire(&v, 1));
        assert_eq!(
            header_text(&v.auxiliaries[0], Lang::PtBr),
            "Auxiliares (0/2)"
        );
        let (miner_name, _, _, miner_cost, show_coin) =
            slot_strings(v.auxiliaries[0][0], Lang::PtBr);
        assert_eq!(miner_name, "Mineiro");
        assert_eq!(miner_cost, "6");
        assert!(show_coin);
        let (merch_name, _, _, merch_cost, _) = slot_strings(v.auxiliaries[0][1], Lang::PtBr);
        assert_eq!(merch_name, "Mercador");
        assert_eq!(merch_cost, "4");
    }
}
