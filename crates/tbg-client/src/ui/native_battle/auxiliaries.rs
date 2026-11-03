//! War Auxiliaries (`Mineiro` and `Mercador`) cards in the native battle HUD (P74, BR-AX12):
//! compact cards beside the five Shop cards, showing tier, status and the next Evolution price.

use super::{
    super::{hud::Hud, kit, PendingBuy, UiAction},
    chrome,
};
use crate::{i18n::Lang, sync::LatestView, GameState};
use bevy::prelude::*;
use tbg_core::{
    auxiliary::{
        AuxiliaryKind, AuxiliaryPhase, AuxiliarySlotView, AUXILIARY_MAX_TIER, AUXILIARY_RULE,
        AUXILIARY_SLOTS,
    },
    content::{ENERGY_SCALE, TICK_HZ},
    MatchView, Phase, Seat,
};

const CARD_W: f32 = 112.;
const CARD_H: f32 = 152.;
/// Gap between the last Shop card and the auxiliary panel.
const SHOP_GAP: f32 = 10.;

#[derive(Component)]
struct AuxCardSurface(usize);

#[derive(Component)]
struct AuxPortrait(usize);

#[derive(Component)]
struct AuxCoinIcon(usize);

#[derive(Component)]
struct AuxPip(usize, u8);

#[derive(Component)]
enum AuxReadout {
    Header,
    Name(usize),
    Subtitle(usize),
    BadgePrefix(usize),
    BadgeValue(usize),
}

pub fn plugin(app: &mut App) {
    app.add_systems(Update, update.run_if(in_state(GameState::InMatch)));
}

/// Spawned inside the scaled Shop column so it sits just right of the fifth card.
pub fn spawn(parent: &mut ChildSpawnerCommands, assets: &AssetServer) {
    parent
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: percent(100),
                bottom: px(0.),
                margin: UiRect::left(px(SHOP_GAP)),
                padding: UiRect::axes(px(6.), px(5.)),
                border: UiRect::all(px(2.)),
                border_radius: BorderRadius::all(px(9.)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(4.),
                ..default()
            },
            BackgroundColor(Color::srgba_u8(12, 22, 33, 236)),
            BorderColor::all(chrome::GOLD),
            Hud::PrivateControls,
        ))
        .with_children(|panel| {
            panel
                .spawn((
                    Node {
                        width: percent(100),
                        height: px(19.),
                        border: UiRect::bottom(px(1.)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor::all(chrome::GOLD.with_alpha(0.45)),
                ))
                .with_children(|hdr| {
                    hdr.spawn((chrome::title("", 13.), AuxReadout::Header));
                });
            panel
                .spawn(Node {
                    column_gap: px(6.),
                    ..default()
                })
                .with_children(|row| {
                    for slot in 0..AUXILIARY_SLOTS {
                        card(row, assets, slot);
                    }
                });
        });
}

fn absolute(left: f32, right: f32, top: f32, height: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: px(left),
        right: px(right),
        top: px(top),
        height: px(height),
        overflow: Overflow::clip(),
        ..default()
    }
}

