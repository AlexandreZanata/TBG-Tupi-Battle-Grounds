//! Bottom-left ranked record and Stronghold shortcut, matching the browser profile card.

use super::{button, chrome, kit, Account, Action, Data, Lang, Panel};
use bevy::prelude::*;

pub fn spawn(
    root: &mut ChildSpawnerCommands,
    data: &Data,
    account: &Account,
    assets: &AssetServer,
    lang: Lang,
) {
    #[cfg(target_os = "linux")]
    let pending = account.auto_login || account.pending.is_some() || data.stats_pending();
    root.spawn(chrome::panel([2., 68., 27.5, 27.]))
        .with_children(|p| {
            p.spawn((
                ImageNode {
                    color: Color::WHITE.with_alpha(0.22),
                    ..ImageNode::new(assets.load("lobby/profile-v2.webp"))
                },
                chrome::region(0., 0., 100., 100.),
            ));
            #[cfg(not(target_os = "linux"))]
            p.spawn((
                ImageNode::new(assets.load("branding/tbg-emblem.webp")),
                Node {
                    width: percent(27),
                    height: percent(60),
                    ..chrome::region(2., 6., 27., 60.)
                },
            ));
            #[cfg(target_os = "linux")]
            p.spawn(Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..chrome::region(2., 6., 27., 60.)
            })
            .with_children(|crest| {
                crest.spawn((
                    ImageNode::new(assets.load("lobby/crest-v2.webp")),
                    Node {
                        height: percent(100),
                        width: Val::Auto,
                        aspect_ratio: Some(0.75),
                        ..default()
                    },
                ));
            });
            p.spawn(button(
                format!(
                    "{}\n{:?}",
                    lang.key("lobby.profile.tower"),
                    crate::config::load_stronghold()
                ),
                Action::Open(Panel::Strongholds),
                88.,
            ))
            .insert(Node {
                position_type: PositionType::Absolute,
                top: px(6),
                right: px(6),
                width: percent(34),
                min_height: px(38),
                padding: UiRect::all(px(3)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(px(1)),
                ..default()
            });
            p.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(5),
                padding: UiRect::top(px(44)),
                ..chrome::region(34., 4., 62., 59.)
            })
            .with_children(|info| {
                let title = data
                    .stats
                    .as_ref()
                    .and_then(|s| s["rank"].as_u64())
                    .map_or_else(
                        || lang.key("lobby.profile.unranked"),
                        |rank| {
                            lang.key("lobby.profile.global")
                                .replace("{rank}", &rank.to_string())
                        },
                    );
                let mut title = info.spawn(kit::heading(title, 20.));
                title.insert(TextColor(Color::srgb_u8(245, 240, 224)));
                #[cfg(target_os = "linux")]
                title.insert((
                    kit::Pending(pending),
                    Node {
                        width: percent(100),
                        min_height: px(24),
                        ..default()
                    },
                ));
                let mut _rating = info.spawn(kit::heading(
                    account
                        .profile
                        .as_ref()
                        .map_or_else(|| "—".into(), |p| format!("{}", p.rating)),
                    18.,
                ));
                #[cfg(target_os = "linux")]
                _rating.insert((
                    kit::Pending(pending),
                    Node {
                        width: percent(65),
                        min_height: px(22),
                        ..default()
                    },
                ));
                info.spawn(kit::muted(
                    lang.key(if account.profile.is_some() {
                        "lobby.profile.ranked_note_member"
                    } else {
                        "lobby.profile.ranked_note_guest"
                    }),
                    12.,
                ));
            });
            p.spawn(Node {
                align_items: AlignItems::FlexStart,
                padding: UiRect::top(px(10)),
                border: UiRect::top(px(1)),
                ..chrome::region(4., 66., 92., 22.)
            })
            .insert(BorderColor::all(chrome::EDGE.with_alpha(0.4)))
            .with_children(|stats| {
                for key in ["wins", "losses", "winrate"] {
                    let value = if account.profile.is_none() {
                        "—".into()
                    } else {
                        data.stats.as_ref().map_or_else(
                            || "—".into(),
                            |s| {
                                if key == "winrate" {
                                    format!("{:.0}%", s["win_rate"].as_f64().unwrap_or(0.))
                                } else {
                                    s[key].as_u64().unwrap_or(0).to_string()
                                }
                            },
                        )
                    };
                    stats
                        .spawn(Node {
                            width: percent(33.33),
                            align_items: AlignItems::Center,
                            row_gap: px(5),
                            ..chrome::column()
                        })
                        .with_children(|stat| {
                            stat.spawn(kit::muted(lang.key(&format!("lobby.profile.{key}")), 12.));
                            let mut _value = stat.spawn(kit::heading(value, 21.));
                            #[cfg(target_os = "linux")]
                            _value.insert((
                                kit::Pending(pending),
                                TextLayout::justify(Justify::Center),
                                Node {
                                    width: if pending { percent(75) } else { Val::Auto },
                                    min_height: px(26),
                                    ..default()
                                },
                            ));
                        });
                }
            });
            p.spawn((
                kit::muted(
                    lang.key(if account.profile.is_some() {
                        "lobby.profile.ranked_note_member"
                    } else {
                        "lobby.profile.stats_guest"
                    }),
                    10.,
                ),
                chrome::region(5., 91., 92., 8.),
            ));
        });
}
