//! Bottom feature/news cards and the compact public live-match panel.

use super::{button, chrome, kit, Action, Data, Lang, Panel, UiAction};
use bevy::prelude::*;

pub fn features(root: &mut ChildSpawnerCommands, assets: &AssetServer, lang: Lang) {
    root.spawn(Node {
        column_gap: px(14),
        ..chrome::region(31., 68., 43., 27.)
    })
    .with_children(|cards| {
        for (key, panel) in [
            ("arcanas", Some(Panel::Arcanas)),
            ("strongholds", Some(Panel::Strongholds)),
            ("troops", None),
            ("ranking", None),
        ] {
            let enabled = key != "troops";
            let mut card = cards.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.,
                    flex_basis: px(0),
                    min_width: px(0),
                    height: percent(100),
                    overflow: Overflow::clip(),
                    border: UiRect::all(px(2)),
                    border_radius: BorderRadius::all(px(3)),
                    ..default()
                },
                chrome::skin(chrome::METAL, enabled, false),
                if key == "ranking" {
                    UiAction::OpenRanking
                } else {
                    UiAction::Disabled
                },
            ));
            if let Some(panel) = panel {
                card.insert(Action::Open(panel));
            }
            card.with_children(|p| {
                p.spawn(chrome::image(assets, &format!("lobby/{key}-v2.webp"), 60.));
                p.spawn(Node {
                    padding: UiRect::all(px(10)),
                    row_gap: px(5),
                    ..chrome::column()
                })
                .with_children(|copy| {
                    copy.spawn(kit::heading(
                        lang.key(&format!("lobby.feature.{key}.title"))
                            .to_uppercase(),
                        19.,
                    ));
                    copy.spawn(kit::muted(
                        lang.key(&format!("lobby.feature.{key}.hint")),
                        12.,
                    ));
                });
                if !enabled {
                    p.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            top: px(5),
                            right: px(5),
                            padding: UiRect::all(px(4)),
                            ..default()
                        },
                        BackgroundColor(chrome::METAL),
                        children![kit::muted(
                            lang.key("lobby.feature.troops.locked").to_uppercase(),
                            9.
                        )],
                    ));
                }
            });
        }
    });
}

pub fn news(root: &mut ChildSpawnerCommands, page: usize, assets: &AssetServer, lang: Lang) {
    root.spawn(chrome::panel([75., 68., 23., 27.]))
        .with_children(|p| {
            p.spawn(kit::heading(
                lang.key("lobby.news.title").to_uppercase(),
                14.,
            ));
            p.spawn(chrome::image(assets, "lobby/news.webp", 32.));
            p.spawn((
                kit::muted(lang.key(&format!("lobby.news.{page}.title")), 13.),
                Node {
                    margin: UiRect::top(px(5)),
                    ..default()
                },
            ));
            p.spawn(kit::muted(
                lang.key(&format!("lobby.news.{page}.body")),
                12.,
            ));
            p.spawn(Node {
                flex_grow: 1.,
                min_height: px(0),
                ..default()
            });
            p.spawn(Node {
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|r| {
                r.spawn(news_arrow("<", -1));
                r.spawn(kit::muted(format!("{} / 3", page + 1), 12.));
                r.spawn(news_arrow(">", 1));
            });
        });
}

pub fn live(root: &mut ChildSpawnerCommands, data: &Data, lang: Lang) {
    root.spawn(chrome::panel([75., 17., 23., 50.]))
        .with_children(|p| {
            let count = data.live.as_ref().map_or(0, |v| v.matches.len());
            let mut _title = p.spawn(kit::heading(
                lang.key("live.panel.title")
                    .replace("{n}", &count.to_string())
                    .to_uppercase(),
                14.,
            ));
            #[cfg(target_os = "linux")]
            _title.insert((
                kit::Pending(data.live_pending()),
                Node {
                    width: percent(100),
                    min_height: px(18),
                    ..default()
                },
            ));
            p.spawn(Node {
                flex_grow: 1.,
                min_height: px(0),
                overflow: Overflow::clip_y(),
                row_gap: px(12),
                ..chrome::column()
            })
            .with_children(|rows| {
                #[cfg(target_os = "linux")]
                if data.live_pending() {
                    crate::ui::native_loading::rows(rows, 3);
                }
                for battle in data
                    .live
                    .as_ref()
                    .into_iter()
                    .flat_map(|v| v.matches.iter())
                    .take(2)
                {
                    rows.spawn((
                        Node {
                            padding: UiRect::all(px(10)),
                            row_gap: px(5),
                            border: UiRect::all(px(1)),
                            ..chrome::column()
                        },
                        chrome::skin(chrome::METAL, true, false),
                        UiAction::Watch(battle.id),
                    ))
                    .with_children(|row| {
                        row.spawn(kit::heading(
                            format!("{} vs {}", battle.names[0], battle.names[1]),
                            17.,
                        ));
                        row.spawn(kit::muted(
                            format!(
                                "{} · {}:{:02}",
                                lang.key(&format!("arenas.{}.name", battle.arena)),
                                battle.elapsed / 60,
                                battle.elapsed % 60
                            ),
                            12.,
                        ));
                        row.spawn(kit::muted(lang.key("live.watch"), 13.));
                    });
                }
            });
            if count == 0 {
                p.spawn((
                    kit::muted(
                        lang.key(if data.unavailable {
                            "live.unavailable"
                        } else if data.live.is_none() {
                            "live.loading"
                        } else {
                            "live.none"
                        }),
                        13.,
                    ),
                    Node {
                        margin: UiRect::bottom(px(12)),
                        ..default()
                    },
                ));
                p.spawn(kit::button(
                    lang.key("live.play_casual"),
                    UiAction::Quick,
                    150.,
                ))
                .insert(Node {
                    width: percent(100),
                    min_height: px(42),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border: UiRect::all(px(1)),
                    ..default()
                });
            }
            p.spawn(Node {
                justify_content: JustifyContent::FlexEnd,
                margin: UiRect::top(px(8)),
                ..default()
            })
            .with_children(|row| {
                row.spawn(button(
                    lang.key("live.see_all"),
                    Action::Open(Panel::Live),
                    90.,
                ));
            });
        });
}

fn news_arrow(text: &str, direction: i8) -> impl Bundle {
    (
        Node {
            width: px(32),
            height: px(30),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        UiAction::Disabled,
        Action::News(direction),
        children![kit::heading(text.to_owned(), 26.)],
    )
}
