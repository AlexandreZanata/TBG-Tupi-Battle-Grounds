//! Main lobby profile, feature cards, news, modes and private-room entry.

use super::{button, data::Data, kit, Account, Action, Lang, Panel, Shell, UiAction};
use bevy::prelude::*;

pub fn spawn(
    root: &mut ChildSpawnerCommands,
    shell: &Shell,
    data: &Data,
    account: &Account,
    assets: &AssetServer,
    lang: Lang,
    compact: bool,
) {
    root.spawn(Node {
        flex_direction: if compact {
            FlexDirection::Column
        } else {
            FlexDirection::Row
        },
        column_gap: px(20),
        row_gap: px(16),
        width: percent(100),
        flex_grow: 1.,
        ..default()
    })
    .with_children(|columns| {
        profile(columns, data, account, assets, lang);
        columns
            .spawn(Node {
                flex_direction: FlexDirection::Column,
                flex_grow: 1.,
                flex_basis: px(0),
                min_width: px(360),
                row_gap: px(16),
                ..default()
            })
            .with_children(|center| {
                center.spawn(kit::panel()).with_children(|p| {
                    p.spawn(kit::heading(lang.key("lobby.feature.explore"), 26.));
                    p.spawn(Node {
                        flex_wrap: FlexWrap::Wrap,
                        column_gap: px(16),
                        row_gap: px(16),
                        ..default()
                    })
                    .with_children(|cards| {
                        for (key, art, panel) in [
                            ("arcanas", "arcanas-v2.webp", Panel::Arcanas),
                            ("strongholds", "strongholds-v2.webp", Panel::Strongholds),
                            ("ranking", "ranking-v2.webp", Panel::Lobby),
                        ] {
                            feature(cards, assets, lang, key, art, panel);
                        }
                        feature(
                            cards,
                            assets,
                            lang,
                            "arenas",
                            "troops-v2.webp",
                            Panel::Arenas,
                        );
                    });
                });
                center.spawn(kit::panel()).with_children(|p| {
                    p.spawn(kit::heading(lang.key("lobby.news.title"), 22.));
                    p.spawn(super::super::label(
                        lang.key(&format!("lobby.news.{}.title", shell.news)),
                        20.,
                    ));
                    p.spawn(kit::muted(
                        lang.key(&format!("lobby.news.{}.body", shell.news)),
                        16.,
                    ));
                    p.spawn(kit::row()).with_children(|r| {
                        r.spawn(button("<", Action::News(-1), 40.));
                        r.spawn(kit::muted(format!("{} / 3", shell.news + 1), 14.));
                        r.spawn(button(">", Action::News(1), 40.));
                    });
                });
            });
        modes(columns, account, lang);
    });
}

fn profile(
    root: &mut ChildSpawnerCommands,
    data: &Data,
    account: &Account,
    assets: &AssetServer,
    lang: Lang,
) {
    root.spawn(kit::panel()).with_children(|p| {
        p.spawn(kit::image(assets, "lobby/profile-v2.webp", 220., 180.));
        p.spawn(kit::heading(
            account
                .profile
                .as_ref()
                .map_or_else(|| lang.key("lobby.guest.name"), |p| p.name.clone()),
            26.,
        ));
        if let Some(profile) = &account.profile {
            p.spawn(kit::muted(
                format!("{} · {}", lang.key("native.rating"), profile.rating),
                16.,
            ));
            let stats = data.stats.as_ref();
            for key in ["wins", "losses", "winrate"] {
                let value = stats.map_or_else(
                    || "—".into(),
                    |s| {
                        if key == "winrate" {
                            format!("{:.0}%", s["win_rate"].as_f64().unwrap_or(0.))
                        } else {
                            s[key].as_u64().unwrap_or(0).to_string()
                        }
                    },
                );
                p.spawn(super::super::label(
                    format!("{}  {value}", lang.key(&format!("lobby.profile.{key}"))),
                    16.,
                ));
            }
            if let Some(rank) = stats.and_then(|s| s["rank"].as_u64()) {
                p.spawn(kit::heading(
                    lang.key("lobby.profile.global")
                        .replace("{rank}", &rank.to_string()),
                    18.,
                ));
            }
        } else {
            p.spawn(kit::muted(lang.key("lobby.profile.stats_guest"), 14.));
            p.spawn(kit::button(
                lang.key("lobby.account.create"),
                UiAction::CreateAccount,
                210.,
            ));
            p.spawn(kit::button(
                lang.key("lobby.account.key"),
                UiAction::HaveKey,
                210.,
            ));
        }
        let kind = crate::config::load_stronghold();
        p.spawn(button(
            format!("{} · {kind:?}", lang.key("lobby.profile.tower")),
            Action::Open(Panel::Strongholds),
            210.,
        ));
        p.spawn(button(
            lang.key("live.dialog.title"),
            Action::Open(Panel::Live),
            210.,
        ));
    });
}

