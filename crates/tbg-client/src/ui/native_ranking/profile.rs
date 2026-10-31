//! Account rank, record and recent public matches in the Linux ranking sidebar.

use super::*;

pub(super) fn spawn(
    parent: &mut ChildSpawnerCommands,
    ranking: &Ranking,
    account: &Account,
    data: &LobbyData,
    assets: &AssetServer,
    lang: Lang,
) {
    parent
        .spawn(Node {
            width: px(252),
            min_height: px(0),
            flex_shrink: 0.,
            flex_direction: FlexDirection::Column,
            row_gap: px(12),
            overflow: Overflow::scroll_y(),
            ..default()
        })
        .with_children(|sidebar| {
            summary(sidebar, ranking, account, data, assets, lang);
            recent(sidebar, account, data, lang);
        });
}

fn summary(
    sidebar: &mut ChildSpawnerCommands,
    ranking: &Ranking,
    account: &Account,
    data: &LobbyData,
    assets: &AssetServer,
    lang: Lang,
) {
    sidebar
        .spawn((
            Node {
                width: percent(100),
                padding: UiRect::all(px(12)),
                flex_direction: FlexDirection::Column,
                row_gap: px(8),
                border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(4)),
                flex_shrink: 0.,
                ..default()
            },
            frame(),
        ))
        .with_children(|card| {
            card.spawn(kit::heading(
                words(lang, "Your Rank", "Sua classificação"),
                20.,
            ));
            card.spawn(Node {
                width: percent(100),
                justify_content: JustifyContent::Center,
                ..default()
            })
            .with_children(|crest| {
                crest.spawn(kit::image(assets, "ranking/player-crest.webp", 108., 108.));
            });
            centered(
                card,
                account
                    .profile
                    .as_ref()
                    .map_or_else(|| lang.key("lobby.guest.name"), |p| p.name.clone()),
                12.,
            );
            let rank = data
                .stats
                .as_ref()
                .and_then(|s| s["rank"].as_u64())
                .or_else(|| ranking.page.as_ref().and_then(|p| p.you).map(u64::from));
            centered(
                card,
                rank.map_or_else(
                    || words(lang, "Unranked", "Sem classificação"),
                    |r| format!("#{r} Global"),
                ),
                24.,
            );
            centered(
                card,
                account
                    .profile
                    .as_ref()
                    .map_or("—".into(), |p| p.rating.to_string()),
                20.,
            );
            card.spawn(kit::muted(
                words(
                    lang,
                    "Play Ranked to build your record.",
                    "Jogue Ranqueada para construir seu histórico.",
                ),
                11.,
            ));
            record(card, data, lang);
            card.spawn(kit::button(
                words(
                    lang,
                    if account.profile.is_some() {
                        "Play Ranked"
                    } else {
                        "Create account"
                    },
                    if account.profile.is_some() {
                        "Jogar Ranqueada"
                    } else {
                        "Criar conta"
                    },
                ),
                if account.profile.is_some() {
                    UiAction::Ranked
                } else {
                    UiAction::CreateAccount
                },
                180.,
            ));
        });
}

fn record(card: &mut ChildSpawnerCommands, data: &LobbyData, lang: Lang) {
    card.spawn(Node {
        width: percent(100),
        ..default()
    })
    .with_children(|record| {
        for (key, label) in [
            ("wins", words(lang, "Wins", "Vitórias")),
            ("losses", words(lang, "Losses", "Derrotas")),
            ("win_rate", words(lang, "Win rate", "Taxa de vitória")),
        ] {
            let value = data
                .stats
                .as_ref()
                .and_then(|s| s.get(key))
                .map_or("—".into(), |v| {
                    if key == "win_rate" {
                        v.as_f64().map_or("—".into(), |x| format!("{x:.1}%"))
                    } else {
                        v.as_u64().map_or("—".into(), |x| x.to_string())
                    }
                });
            record
                .spawn(Node {
                    width: percent(100. / 3.),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: px(6),
                    ..default()
                })
                .with_children(|stat| {
                    stat.spawn(kit::muted(label, 9.));
                    stat.spawn(text(value, 14.));
                });
        }
    });
}

fn recent(sidebar: &mut ChildSpawnerCommands, account: &Account, data: &LobbyData, lang: Lang) {
    sidebar.spawn(kit::heading(
        words(lang, "Recent Matches", "Partidas recentes"),
        18.,
    ));
    let recent = data.stats.as_ref().and_then(|s| s["recent"].as_array());
    let mut shown = 0;
    for value in recent.into_iter().flatten().take(5) {
        let Some(name) = value["opponent"].as_str() else {
            continue;
        };
        let Some(result) = value["result"]
            .as_str()
            .filter(|r| ["W", "L", "D"].contains(r))
        else {
            continue;
        };
        let delta = value["delta"].as_i64().unwrap_or(0);
        sidebar.spawn(kit::muted(
            format!(
                "vs {} · {result} · {delta:+}",
                name.chars().take(16).collect::<String>()
            ),
            11.,
        ));
        shown += 1;
    }
    if shown == 0 {
        sidebar.spawn(kit::muted(
            words(
                lang,
                if account.profile.is_some() {
                    "No ranked matches yet."
                } else {
                    "Sign in to see your recent ranked matches."
                },
                if account.profile.is_some() {
                    "Ainda não há partidas ranqueadas."
                } else {
                    "Entre para ver suas partidas ranqueadas recentes."
                },
            ),
            12.,
        ));
    }
}

fn centered(parent: &mut ChildSpawnerCommands, value: String, size: f32) {
    parent
        .spawn(Node {
            width: percent(100),
            justify_content: JustifyContent::Center,
            overflow: Overflow::clip(),
            ..default()
        })
        .with_children(|row| {
            row.spawn((kit::heading(value, size), TextLayout::no_wrap()));
        });
}
