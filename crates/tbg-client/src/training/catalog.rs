//! Responsive Troops browser, real model controls and exact-team editor in the shared shell.

use super::{Action, Catalog, Tab};
use crate::{
    art::anim::Clip,
    i18n::{content, Lang},
    ui::{kit, shell},
};
use bevy::prelude::*;
use tbg_core::{
    content::{def, effective_stats, form_of, ENERGY_SCALE},
    training::TrainingUnitSpec,
};

pub fn button(text: impl Into<String>, action: Action, width: f32) -> impl Bundle {
    shell::button(text, shell::Action::Catalog(action), width)
}
#[derive(Component)]
pub(crate) struct Pane(pub usize);

pub(super) mod arena_picture;
mod inspector;
mod skin;
pub(super) mod viewer;

pub fn spawn(
    parent: &mut ChildSpawnerCommands,
    c: &Catalog,
    assets: &AssetServer,
    lang: Lang,
    scroll: [Vec2; 2],
) {
    parent
        .spawn(Node {
            #[cfg(target_os = "android")]
            width: percent(100),
            column_gap: px(10),
            align_items: AlignItems::Center,
            flex_shrink: 0.,
            ..default()
        })
        .with_children(|row| {
            for (key, tab) in [
                ("catalog", Tab::Catalog),
                ("editor", Tab::Editor),
                ("practice", Tab::Practice),
            ] {
                #[cfg_attr(not(target_os = "android"), allow(unused_mut))]
                let mut _control = row.spawn(skin::control(
                    lang.key(&format!("training.{key}")),
                    Action::Tab(tab),
                    220.,
                    c.tab == tab,
                ));
                #[cfg(target_os = "android")]
                _control.insert(Node {
                    flex_grow: 1.,
                    flex_basis: px(0),
                    min_width: px(48),
                    min_height: px(48),
                    padding: UiRect::all(px(4)),
                    border: UiRect::all(px(1)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                });
            }
            #[cfg(not(target_os = "android"))]
            row.spawn(kit::muted(
                format!(
                    "{} · {}",
                    tbg_core::COMPLETION_POOL.len(),
                    lang.key("training.champions")
                ),
                13.,
            ));
        });
    parent
        .spawn(Node {
            width: percent(100),
            flex_grow: 1.,
            min_height: px(0),
            column_gap: px(if cfg!(target_os = "android") { 6 } else { 14 }),
            ..default()
        })
        .with_children(|row| {
            row.spawn(skin::panel(if cfg!(target_os = "android") {
                30.
            } else {
                28.
            }))
            .with_children(|p| list(p, c, assets, lang, scroll[0]));
            row.spawn(Node {
                width: percent(if cfg!(target_os = "android") { 36 } else { 40 }),
                min_width: px(0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexEnd,
                ..default()
            })
            .with_children(|p| viewer::controls(p, c, lang));
            row.spawn((
                skin::panel(if cfg!(target_os = "android") {
                    32.
                } else {
                    30.
                }),
                ScrollPosition(scroll[1]),
                Pane(1),
            ))
            .with_children(|p| match c.tab {
                Tab::Catalog => inspector::details(p, c, lang),
                Tab::Editor => inspector::editor(p, c, lang),
                Tab::Practice => practice(p, c, assets, lang),
            });
        });
}
fn list(p: &mut ChildSpawnerCommands, c: &Catalog, assets: &AssetServer, lang: Lang, scroll: Vec2) {
    #[cfg(not(target_os = "android"))]
    p.spawn(kit::heading(
        lang.key("training.catalog"),
        if cfg!(target_os = "android") {
            18.
        } else {
            25.
        },
    ));
    p.spawn(skin::control(
        if c.search.is_empty() {
            lang.key("training.search")
        } else {
            c.search.clone()
        },
        Action::Search,
        0.,
        !c.search.is_empty(),
    ));
    p.spawn(Node {
        width: percent(100),
        flex_direction: if cfg!(target_os = "android") {
            FlexDirection::Column
        } else {
            FlexDirection::Row
        },
        column_gap: px(6),
        row_gap: px(6),
        flex_shrink: 0.,
        ..default()
    })
    .with_children(|row| {
        for (label, action, active) in [
            (
                c.faction.map_or_else(
                    || lang.key("training.all_factions"),
                    |f| content::faction(lang, f).into(),
                ),
                Action::Faction,
                c.faction.is_some(),
            ),
            (
                c.role.map_or_else(
                    || lang.key("training.all_roles"),
                    |r| content::archetype(lang, r).into(),
                ),
                Action::Role,
                c.role.is_some(),
            ),
        ] {
            row.spawn(skin::control(label, action, 0., active))
                .insert(Node {
                    width: if cfg!(target_os = "android") {
                        percent(100)
                    } else {
                        percent(50)
                    },
                    flex_shrink: 0.,
                    min_width: px(0),
                    min_height: px(if cfg!(target_os = "android") { 48 } else { 40 }),
                    padding: UiRect::all(px(if cfg!(target_os = "android") { 4 } else { 6 })),
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(4)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                });
        }
    });
    let visible = c.visible(lang);
    #[cfg(not(target_os = "android"))]
    p.spawn(kit::muted(lang.key("training.list_hint"), 11.));
    p.spawn((
        Node {
            width: percent(100),
            flex_grow: 1.,
            min_height: px(if cfg!(target_os = "android") { 96 } else { 0 }),
            flex_direction: FlexDirection::Column,
            overflow: Overflow::scroll_y(),
            row_gap: px(7),
            ..default()
        },
        ScrollPosition(scroll),
        Pane(0),
    ))
    .with_children(|list| {
        #[cfg(target_os = "android")]
        list.spawn((
            Node {
                width: percent(100),
                min_width: px(0),
                flex_shrink: 0.,
                ..default()
            },
            kit::muted(lang.key("training.list_hint"), 11.),
        ));
        if visible.is_empty() {
            list.spawn(kit::muted(lang.key("training.no_results"), 15.));
        }
        for kind in visible {
            let selected = kind == c.selected;
            list.spawn((
                Node {
                    width: percent(100),
                    min_height: px(67),
                    flex_shrink: 0.,
                    padding: UiRect::all(px(5)),
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(4)),
                    align_items: AlignItems::Center,
                    column_gap: px(if cfg!(target_os = "android") { 6 } else { 10 }),
                    ..default()
                },
                skin::colors(selected),
                crate::ui::UiAction::Disabled,
                shell::Action::Catalog(Action::Select(kind)),
            ))
            .with_children(|row| {
                row.spawn(kit::image(
                    assets,
                    format!(
                        "battle/portraits/{}.webp",
                        format!("{kind:?}").to_lowercase()
                    ),
                    if cfg!(target_os = "android") {
                        42.
                    } else {
                        55.
                    },
                    if cfg!(target_os = "android") {
                        42.
                    } else {
                        55.
                    },
                ));
                row.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.,
                    #[cfg(target_os = "android")]
                    min_width: px(0),
                    #[cfg(target_os = "android")]
                    flex_basis: px(0),
                    row_gap: px(4),
                    ..default()
                })
                .with_children(|copy| {
                    copy.spawn((
                        #[cfg(target_os = "android")]
                        Node {
                            width: percent(100),
                            min_width: px(0),
                            ..default()
                        },
                        kit::heading(
                            content::unit_name(lang, kind),
                            if cfg!(target_os = "android") {
                                14.
                            } else {
                                18.
                            },
                        ),
                        #[cfg(target_os = "android")]
                        TextLayout::new(Justify::Left, LineBreak::WordOrCharacter),
                    ));
                    copy.spawn(kit::muted(
                        format!(
                            "{} · {}",
                            content::faction(lang, def(kind).faction),
                            content::archetype(lang, def(kind).archetype)
                        ),
                        12.,
                    ));
                });
                row.spawn(kit::heading(
                    ">",
                    if cfg!(target_os = "android") {
                        16.
                    } else {
                        22.
                    },
                ));
            });
        }
    });
}
fn practice(p: &mut ChildSpawnerCommands, c: &Catalog, assets: &AssetServer, lang: Lang) {
    p.spawn(Node {
        width: percent(100),
        height: percent(100),
        min_height: px(0),
        flex_direction: FlexDirection::Column,
        row_gap: px(8),
        ..default()
    })
    .with_children(|p| practice_contents(p, c, assets, lang));
}
fn practice_contents(p: &mut ChildSpawnerCommands, c: &Catalog, assets: &AssetServer, lang: Lang) {
    p.spawn(kit::heading(lang.key("training.practice"), 24.));
    p.spawn(kit::muted(lang.key("training.practice_hint"), 13.));
    for (i, key) in ["easy", "medium", "hard", "impossible"]
        .into_iter()
        .enumerate()
    {
        p.spawn(skin::control(
            lang.key(&format!("training.difficulty.{key}")),
            Action::Difficulty(i as u8),
            0.,
            c.difficulty == i as u8,
        ));
    }
    p.spawn((
        Node {
            padding: UiRect::all(px(10)),
            border: UiRect::all(px(1)),
            ..default()
        },
        BackgroundColor(kit::INK),
        BorderColor::all(kit::LINE),
    ))
    .with_children(|note| {
        note.spawn(kit::muted(lang.key("training.difficulty_note"), 12.));
    });
    p.spawn(kit::heading(lang.key("training.arena"), 19.));
    arena_picture::spawn(p, assets, c.arena);
    p.spawn(skin::control(
        format!("{}   >", content::arena_name(lang, c.arena)),
        Action::Arena,
        0.,
        false,
    ));
    #[cfg(tbg_desktop)]
    let preview_5v5 = super::moba::available();
    #[cfg(not(tbg_desktop))]
    let preview_5v5 = false;
    let practice_label = if preview_5v5 {
        "training.play_1v1"
    } else {
        "training.play_practice"
    };
    p.spawn(skin::control(
        lang.key(practice_label),
        Action::Practice,
        0.,
        true,
    ))
    .insert(skin::primary());
    #[cfg(tbg_desktop)]
    if preview_5v5 {
        p.spawn(skin::control(
            lang.key("training.play_5v5"),
            Action::Practice5v5,
            0.,
            false,
        ));
    }
}
