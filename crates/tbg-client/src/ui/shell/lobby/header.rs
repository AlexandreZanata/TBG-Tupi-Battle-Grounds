//! One native header strip with consistent account, navigation and presence blocks.

use super::{chrome, kit, Account, Action, Data, Lang, Panel, UiAction};
use bevy::prelude::*;

#[cfg(not(target_os = "linux"))]
pub fn spawn(
    root: &mut ChildSpawnerCommands,
    data: &Data,
    account: &Account,
    assets: &AssetServer,
    lang: Lang,
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
            .with_children(|blocks| {
                account_block(blocks, account, assets, lang);
                navigation(blocks, assets, lang);
                presence_block(blocks, data, assets, lang);
            });
    });
}

#[cfg(target_os = "linux")]
pub fn spawn(
    root: &mut ChildSpawnerCommands,
    data: &Data,
    account: &Account,
    assets: &AssetServer,
    lang: Lang,
) {
    root.spawn((
        Node {
            align_items: AlignItems::Center,
            border: UiRect::bottom(px(3)),
            ..chrome::region(2., 2.4, 96., 10.4)
        },
        BackgroundColor(chrome::METAL),
        BorderColor::all(chrome::EDGE),
    ))
    .with_children(|header| {
        header
            .spawn((
                Node {
                    width: percent(13),
                    flex_grow: 0.,
                    flex_basis: Val::Auto,
                    padding: UiRect::axes(px(8), px(6)),
                    ..block()
                },
                BackgroundColor(chrome::METAL),
                BorderColor::all(chrome::EDGE),
            ))
            .with_children(|logo| {
                logo.spawn((
                    ImageNode::new(assets.load("lobby/wordmark.webp")),
                    Node {
                        width: percent(100),
                        height: percent(100),
                        ..default()
                    },
                ));
            });
        account_block(header, account, assets, lang);
        navigation(header, assets, lang);
        presence_block(header, data, assets, lang);
    });
}

/// Shared dimensions and dividers keep every header cell in the same strip.
fn block() -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        height: percent(100),
        flex_grow: 1.,
        flex_basis: px(0),
        min_width: px(0),
        min_height: px(0),
        row_gap: px(5),
        border: UiRect {
            right: px(1),
            bottom: px(2),
            ..default()
        },
        overflow: Overflow::clip(),
        ..default()
    }
}

fn navigation(root: &mut ChildSpawnerCommands, assets: &AssetServer, lang: Lang) {
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
        let mut tab = root.spawn((
            block(),
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
}

fn account_block(
    root: &mut ChildSpawnerCommands,
    account: &Account,
    assets: &AssetServer,
    lang: Lang,
) {
    root.spawn((
        Node {
            width: percent(16),
            flex_grow: 0.,
            flex_basis: Val::Auto,
            flex_direction: FlexDirection::Row,
            column_gap: px(8),
            padding: UiRect::all(px(8)),
            ..block()
        },
        BackgroundColor(chrome::METAL),
        BorderColor::all(chrome::EDGE),
    ))
    .with_children(|account_cell| {
        account_cell.spawn(kit::image(assets, "branding/tbg-emblem.webp", 32., 32.));
        account_cell
            .spawn(Node {
                flex_grow: 1.,
                flex_basis: px(0),
                ..chrome::column()
            })
            .with_children(|copy| {
                copy.spawn(kit::heading(
                    account
                        .profile
                        .as_ref()
                        .map_or_else(|| lang.key("lobby.guest.name"), |p| p.name.clone()),
                    16.,
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

fn presence_block(root: &mut ChildSpawnerCommands, data: &Data, assets: &AssetServer, lang: Lang) {
    let online = data.live.as_ref().map_or_else(
        || lang.key("live.chip_loading"),
        |v| {
            lang.key("live.chip")
                .replace("{online}", &v.players.to_string())
                .replace("{battle}", &v.battle.to_string())
        },
    );
    root.spawn((
        Node {
            width: percent(16),
            flex_grow: 0.,
            flex_basis: Val::Auto,
            padding: UiRect::horizontal(px(6)),
            ..block()
        },
        chrome::skin(chrome::METAL, true, false),
        UiAction::Disabled,
        Action::Open(Panel::Live),
    ))
    .with_children(|presence| {
        presence.spawn(chrome::icon(assets, "friends", 24., kit::MUTED));
        presence.spawn(kit::muted(online, 11.));
    });
}
