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
                height: px(43),
                padding: UiRect::axes(px(15), px(5)),
                column_gap: px(14),
                align_items: AlignItems::Center,
                ..default()
            },
            ImageNode {
                image_mode: NodeImageMode::Stretch,
                visual_box: bevy::ui::VisualBox::BorderBox,
                ..ImageNode::new(assets.load("battle/native/resources.webp"))
            },
        ))
        .with_children(|bar| {
            bar.spawn(chrome::title("Energy", 17.));
            bar.spawn((
                Node {
                    flex_grow: 1.,
                    min_width: px(70),
                    max_width: px(240),
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
            bar.spawn((
                label("", 13.),
                Node {
                    min_width: px(36),
                    ..default()
                },
                Readout::Energy,
            ));
            divider(bar);
            bar.spawn(chrome::title("Interest", 17.));
            bar.spawn((
                Text::new(""),
                TextFont::from_font_size(FontSize::Px(13.)),
                TextColor(Color::srgb_u8(200, 240, 160)),
                Readout::Interest,
            ));
            bar.spawn((kit::muted("", 9.), Readout::Band));
            divider(bar);
            bar.spawn(chrome::icon(assets, "friends", 27., chrome::CREAM));
            bar.spawn((chrome::title("", 17.), Readout::Army));
            bar.spawn((
                Node {
                    min_width: px(150),
                    height: px(30),
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
                button.spawn((label("", 13.), Readout::Reroll));
            });
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
            Readout::Interest => format!("×{:.2}", v.regen_bp as f32 / 10_000.),
            Readout::Band => format!(
                "{}/{}",
                (v.energy / INTEREST_BAND).min(INTEREST_MAX_BANDS),
                INTEREST_MAX_BANDS
            ),
            Readout::Army => army_label(v, *lang),
            Readout::Reroll => match v.reroll_cost {
                Some(cost) => format!(
                    "{} · {} Energy",
                    lang.key("shop.reroll"),
                    cost / ENERGY_SCALE
                ),
                None => lang.key("shop.reroll"),
            },
        };
        if text.0 != want {
            text.0 = want;
        }
    }
    let enabled = v.phase == Phase::Running
        && v.augment_round.is_none()
        && pending.0.is_none()
        && v.reroll_cost.is_some_and(|cost| v.energy >= cost);
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
