//! Native Shop reservation controls, reading charges and sending existing intentions.

use super::{
    super::{kit, PendingBuy, UiAction},
    chrome,
};
use crate::{i18n::Lang, sync::LatestView, GameState};
use bevy::prelude::*;
use tbg_core::{ruleset::ShopRule, MatchView, Phase};

#[derive(Component)]
struct Lock(usize);
#[derive(Component)]
struct Icon(usize);
#[derive(Component)]
struct Badge(usize);
#[derive(Component)]
struct Highlight(usize);

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (controls, badges, icons, highlights).run_if(in_state(GameState::InMatch)),
    );
}

pub fn spawn(parent: &mut ChildSpawnerCommands, assets: &AssetServer, slot: usize) {
    parent.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            border: UiRect::all(px(2)),
            border_radius: BorderRadius::all(px(7)),
            ..default()
        },
        BorderColor::all(chrome::GOLD),
        Visibility::Hidden,
        Highlight(slot),
    ));
    parent
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: px(8),
                top: px(12),
                width: px(27),
                height: px(28),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(5)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(chrome::INK),
            BorderColor::all(chrome::GOLD),
            UiAction::Disabled,
            Lock(slot),
            ZIndex(4),
            kit::buttons::Chrome {
                enabled: false,
                focused: false,
            },
            kit::buttons::Palette {
                idle: chrome::INK,
                hover: Color::srgb_u8(50, 58, 60),
                pressed: Color::srgb_u8(25, 32, 38),
                border: chrome::GOLD,
            },
        ))
        .with_children(|button| {
            button.spawn((
                kit::image(assets, "battle/native/shop-lock-open.webp", 17., 19.),
                Icon(slot),
            ));
        });
    parent
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(10),
                right: px(10),
                top: px(46),
                height: px(20),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(3)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.15, 0.12, 0.06, 0.94)),
            BorderColor::all(chrome::GOLD),
            Badge(slot),
            ZIndex(3),
        ))
        .with_children(|badge| {
            badge.spawn(kit::heading("", 10.));
        });
}

fn action(v: &MatchView, pending: bool, slot: usize) -> UiAction {
    if slot >= v.offers.len()
        || pending
        || v.reroll_cost.is_none()
        || v.phase != Phase::Running
        || v.augment_round.is_some()
    {
        return UiAction::Disabled;
    }
    if v.reserve[slot] > 0 {
        return UiAction::Release(slot as u8);
    }
    let active = v.reserve.iter().filter(|charges| **charges > 0).count();
    if v.held_mask & (1 << slot) != 0 || active >= usize::from(ShopRule::default().reserve_slots) {
        return UiAction::Disabled;
    }
    UiAction::Reserve {
        offer: slot as u8,
        kind: v.offers[slot],
    }
}

fn controls(
    view: Res<LatestView>,
    pending: Res<PendingBuy>,
    mut controls: Query<(&Lock, &mut Node, &mut UiAction, &mut kit::buttons::Chrome)>,
) {
    let Some(v) = &view.0 else { return };
    for (lock, mut node, mut intent, mut chrome) in &mut controls {
        node.display = if v.reroll_cost.is_some() {
            Display::Flex
        } else {
            Display::None
        };
        *intent = action(v, pending.0.is_some(), lock.0);
        chrome.enabled = *intent != UiAction::Disabled;
    }
}

fn badges(
    view: Res<LatestView>,
    lang: Res<Lang>,
    mut badges: Query<(&Badge, &mut Node, &Children)>,
    mut texts: Query<&mut Text>,
) {
    let Some(v) = &view.0 else { return };
    for (badge, mut node, children) in &mut badges {
        let charges = v.reserve[badge.0];
        node.display = if charges > 0 {
            Display::Flex
        } else {
            Display::None
        };
        for child in children.iter() {
            if let Ok(mut text) = texts.get_mut(child) {
                text.0 = format!(
                    "{} {}",
                    lang.key("shop.reserved"),
                    "●".repeat(usize::from(charges))
                );
            }
        }
    }
}

fn icons(
    view: Res<LatestView>,
    pending: Res<PendingBuy>,
    assets: Res<AssetServer>,
    mut icons: Query<(&Icon, &mut ImageNode)>,
) {
    let Some(v) = &view.0 else { return };
    for (icon, mut image) in &mut icons {
        let kind = if v.reserve[icon.0] > 0 {
            "closed"
        } else {
            "open"
        };
        image.image = assets.load(format!("battle/native/shop-lock-{kind}.webp"));
        image.color = if action(v, pending.0.is_some(), icon.0) == UiAction::Disabled {
            Color::srgb(0.55, 0.55, 0.55)
        } else {
            Color::WHITE
        };
    }
}

fn highlights(view: Res<LatestView>, mut highlights: Query<(&Highlight, &mut Visibility)>) {
    let Some(v) = &view.0 else { return };
    for (highlight, mut visible) in &mut highlights {
        *visible = if v.reserve[highlight.0] > 0 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn br_s06_s10_s12_controls_follow_holds_expiry_and_pause() {
        let mut state =
            tbg_core::MatchState::with_preview(29, [tbg_core::ArcanaPreset::NEUTRAL; 2]);
        state.phase = Phase::Running;
        let mut v = state.view_for(tbg_core::Seat::A, &[]);
        assert_eq!(
            action(&v, false, 0),
            UiAction::Reserve {
                offer: 0,
                kind: v.offers[0]
            }
        );
        v.reserve[0] = 3;
        v.held_mask = 1;
        assert_eq!(action(&v, false, 0), UiAction::Release(0));
        v.reserve[1] = 3;
        assert_eq!(action(&v, false, 2), UiAction::Disabled);
        v.reserve[0] = 0;
        assert_eq!(action(&v, false, 0), UiAction::Disabled);
        assert_eq!(action(&v, true, 1), UiAction::Disabled);
        v.phase = Phase::Countdown;
        assert_eq!(action(&v, false, 1), UiAction::Disabled);
    }
}
