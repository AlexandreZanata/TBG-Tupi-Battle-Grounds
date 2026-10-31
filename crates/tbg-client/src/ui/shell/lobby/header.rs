//! Browser-style wordmark, account chip, navigation, presence and sound controls.

use super::{chrome, kit, Account, Action, Data, Lang, Panel, UiAction};
use bevy::prelude::*;

pub fn spawn(
    root: &mut ChildSpawnerCommands,
    data: &Data,
    account: &Account,
    assets: &AssetServer,
    lang: Lang,
    muted: bool,
) {
    root.spawn(Node {
        align_items: AlignItems::Center,
        column_gap: px(10),
        ..chrome::region(2., 2.4, 96., 10.4)
    })
    .with_children(|header| {
        header
            .spawn(Node {
                width: percent(18),
                height: percent(130),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            })
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
        account_chip(header, account, assets, lang);
        header
            .spawn((
                Node {
                    height: percent(100),
                    flex_grow: 1.,
                    flex_basis: px(0),
                    min_width: px(0),
                    border: UiRect::bottom(px(3)),
                    ..default()
                },
                BackgroundColor(chrome::METAL),
                BorderColor::all(chrome::EDGE),
            ))
            .with_children(|tabs| {
                for (key, icon, panel) in [
                    ("lobby", "home", Some(Panel::Lobby)),
                    ("arcanas", "arcana", Some(Panel::Arcanas)),
                    ("stronghold", "tower", Some(Panel::Strongholds)),
                    ("arenas", "island", Some(Panel::Arenas)),
                    ("troops", "helmet", None),
                    ("ranking", "trophy", None),
                    ("settings", "settings", Some(Panel::Settings)),
                ] {
                    let enabled = key != "troops";
                    let mut tab = tabs.spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            height: percent(100),
                            flex_grow: 1.,
                            flex_basis: px(0),
                            min_width: px(0),
                            row_gap: px(5),
                            border: UiRect::bottom(px(2)),
                            ..default()
                        },
                        chrome::skin(chrome::METAL, enabled, key == "lobby"),
                        if key == "ranking" {
                            UiAction::OpenRanking
                        } else {
                            UiAction::Disabled
                        },
                    ));
                    if let Some(panel) = panel {
                        tab.insert(Action::Open(panel));
                    }
                    tab.with_children(|t| {
                        t.spawn(chrome::icon(
                            assets,
                            icon,
                            24.,
                            if key == "lobby" {
                                kit::GOLD
                            } else {
                                kit::MUTED
                            },
                        ));
                        t.spawn(kit::muted(
                            lang.key(&format!("lobby.tabs.{key}")).to_uppercase(),
                            11.,
                        ));
                        if !enabled {
                            t.spawn(kit::muted(lang.key("lobby.tabs.locked"), 8.));
                        }
                    });
                }
            });
        let online = data.live.as_ref().map_or_else(
            || lang.key("live.chip_loading"),
            |v| {
                lang.key("live.chip")
                    .replace("{online}", &v.players.to_string())
                    .replace("{battle}", &v.battle.to_string())
            },
        );
        chip(header, online, UiAction::Disabled).insert(Action::Open(Panel::Live));
        let text = format!(
            "{} {}",
            lang.key("lobby.sound.label"),
            lang.key(if muted {
                "lobby.sound.off"
            } else {
                "lobby.sound.on"
            })
        );
        chip(header, text, UiAction::ToggleSound);
    });
}

fn chip<'a>(
    root: &'a mut ChildSpawnerCommands,
    text: String,
    action: UiAction,
) -> EntityCommands<'a> {
    root.spawn((
        Node {
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            padding: UiRect::all(px(9)),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(4)),
            ..default()
        },
        chrome::skin(chrome::METAL, true, false),
        action,
        children![kit::muted(text, 12.)],
    ))
}

fn account_chip(
    root: &mut ChildSpawnerCommands,
    account: &Account,
    assets: &AssetServer,
    lang: Lang,
) {
    root.spawn((
        Node {
            width: percent(12),
            min_width: px(0),
            height: percent(78),
            align_items: AlignItems::Center,
            column_gap: px(9),
            padding: UiRect::all(px(7)),
            border: UiRect::all(px(1)),
            ..default()
        },
        BackgroundColor(chrome::METAL),
        BorderColor::all(chrome::EDGE),
    ))
    .with_children(|chip| {
        chip.spawn(kit::image(assets, "branding/tbg-emblem.webp", 34., 34.));
        chip.spawn(chrome::column()).with_children(|copy| {
            copy.spawn(kit::heading(
                account
                    .profile
                    .as_ref()
                    .map_or_else(|| lang.key("lobby.guest.name"), |p| p.name.clone()),
                18.,
            ));
            if account.profile.is_none() {
                for (key, action) in [
                    ("create", UiAction::CreateAccount),
                    ("key", UiAction::HaveKey),
                ] {
                    copy.spawn((
                        Node {
                            padding: UiRect::vertical(px(1)),
                            ..default()
                        },
                        action,
                        children![kit::muted(lang.key(&format!("lobby.account.{key}")), 11.)],
                    ));
                }
            } else {
                copy.spawn(kit::muted(
                    account.profile.as_ref().map_or(0, |p| p.rating).to_string(),
                    12.,
                ));
            }
        });
    });
}