fn feature(
    root: &mut ChildSpawnerCommands,
    assets: &AssetServer,
    lang: Lang,
    key: &str,
    art: &str,
    panel: Panel,
) {
    let mut card = root.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            flex_grow: 1.,
            min_width: px(210),
            flex_basis: px(210),
            max_width: px(360),
            padding: UiRect::all(px(8)),
            row_gap: px(8),
            border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(8)),
            ..default()
        },
        BackgroundColor(kit::INK),
        BorderColor::all(kit::LINE),
    ));
    if key == "ranking" {
        card.insert(UiAction::OpenRanking);
    } else {
        card.insert((UiAction::Disabled, Action::Open(panel)));
    }
    card.with_children(|p| {
        p.spawn((
            ImageNode::new(assets.load(format!("lobby/{art}"))),
            Node {
                width: percent(100),
                height: px(120),
                ..default()
            },
        ));
        let title = if key == "arenas" {
            lang.key("lobby.tabs.arenas")
        } else {
            lang.key(&format!("lobby.feature.{key}.title"))
        };
        p.spawn(kit::heading(title, 22.));
        if key != "arenas" {
            p.spawn(kit::muted(
                lang.key(&format!("lobby.feature.{key}.hint")),
                13.,
            ));
        }
    });
}

fn modes(root: &mut ChildSpawnerCommands, account: &Account, lang: Lang) {
    root.spawn(kit::panel()).with_children(|p| {
        p.spawn(kit::heading(lang.key("lobby.mode.choose"), 22.));
        for (key, action) in [
            (
                "ranked",
                if account.profile.is_some() {
                    UiAction::Ranked
                } else {
                    UiAction::CreateAccount
                },
            ),
            ("casual", UiAction::Quick),
            ("challenge", UiAction::CreateChallenge),
            ("practice", UiAction::Practice),
        ] {
            p.spawn(kit::button(
                lang.key(&format!("lobby.mode.{key}.title")),
                action,
                260.,
            ));
            let hint = if key == "ranked" && account.profile.is_none() {
                lang.key("lobby.mode.ranked_hint_guest")
            } else {
                lang.key(&format!("lobby.mode.{key}.hint"))
            };
            p.spawn(kit::muted(hint, 12.));
        }
        p.spawn(button(
            lang.key("lobby.mode.room.title"),
            Action::Open(Panel::Friends),
            260.,
        ));
    });
}

pub fn friends(root: &mut ChildSpawnerCommands, lang: Lang) {
    root.spawn(kit::panel()).with_children(|p| {
        p.spawn(kit::heading(lang.key("lobby.friend.title"), 32.));
        p.spawn(kit::muted(lang.key("lobby.friend.body"), 18.));
        p.spawn(kit::button(
            lang.key("lobby.friend.create"),
            UiAction::CreateRoom,
            320.,
        ));
        p.spawn(button(
            lang.key("lobby.friend.join"),
            Action::RoomInput,
            320.,
        ));
        p.spawn(kit::button(
            lang.key("lobby.mode.challenge.title"),
            UiAction::CreateChallenge,
            320.,
        ));
    });
}