fn card(parent: &mut ChildSpawnerCommands, assets: &AssetServer, slot: usize) {
    parent
        .spawn((
            Node {
                width: px(CARD_W),
                height: px(CARD_H),
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
                    top: px(6.),
                    left: px(5.),
                    width: px(CARD_W - 10.),
                    height: px(80.),
                    ..default()
                },
                ImageNode::new(assets.load(portrait_path(slot, Seat::A))),
                AuxPortrait(slot),
            ));
            card.spawn(Node {
                position_type: PositionType::Absolute,
                top: px(9.),
                right: px(9.),
                column_gap: px(3.),
                ..default()
            })
            .with_children(|pips| {
                for tier in 1..=AUXILIARY_MAX_TIER {
                    pips.spawn((
                        Node {
                            width: px(8.),
                            height: px(8.),
                            border: UiRect::all(px(1.5)),
                            border_radius: BorderRadius::all(percent(50)),
                            ..default()
                        },
                        BorderColor::all(chrome::GOLD),
                        BackgroundColor(chrome::INK),
                        AuxPip(slot, tier),
                    ));
                }
            });
            card.spawn(absolute(8., 6., 88., 17.))
                .with_children(|title| {
                    title.spawn((
                        chrome::title("", 12.),
                        TextLayout::no_wrap(),
                        AuxReadout::Name(slot),
                    ));
                });
            card.spawn(absolute(8., 6., 105., 14.))
                .with_children(|sub| {
                    sub.spawn((
                        kit::muted("", 9.),
                        TextLayout::no_wrap(),
                        AuxReadout::Subtitle(slot),
                    ));
                });
            card.spawn(Node {
                position_type: PositionType::Absolute,
                left: px(6.),
                right: px(6.),
                bottom: px(5.),
                height: px(20.),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: px(4.),
                ..default()
            })
            .with_children(|cost| {
                cost.spawn((
                    Node {
                        width: px(12.),
                        height: px(12.),
                        border: UiRect::all(px(2.)),
                        border_radius: BorderRadius::all(percent(50)),
                        ..default()
                    },
                    BorderColor::all(Color::srgb_u8(255, 228, 162)),
                    chrome::gradient(Color::srgb_u8(255, 233, 137), Color::srgb_u8(181, 115, 13)),
                    AuxCoinIcon(slot),
                ));
                cost.spawn((kit::muted("", 10.), AuxReadout::BadgePrefix(slot)));
                cost.spawn((chrome::title("", 14.), AuxReadout::BadgeValue(slot)));
            });
        });
}

/// Portraits are renders of the in-game low-poly figure in the viewer's team colors.
fn portrait_path(slot: usize, you: Seat) -> &'static str {
    match (slot, you) {
        (0, Seat::A) => "battle/portraits/aux_miner_blue.webp",
        (0, Seat::B) => "battle/portraits/aux_miner_red.webp",
        (_, Seat::A) => "battle/portraits/aux_merchant_blue.webp",
        (_, Seat::B) => "battle/portraits/aux_merchant_red.webp",
    }
}

/// Sub-energy price of the next tier for this slot, if any tier is left.
fn next_cost(entry: AuxiliarySlotView) -> Option<u32> {
    AUXILIARY_RULE.cost_of(entry.kind, entry.tier + 1)
}

pub(crate) fn can_hire(v: &MatchView, slot: usize) -> bool {
    if v.phase != Phase::Running || v.augment_round.is_some() || slot >= AUXILIARY_SLOTS {
        return false;
    }
    next_cost(v.auxiliaries[v.you.index()][slot]).is_some_and(|cost| v.energy >= cost)
}

pub(crate) fn header_text(slots: &[AuxiliarySlotView; AUXILIARY_SLOTS], lang: Lang) -> String {
    let hired = slots.iter().filter(|s| s.tier > 0).count();
    match lang {
        Lang::PtBr => format!("Auxiliares ({hired}/{AUXILIARY_SLOTS})"),
        Lang::En => format!("Auxiliaries ({hired}/{AUXILIARY_SLOTS})"),
    }
}

/// Everything one card shows.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct SlotText {
    pub name: &'static str,
    pub sub: String,
    pub prefix: &'static str,
    pub value: String,
    pub coin: bool,
}

fn tier_index(tier: u8) -> usize {
    usize::from(tier.clamp(1, AUXILIARY_MAX_TIER) - 1)
}

fn miner_rate(tier: u8) -> String {
    let per_sec = AUXILIARY_RULE.miner_sub_energy_per_sec[tier_index(tier)];
    format!("+{:.2}/s", per_sec as f32 / ENERGY_SCALE as f32)
}

fn merchant_need(tier: u8) -> u8 {
    AUXILIARY_RULE.merchant_purchases_per_activation[tier_index(tier)]
}

