//! Framed Energy, interest, Army and Reroll strip above the native Shop.

use super::{
    super::{hud::Hud, hud_update::army_label, kit, label, PendingBuy, UiAction},
    chrome,
};
use crate::{i18n::Lang, sync::LatestView};
use bevy::prelude::*;
use tbg_core::{
    content::{ENERGY_SCALE, INTEREST_BAND, INTEREST_MAX_BANDS},
    Phase,
};

#[derive(Component)]
enum Readout {
    Energy,
    EnergyLabel,
    InterestLabel,
    Interest,
    Band,
    Army,
    Reroll,
}
#[derive(Component)]
struct Reroll;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, update.run_if(in_state(crate::GameState::InMatch)));
}

pub fn spawn(parent: &mut ChildSpawnerCommands, assets: &AssetServer) {
    parent
        .spawn((
            Node {
                width: percent(100),
                height: px(if cfg!(target_os = "android") { 48 } else { 43 }),
                padding: UiRect::axes(
                    px(if cfg!(target_os = "android") { 6 } else { 15 }),
                    px(if cfg!(target_os = "android") { 0 } else { 5 }),
                ),
                column_gap: px(if cfg!(target_os = "android") { 4 } else { 14 }),
                align_items: AlignItems::Center,
                ..default()
            },
            ImageNode {
                image_mode: NodeImageMode::Stretch,
                visual_box: bevy::ui::VisualBox::BorderBox,
                ..ImageNode::new(assets.load("battle/native/resources.webp"))
            },
            #[cfg(target_os = "android")]
            crate::camera::mobile::BelowBattle,
        ))
        .with_children(|bar| {
            #[cfg(target_os = "android")]
            bar.spawn(Node {
                width: px(48),
                min_width: px(48),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            })
            .with_children(|energy| {
                energy.spawn((chrome::title("", 10.), Readout::EnergyLabel));
                energy.spawn((label("", 13.), Readout::Energy));
            });
            #[cfg(not(target_os = "android"))]
            bar.spawn((
                chrome::title(
                    "Energy",
                    if cfg!(target_os = "android") {
                        11.
                    } else {
                        17.
                    },
                ),
                Readout::EnergyLabel,
            ));
            bar.spawn((
                Node {
                    flex_grow: 1.,
                    min_width: px(if cfg!(target_os = "android") { 24 } else { 70 }),
                    max_width: px(if cfg!(target_os = "android") { 48 } else { 240 }),
                    height: px(21),
                    border: UiRect::all(px(2)),
                    border_radius: BorderRadius::all(px(20)),
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(chrome::INK),
                BorderColor::all(Color::srgb_u8(83, 97, 111)),
            ))
            .with_children(|track| {
                track.spawn((
                    Node {
                        width: percent(0),
                        height: percent(100),
                        border_radius: BorderRadius::all(px(15)),
                        ..default()
                    },
                    chrome::gradient(Color::srgb_u8(223, 156, 255), Color::srgb_u8(141, 37, 210)),
                    Hud::EnergyFill,
                ));
            });
            #[cfg(not(target_os = "android"))]
            bar.spawn((
                label("", 13.),
                Node {
                    min_width: px(36),
                    ..default()
                },
                Readout::Energy,
            ));
            divider(bar);
            bar.spawn((
                chrome::title(
                    "Interest",
                    if cfg!(target_os = "android") {
                        11.
                    } else {
                        17.
                    },
                ),
                Readout::InterestLabel,
            ));
            bar.spawn((
                Text::new(""),
                TextFont::from_font_size(FontSize::Px(13.)),
                TextColor(Color::srgb_u8(200, 240, 160)),
                Readout::Interest,
            ));
            bar.spawn((kit::muted("", 9.), Readout::Band));
            divider(bar);
            bar.spawn(chrome::icon(
                assets,
                "friends",
                if cfg!(target_os = "android") {
                    18.
                } else {
                    27.
                },
                chrome::CREAM,
            ));
            bar.spawn((
                #[cfg(target_os = "android")]
                Node {
                    flex_grow: 1.,
                    flex_basis: px(0),
                    min_width: px(0),
                    max_height: percent(100),
                    height: px(48),
                    padding: UiRect::all(px(4)),
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(4)),
                    ..default()
                },
                #[cfg(target_os = "android")]
                BorderColor::all(chrome::GOLD),
                #[cfg(target_os = "android")]
                UiAction::ToggleBattleDetails,
                #[cfg(target_os = "android")]
                TextLayout::new(Justify::Left, LineBreak::WordOrCharacter),
                chrome::title(
                    "",
                    if cfg!(target_os = "android") {
                        11.
                    } else {
                        17.
                    },
                ),
                Readout::Army,
            ));
            bar.spawn((
                Node {
                    min_width: px(if cfg!(target_os = "android") {
                        112
                    } else {
                        150
                    }),
                    height: px(if cfg!(target_os = "android") { 48 } else { 30 }),
                    padding: UiRect::horizontal(px(8)),
                    border: UiRect::all(px(2)),
                    border_radius: BorderRadius::all(px(6)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(chrome::INK),
                BorderColor::all(chrome::GOLD),
                UiAction::Reroll,
                Reroll,
                kit::buttons::Chrome {
                    enabled: true,
                    focused: false,
                },
                kit::buttons::Palette {
                    idle: chrome::INK,
                    hover: Color::srgb_u8(40, 53, 58),
                    pressed: Color::srgb_u8(25, 35, 42),
                    border: chrome::GOLD,
                },
            ))
            .with_children(|button| {
                button.spawn((
                    #[cfg(target_os = "android")]
                    Node {
                        width: percent(100),
                        min_width: px(0),
                        ..default()
                    },
                    #[cfg(target_os = "android")]
                    TextLayout::new(Justify::Center, LineBreak::WordOrCharacter),
                    label("", 13.),
                    Readout::Reroll,
                ));
            });
            #[cfg(target_os = "android")]
            super::auxiliaries::spawn_toggle(bar);
        });
}

fn divider(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        Node {
            width: px(1),
            height: px(26),
            ..default()
        },
        BackgroundColor(Color::WHITE.with_alpha(0.15)),
    ));
}

