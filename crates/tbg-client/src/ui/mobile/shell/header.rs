//! Compact landscape navigation reuses the Linux heraldic skin and shared screen actions.

use crate::{
    i18n::Lang,
    ui::{
        account::Account,
        kit,
        shell::{mobile_chrome as chrome, Action, Panel},
        UiAction,
    },
};
use bevy::prelude::*;

pub fn spawn(
    root: &mut ChildSpawnerCommands,
    assets: &AssetServer,
    lang: Lang,
    account: &Account,
    panel: Panel,
    ranking: bool,
) {
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(8),
            right: px(8),
            top: px(0),
            height: px(56),
            align_items: AlignItems::Stretch,
            border: UiRect::bottom(px(2)),
            ..default()
        },
        BackgroundColor(chrome::METAL),
        BorderColor::all(chrome::EDGE),
        ZIndex(10),
    ))
    .with_children(|bar| {
        bar.spawn((
            Node {
                width: px(76),
                min_width: px(48),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::all(px(4)),
                ..default()
            },
            chrome::skin(chrome::METAL, true, panel == Panel::Lobby && !ranking),
            UiAction::Disabled,
            Action::Open(Panel::Lobby),
        ))
        .with_children(|logo| {
            logo.spawn((
                ImageNode::new(assets.load("lobby/wordmark.webp")),
                Node {
                    width: percent(100),
                    height: Val::Auto,
                    aspect_ratio: Some(2.5),
                    ..default()
                },
            ));
        });
        for (key, icon, page) in [
            ("arcanas", "arcana", Some(Panel::Arcanas)),
            ("stronghold", "tower", Some(Panel::Strongholds)),
            ("arenas", "island", Some(Panel::Arenas)),
            ("troops", "helmet", Some(Panel::Troops)),
            ("ranking", "trophy", None),
            ("settings", "settings", Some(Panel::Settings)),
        ] {
            let active = page.map_or(ranking, |p| panel == p && !ranking);
            let mut tab = bar.spawn((
                Node {
                    min_width: px(48),
                    flex_grow: 1.,
                    flex_basis: px(0),
                    height: percent(100),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    row_gap: px(2),
                    padding: UiRect::axes(px(3), px(2)),
                    border: UiRect::right(px(1)),
                    ..default()
                },
                chrome::skin(chrome::METAL, true, active),
                if page.is_none() {
                    UiAction::OpenRanking
                } else {
                    UiAction::Disabled
                },
            ));
            if let Some(page) = page {
                tab.insert(Action::Open(page));
            }
            tab.with_children(|t| {
                t.spawn(chrome::icon(
                    assets,
                    icon,
                    18.,
                    if active { kit::GOLD } else { kit::MUTED },
                ));
                t.spawn((
                    kit::muted(lang.key(&format!("lobby.tabs.{key}")), 12.),
                    TextLayout::justify(Justify::Center),
                ));
            });
        }
        bar.spawn((
            Node {
                width: px(80),
                min_width: px(48),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: px(2),
                padding: UiRect::all(px(4)),
                overflow: Overflow::clip(),
                ..default()
            },
            chrome::skin(chrome::METAL, true, false),
            UiAction::Disabled,
            Action::Open(Panel::Settings),
        ))
        .with_children(|a| {
            a.spawn(chrome::icon(assets, "account-user", 18., kit::GOLD));
            a.spawn((
                kit::muted(
                    account
                        .profile
                        .as_ref()
                        .map(|p| p.name.clone())
                        .unwrap_or_else(|| lang.key("lobby.guest.name")),
                    12.,
                ),
                TextLayout::justify(Justify::Center),
            ));
        });
    });
}