/// Status line for a hired slot.
fn status(entry: AuxiliarySlotView, lang: Lang) -> String {
    let pt = lang == Lang::PtBr;
    match (entry.kind, entry.phase) {
        (_, AuxiliaryPhase::Completed) => (if pt {
            "Turno encerrado"
        } else {
            "Shift complete"
        })
        .into(),
        (_, AuxiliaryPhase::Preparing { ticks_left }) => {
            let secs = u32::from(ticks_left).div_ceil(TICK_HZ);
            if pt {
                format!("Construindo {secs}s")
            } else {
                format!("Building {secs}s")
            }
        }
        (AuxiliaryKind::Miner, AuxiliaryPhase::Active { ticks_left, .. }) => {
            let secs = u32::from(ticks_left).div_ceil(TICK_HZ);
            format!("{} · {secs}s", miner_rate(entry.tier))
        }
        (
            AuxiliaryKind::Merchant,
            AuxiliaryPhase::Active {
                progress,
                activations_used,
                ..
            },
        ) => {
            let cap = AUXILIARY_RULE.merchant_max_activations[tier_index(entry.tier)];
            let left = cap.saturating_sub(activations_used);
            let need = merchant_need(entry.tier);
            if pt {
                format!("Compras {progress}/{need} · {left} cargas")
            } else {
                format!("Buys {progress}/{need} · {left} charges")
            }
        }
        (_, AuxiliaryPhase::Available) => String::new(),
    }
}

/// What tier 1 offers, shown before hiring.
fn offer(kind: AuxiliaryKind, lang: Lang) -> String {
    match (kind, lang) {
        (AuxiliaryKind::Miner, _) => format!("{} · 180s", miner_rate(1)),
        (AuxiliaryKind::Merchant, Lang::PtBr) => format!("{} compras → Reroll", merchant_need(1)),
        (AuxiliaryKind::Merchant, Lang::En) => format!("{} buys → Reroll", merchant_need(1)),
    }
}

pub(crate) fn slot_strings(entry: AuxiliarySlotView, lang: Lang) -> SlotText {
    let pt = lang == Lang::PtBr;
    let name = match (entry.kind, pt) {
        (AuxiliaryKind::Miner, true) => "Mineiro",
        (AuxiliaryKind::Miner, false) => "Miner",
        (AuxiliaryKind::Merchant, true) => "Mercador",
        (AuxiliaryKind::Merchant, false) => "Merchant",
    };
    let price = next_cost(entry).map(|c| (c / ENERGY_SCALE).to_string());
    match (entry.tier, price) {
        (0, Some(value)) => SlotText {
            name,
            sub: offer(entry.kind, lang),
            prefix: if pt { "Custo" } else { "Cost" },
            value,
            coin: true,
        },
        (_, Some(value)) => SlotText {
            name,
            sub: status(entry, lang),
            prefix: if pt { "Evoluir" } else { "Evolve" },
            value,
            coin: true,
        },
        (_, None) => SlotText {
            name,
            sub: status(entry, lang),
            prefix: if pt { "Nível" } else { "Tier" },
            value: "III".into(),
            coin: false,
        },
    }
}

fn card_tint(entry: AuxiliarySlotView, actionable: bool) -> Color {
    match (entry.tier, entry.phase, actionable) {
        (_, _, true) => Color::WHITE,
        (0, _, false) => Color::srgb(0.65, 0.65, 0.65),
        (_, AuxiliaryPhase::Completed, false) => Color::srgb(0.52, 0.54, 0.58),
        (_, AuxiliaryPhase::Preparing { .. }, false) => Color::srgb(0.88, 0.84, 0.72),
        _ => Color::srgb(0.86, 0.96, 0.88),
    }
}