fn update(
    view: Res<LatestView>,
    pending: Res<PendingBuy>,
    lang: Res<Lang>,
    mut texts: Query<(&Readout, &mut Text)>,
    mut rerolls: Query<(&mut UiAction, &mut kit::buttons::Chrome), With<Reroll>>,
) {
    let Some(v) = &view.0 else { return };
    for (kind, mut text) in &mut texts {
        let want = match kind {
            Readout::Energy => energy_text(v.energy),
            Readout::EnergyLabel => {
                if cfg!(target_os = "android") {
                    lang.key("battle.energy")
                } else {
                    "Energy".into()
                }
            }
            Readout::InterestLabel => {
                if cfg!(target_os = "android") {
                    lang.key("battle.interest")
                } else {
                    "Interest".into()
                }
            }
            Readout::Interest => format!("×{:.2}", v.regen_bp as f32 / 10_000.),
            Readout::Band => format!(
                "{}/{}",
                (v.energy / INTEREST_BAND).min(INTEREST_MAX_BANDS),
                INTEREST_MAX_BANDS
            ),
            Readout::Army => army_label(v, *lang),
            Readout::Reroll => {
                if v.free_rerolls > 0 {
                    let free_tag = match *lang {
                        Lang::PtBr => "GRÁTIS",
                        Lang::En => "FREE",
                    };
                    format!(
                        "{} · {} ({})",
                        lang.key("shop.reroll"),
                        free_tag,
                        v.free_rerolls
                    )
                } else {
                    match v.reroll_cost {
                        Some(0) => {
                            let free_tag = match *lang {
                                Lang::PtBr => "GRÁTIS",
                                Lang::En => "FREE",
                            };
                            format!("{} · {}", lang.key("shop.reroll"), free_tag)
                        }
                        Some(cost) if cfg!(target_os = "android") => format!(
                            "{} · {}",
                            lang.key("shop.reroll"),
                            lang.key("common.energy_amount")
                                .replace("{cost}", &(cost / ENERGY_SCALE).to_string())
                        ),
                        Some(cost) => format!(
                            "{} · {} Energy",
                            lang.key("shop.reroll"),
                            cost / ENERGY_SCALE
                        ),
                        None => lang.key("shop.reroll"),
                    }
                }
            }
        };
        if text.0 != want {
            text.0 = want;
        }
    }
    let effective_cost = if v.free_rerolls > 0 {
        Some(0)
    } else {
        v.reroll_cost
    };
    let enabled = v.phase == Phase::Running
        && v.augment_round.is_none()
        && pending.0.is_none()
        && effective_cost.is_some_and(|cost| v.energy >= cost);
    for (mut action, mut chrome) in &mut rerolls {
        chrome.enabled = enabled;
        *action = if enabled {
            UiAction::Reroll
        } else {
            UiAction::Disabled
        };
    }
}

fn energy_text(energy: u32) -> String {
    if energy.is_multiple_of(ENERGY_SCALE) {
        (energy / ENERGY_SCALE).to_string()
    } else {
        format!("{:.1}", energy as f32 / ENERGY_SCALE as f32)
    }
}
