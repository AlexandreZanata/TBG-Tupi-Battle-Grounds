//! Public ranking rows, aligned result columns and server-owned page navigation.

use super::*;

#[derive(Component)]
pub(super) struct List;

pub(super) fn spawn(
    parent: &mut ChildSpawnerCommands,
    ranking: &Ranking,
    account: &Account,
    lang: Lang,
    scroll: ScrollPosition,
) {
    let page = ranking.page.as_ref();
    let status = if ranking.failed {
        lang.key("live.unavailable")
    } else if ranking.want.is_some() {
        words(lang, "Loading rankings…", "Carregando ranking…")
    } else {
        format!(
            "{} {}",
            page.map_or(0, |p| p.total),
            words(lang, "ranked players", "jogadores classificados")
        )
    };
    parent.spawn((
        Node {
            width: percent(100),
            min_height: px(24),
            flex_shrink: 0.,
            ..default()
        },
        children![kit::muted(status, 12.)],
    ));
    parent.spawn((Node{width:percent(100),flex_grow:1.,min_height:px(0),flex_direction:FlexDirection::Column,overflow:Overflow::scroll_y(),row_gap:px(3),..default()}, scroll, List)).with_children(|list| {
        let headings=["#".into(),words(lang,"Player","Jogador"),words(lang,"Rating","Pontuação"),words(lang,"Wins","Vitórias"),words(lang,"Losses","Derrotas"),words(lang,"Draws","Empates"),words(lang,"Win rate","Taxa de vitória")];
        row(list,headings,Color::srgb_u8(17,43,49),true);
        if page.is_none() && ranking.want.is_some() {
            super::super::native_loading::rows(list, 6);
        }
        if let Some(page)=page {
            for player in &page.rows {
                let mine=account.profile.as_ref().is_some_and(|p|p.name==player.name);
                let color=if mine{super::super::BUTTON}else{match player.rank{1=>Color::srgb_u8(66,58,33),2=>Color::srgb_u8(21,55,67),3=>Color::srgb_u8(28,49,43),_=>Color::srgb_u8(18,38,44)}};
                row(list,[player.rank.to_string(),player.name.clone(),player.rating.to_string(),player.wins.to_string(),player.losses.to_string(),player.draws.to_string(),rate(player)],color,false);
            }
            if page.rows.is_empty(){list.spawn(kit::muted(words(lang,"No ranked players yet. Play Ranked to earn a place.","Ainda não há jogadores classificados. Jogue Ranqueada para conquistar seu lugar."),13.));}
        }
    });
    let at = page.map_or(0, |p| p.offset);
    let idle = ranking.want.is_none();
    let prev = if at > 0 && idle {
        UiAction::RankingPrev
    } else {
        UiAction::Disabled
    };
    let next = if idle
        && page.is_some_and(|p| u32::from(at) + u32::from(super::super::ranking::PAGE) < p.total)
    {
        UiAction::RankingNext
    } else {
        UiAction::Disabled
    };
    parent
        .spawn(Node {
            width: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            column_gap: px(8),
            flex_shrink: 0.,
            ..default()
        })
        .with_children(|footer| {
            footer.spawn(kit::muted(
                format!(
                    "{} {}",
                    words(lang, "Page", "Página"),
                    at / super::super::ranking::PAGE + 1
                ),
                11.,
            ));
            footer
                .spawn(Node {
                    column_gap: px(8),
                    ..default()
                })
                .with_children(|buttons| {
                    for (label, action) in [
                        (words(lang, "Previous", "Anterior"), prev),
                        (words(lang, "Next", "Próxima"), next),
                        (words(lang, "Refresh", "Atualizar"), UiAction::OpenRanking),
                        (
                            words(lang, "Open in browser", "Ver no navegador"),
                            UiAction::OpenFullRanking,
                        ),
                    ] {
                        buttons.spawn(kit::button(label, action, 100.));
                    }
                });
        });
}

fn row(parent: &mut ChildSpawnerCommands, values: [String; 7], color: Color, heading: bool) {
    parent
        .spawn((
            Node {
                width: percent(100),
                min_height: px(28),
                flex_shrink: 0.,
                border_radius: BorderRadius::all(px(3)),
                ..default()
            },
            BackgroundColor(color),
        ))
        .with_children(|line| {
            for (i, (value, width)) in values
                .into_iter()
                .zip([5., 29., 14., 13., 13., 12., 14.])
                .enumerate()
            {
                line.spawn(Node {
                    width: percent(width),
                    min_width: px(0),
                    padding: UiRect::horizontal(px(7)),
                    align_items: AlignItems::Center,
                    justify_content: if i == 1 {
                        JustifyContent::FlexStart
                    } else {
                        JustifyContent::Center
                    },
                    overflow: Overflow::clip(),
                    ..default()
                })
                .with_children(|cell| {
                    let ink = if heading {
                        kit::MUTED
                    } else {
                        match i {
                            3 => Color::srgb_u8(148, 223, 147),
                            4 => Color::srgb_u8(239, 141, 121),
                            _ => Color::srgb_u8(241, 223, 174),
                        }
                    };
                    cell.spawn(text(value, if heading { 10. } else { 12. }))
                        .insert(TextColor(ink));
                });
            }
        });
}
