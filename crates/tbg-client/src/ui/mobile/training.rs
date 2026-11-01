//! Landscape Troops and Practice presentation reuses shared catalog data and actions.

use crate::{
    i18n::{content, Lang},
    training::{self, Action, Catalog, Tab},
    ui::kit,
};
use bevy::prelude::*;

pub fn spawn(
    parent: &mut ChildSpawnerCommands,
    c: &Catalog,
    assets: &AssetServer,
    lang: Lang,
    scroll: [Vec2; 2],
) {
    parent
        .spawn(Node {
            height: px(48),
            flex_shrink: 0.,
            column_gap: px(8),
            ..default()
        })
        .with_children(|tabs| {
            for (key, tab) in [
                ("catalog", Tab::Catalog),
                ("editor", Tab::Editor),
                ("practice", Tab::Practice),
            ] {
                tabs.spawn(training::catalog::button(
                    lang.key(&format!("training.{key}")),
                    Action::Tab(tab),
                    150.,
                ));
            }
        });
    if c.tab == Tab::Practice {
        practice(parent, c, lang);
        return;
    }
    parent
        .spawn(Node {
            width: percent(100),
            flex_grow: 1.,
            min_height: px(0),
            column_gap: px(8),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                pane(25.),
                ScrollPosition(scroll[0]),
                training::catalog::Pane(0),
            ))
            .with_children(|list| training::catalog::list(list, c, assets, lang));
            row.spawn(pane(34.))
                .with_children(|model| training::catalog::controls(model, c, lang));
            row.spawn((
                pane(39.),
                ScrollPosition(scroll[1]),
                training::catalog::Pane(1),
            ))
            .with_children(|details| {
                if c.tab == Tab::Editor {
                    training::catalog::editor(details, c, lang);
                } else {
                    training::catalog::details(details, c, lang);
                }
            });
        });
}
fn pane(width: f32) -> Node {
    Node {
        width: percent(width),
        min_height: px(0),
        flex_direction: FlexDirection::Column,
        overflow: Overflow::scroll_y(),
        row_gap: px(8),
        padding: UiRect::all(px(8)),
        ..default()
    }
}
fn practice(parent: &mut ChildSpawnerCommands, c: &Catalog, lang: Lang) {
    parent.spawn(kit::heading(lang.key("training.practice"), 20.));
    parent
        .spawn(Node {
            display: Display::Grid,
            width: percent(100),
            grid_template_columns: RepeatedGridTrack::flex(4, 1.),
            column_gap: px(8),
            ..default()
        })
        .with_children(|choices| {
            for (index, key) in ["easy", "medium", "hard", "impossible"]
                .into_iter()
                .enumerate()
            {
                choices.spawn(training::catalog::button(
                    format!(
                        "{}{}",
                        if c.difficulty == index as u8 {
                            "> "
                        } else {
                            ""
                        },
                        lang.key(&format!("training.difficulty.{key}"))
                    ),
                    Action::Difficulty(index as u8),
                    130.,
                ));
            }
        });
    parent.spawn(kit::muted(lang.key("training.difficulty_note"), 13.));
    parent.spawn(Node {
        flex_grow: 1.,
        min_height: px(0),
        ..default()
    });
    parent
        .spawn(Node {
            width: percent(100),
            height: px(56),
            flex_shrink: 0.,
            column_gap: px(12),
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|actions| {
            actions.spawn(training::catalog::button(
                content::arena_name(lang, c.arena),
                Action::Arena,
                160.,
            ));
            actions.spawn(training::catalog::button(
                lang.key("training.play_practice"),
                Action::Practice,
                160.,
            ));
        });
}