#[allow(clippy::too_many_arguments)] // One HUD pass over the card widgets it owns.
fn update(
    view: Res<LatestView>,
    pending: Res<PendingBuy>,
    lang: Res<Lang>,
    assets: Res<AssetServer>,
    mut surfaces: Query<(&AuxCardSurface, &mut ImageNode, &mut UiAction), Without<AuxPortrait>>,
    mut portraits: Query<(&AuxPortrait, &mut ImageNode), Without<AuxCardSurface>>,
    mut coins: Query<(&AuxCoinIcon, &mut Node), Without<AuxPip>>,
    mut pips: Query<(&AuxPip, &mut BackgroundColor)>,
    mut texts: Query<(&AuxReadout, &mut Text)>,
) {
    let Some(v) = &view.0 else { return };
    let slots = &v.auxiliaries[v.you.index()];

    for (surface, mut image, mut action) in &mut surfaces {
        let i = surface.0;
        let actionable = can_hire(v, i) && pending.0.is_none();
        *action = if actionable {
            UiAction::HireAuxiliary(i as u8)
        } else {
            UiAction::Disabled
        };
        image.color = card_tint(slots[i], actionable);
    }
    for (portrait, mut image) in &mut portraits {
        let handle = assets.load(portrait_path(portrait.0, v.you));
        if image.image != handle {
            image.image = handle;
        }
    }
    for (coin, mut node) in &mut coins {
        let shown = slot_strings(slots[coin.0], *lang).coin;
        node.display = if shown { Display::Flex } else { Display::None };
    }
    for (pip, mut fill) in &mut pips {
        let reached = slots[pip.0].tier >= pip.1;
        fill.0 = if reached { chrome::GOLD } else { chrome::INK };
    }
    for (readout, mut text) in &mut texts {
        let want = match readout {
            AuxReadout::Header => header_text(slots, *lang),
            AuxReadout::Name(i) => slot_strings(slots[*i], *lang).name.to_string(),
            AuxReadout::Subtitle(i) => slot_strings(slots[*i], *lang).sub,
            AuxReadout::BadgePrefix(i) => slot_strings(slots[*i], *lang).prefix.to_string(),
            AuxReadout::BadgeValue(i) => slot_strings(slots[*i], *lang).value,
        };
        if text.0 != want {
            text.0 = want;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tbg_core::{MatchState, PlayerIntention};

    #[test]
    fn br_ax12_cards_show_hire_then_evolution_prices_and_cap_at_tier_three() {
        let mut s = MatchState::new(7);
        s.phase = Phase::Running;
        s.players[0].energy = 7 * ENERGY_SCALE;
        let v = s.view_for(Seat::A, &[]);
        // 7 Energy: the Merchant (6) is hireable, the Miner (8) is not.
        assert!(!can_hire(&v, 0));
        assert!(can_hire(&v, 1));
        assert_eq!(
            header_text(&v.auxiliaries[0], Lang::PtBr),
            "Auxiliares (0/2)"
        );
        let miner = slot_strings(v.auxiliaries[0][0], Lang::PtBr);
        assert_eq!(
            (miner.name, miner.prefix, miner.value.as_str(), miner.coin),
            ("Mineiro", "Custo", "8", true)
        );

        s.players[0].energy = 100 * ENERGY_SCALE;
        s.apply_intention(Seat::A, 1, PlayerIntention::HireAuxiliary { slot: 1 })
            .unwrap();
        let v = s.view_for(Seat::A, &[]);
        let merchant = slot_strings(v.auxiliaries[0][1], Lang::En);
        assert_eq!((merchant.prefix, merchant.value.as_str()), ("Evolve", "10"));
        assert_eq!(
            header_text(&v.auxiliaries[0], Lang::En),
            "Auxiliaries (1/2)"
        );
        for seq in 2..4 {
            s.apply_intention(Seat::A, seq, PlayerIntention::HireAuxiliary { slot: 1 })
                .unwrap();
        }
        let v = s.view_for(Seat::A, &[]);
        let maxed = slot_strings(v.auxiliaries[0][1], Lang::PtBr);
        assert_eq!(
            (maxed.prefix, maxed.value.as_str(), maxed.coin),
            ("Nível", "III", false)
        );
        assert!(!can_hire(&v, 1));
    }
}
