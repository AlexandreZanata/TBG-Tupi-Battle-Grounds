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
pub fn spawn(parent: &mut ChildSpawnerCommands, c: &Catalog, assets: &AssetServer, lang: Lang) {
    parent.spawn(kit::row()).with_children(|row| {
        for (key, tab) in [
            ("catalog", Tab::Catalog),
            ("editor", Tab::Editor),
            ("practice", Tab::Practice),
        ] {
            row.spawn(button(
                lang.key(&format!("training.{key}")),
                Action::Tab(tab),
                200.,
            ));
        }
        row.spawn(kit::muted(
            format!(
                "{} · {}",
                tbg_core::FULL_POOL.len(),
                lang.key("training.champions")
            ),
            13.,
        ));
    });
    parent
        .spawn(Node {
            width: percent(100),
            height: percent(90),
            min_height: px(500),
            column_gap: px(14),
            ..default()
        })
        .with_children(|row| {
            row.spawn(panel(28.))
                .with_children(|p| list(p, c, assets, lang));
            row.spawn(Node {
                width: percent(35),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexEnd,
                row_gap: px(8),
                ..default()
            })
            .with_children(|p| controls(p, c, lang));
            row.spawn(panel(35.)).with_children(|p| match c.tab {
                Tab::Catalog => details(p, c, lang),
                Tab::Editor => editor(p, c, lang),
                Tab::Practice => practice(p, c, lang),
            });
        });
}
fn panel(width: f32) -> impl Bundle {
    (
        Node {
            width: percent(width),
            padding: UiRect::all(px(14)),
            border: UiRect::all(px(1)),
            flex_direction: FlexDirection::Column,
            row_gap: px(10),
            overflow: Overflow::scroll_y(),
            ..default()
        },
        BackgroundColor(kit::PANEL),
        BorderColor::all(kit::GOLD),
    )
}
fn list(p: &mut ChildSpawnerCommands, c: &Catalog, assets: &AssetServer, lang: Lang) {
    p.spawn(kit::heading(lang.key("training.catalog"), 23.));
    p.spawn(button(
        if c.search.is_empty() {
            lang.key("training.search")
        } else {
            c.search.clone()
        },
        Action::Search,
        250.,
    ));
    p.spawn(button(
        c.faction.map_or_else(
            || lang.key("training.all_factions"),
            |f| content::faction(lang, f).into(),
        ),
        Action::Faction,
        250.,
    ));
    p.spawn(button(
        c.role.map_or_else(
            || lang.key("training.all_roles"),
            |r| content::archetype(lang, r).into(),
        ),
        Action::Role,
        250.,
    ));
    for kind in c.visible() {
        p.spawn((
            Node {
                width: percent(100),
                min_height: px(67),
                padding: UiRect::all(px(5)),
                border: UiRect::all(px(1)),
                align_items: AlignItems::Center,
                column_gap: px(10),
                ..default()
            },
            BackgroundColor(if kind == c.selected {
                kit::BLUE
            } else {
                kit::INK
            }),
            BorderColor::all(kit::LINE),
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
                55.,
                55.,
            ));
            row.spawn(Node {
                flex_direction: FlexDirection::Column,
                ..default()
            })
            .with_children(|copy| {
                copy.spawn(kit::heading(content::unit_name(lang, kind), 16.));
                copy.spawn(kit::muted(
                    format!(
                        "{} · {}",
                        content::faction(lang, def(kind).faction),
                        content::archetype(lang, def(kind).archetype)
                    ),
                    11.,
                ));
            });
        });
    }
}
fn controls(p: &mut ChildSpawnerCommands, c: &Catalog, lang: Lang) {
    p.spawn(kit::heading(content::unit_name(lang, c.selected), 23.));
    p.spawn(kit::muted(lang.key("training.preview_note"), 12.));
    p.spawn((
        Node {
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            column_gap: px(6),
            row_gap: px(6),
            ..default()
        },
        BackgroundColor(kit::INK),
    ))
    .with_children(|row| {
        for (name, t, b) in [
            ("Base", 0, 0),
            ("A1", 1, 0),
            ("A2", 2, 0),
            ("B1", 1, 1),
            ("B2", 2, 1),
        ] {
            if t == 0 || !c.selected.is_summon() {
                row.spawn(button(name, Action::Form(t, b), 57.));
            }
        }
        for (key, clip) in [
            ("idle", Clip::Idle),
            ("walk", Clip::Walk),
            (
                "attack",
                if tbg_core::content::def(c.selected).range > 1500 {
                    Clip::AttackRanged
                } else {
                    Clip::AttackMelee
                },
            ),
            ("cast", Clip::Cast),
            ("death", Clip::Death),
            ("evolve", Clip::Evolve),
            ("dance", Clip::Victory),
        ] {
            row.spawn(button(
                lang.key(&format!("training.clip.{key}")),
                Action::Clip(clip),
                105.,
            ));
        }
        for (text, a) in [
            ("<", Action::Rotate(-1)),
            (">", Action::Rotate(1)),
            ("-", Action::Zoom(-1)),
            ("+", Action::Zoom(1)),
        ] {
            row.spawn(button(text, a, 56.));
        }
        row.spawn(button(lang.key("training.motion"), Action::Motion, 200.));
    });
    p.spawn(kit::muted(lang.key("training.drag_help"), 11.));
}
fn details(p: &mut ChildSpawnerCommands, c: &Catalog, lang: Lang) {
    let d = def(c.selected);
    let s = effective_stats(c.selected, c.form.0, c.form.1, 0);
    p.spawn(kit::heading(
        content::form_name(lang, c.selected, c.form.0, c.form.1)
            .unwrap_or_else(|| content::unit_name(lang, c.selected)),
        22.,
    ));
    p.spawn(kit::muted(
        format!(
            "{} · {}",
            content::faction(lang, d.faction),
            content::archetype(lang, d.archetype)
        ),
        13.,
    ));
    let source = match c.selected {
        tbg_core::UnitKind::Beetling => Some("Nester"),
        tbg_core::UnitKind::Microdrill
        | tbg_core::UnitKind::Duodrill
        | tbg_core::UnitKind::Megadrill => Some("Drillnest"),
        _ => None,
    };
    if let Some(source) = source {
        p.spawn(kit::muted(
            format!("{}: {source}", lang.key("training.summoned_by")),
            13.,
        ));
    }
    let name = crate::i18n::content::unit_slug(c.selected);
    p.spawn(crate::ui::label(
        lang.key(&format!("unit.{name}.lore")),
        14.,
    ));
    for (key, value) in [
        ("hp", s.max_hp.to_string()),
        ("damage", s.damage.to_string()),
        ("heal", s.heal.to_string()),
        ("splash", format!("{:.2}", s.splash as f32 / 1000.)),
        ("interval", format!("{:.2} s", s.interval as f32 / 20.)),
        ("range", format!("{:.2}", s.range as f32 / 1000.)),
        ("speed", format!("{:.2}", s.speed as f32 * 20. / 1000.)),
        ("cost", (d.cost / ENERGY_SCALE).to_string()),
    ] {
        p.spawn(crate::ui::label(
            format!("{}: {value}", lang.key(&format!("training.stat.{key}"))),
            14.,
        ));
    }
    if let Some(f) = form_of(c.selected, c.form.0, c.form.1) {
        p.spawn(kit::muted(crate::ui::evolve::describe(f, lang), 12.));
    }
    p.spawn(kit::heading(lang.key("training.ability"), 18.));
    let ability = crate::ui::ability_details::describe(c.selected, c.form.0, c.form.1, lang);
    p.spawn(crate::ui::label(
        if ability.is_empty() {
            lang.key("training.no_special")
        } else {
            ability
        },
        13.,
    ));
    let (good, weak) = content::matchup(lang, c.selected);
    p.spawn(kit::muted(
        format!(
            "{}: {good}\n{}: {weak}",
            lang.key("training.good"),
            lang.key("training.weak")
        ),
        12.,
    ));
    p.spawn(button(
        lang.key("training.add_ally"),
        Action::Add(true),
        250.,
    ));
    p.spawn(button(
        lang.key("training.editor"),
        Action::Tab(Tab::Editor),
        250.,
    ));
}
fn editor(p: &mut ChildSpawnerCommands, c: &Catalog, lang: Lang) {
    p.spawn(kit::heading(lang.key("training.editor"), 22.));
    p.spawn(kit::muted(lang.key("training.local_note"), 12.));
    team(p, &c.allies, true, lang);
    p.spawn(button(
        lang.key("training.add_ally"),
        Action::Add(true),
        250.,
    ));
    p.spawn(button(
        lang.key(if c.dummies {
            "training.opponent_dummies"
        } else {
            "training.opponent_champions"
        }),
        Action::Opponent,
        250.,
    ));
    if c.dummies {
        dummy(p, c, lang);
    } else {
        team(p, &c.enemies, false, lang);
        p.spawn(button(
            lang.key("training.add_enemy"),
            Action::Add(false),
            250.,
        ));
    }
    p.spawn(button(
        content::arena_name(lang, c.arena),
        Action::Arena,
        250.,
    ));
    p.spawn(button(format!("Seed: {}", c.seed), Action::Seed, 250.));
    p.spawn(button(lang.key("training.start"), Action::Start, 250.));
    if !c.notice.is_empty() {
        p.spawn(kit::muted(c.notice.clone(), 12.));
    }
}
fn team(p: &mut ChildSpawnerCommands, units: &[TrainingUnitSpec], ally: bool, lang: Lang) {
    p.spawn(kit::heading(
        format!(
            "{} {}/12",
            lang.key(if ally {
                "training.allies"
            } else {
                "training.enemies"
            }),
            units.len()
        ),
        17.,
    ));
    for (i, u) in units.iter().enumerate() {
        p.spawn(kit::muted(
            format!(
                "{} · {}{} · {}",
                content::unit_name(lang, u.kind),
                if u.tier == 0 {
                    "T"
                } else if u.branch == 0 {
                    "A"
                } else {
                    "B"
                },
                u.tier,
                u.x.map_or_else(
                    || lang.key("training.auto_position"),
                    |x| format!("{:.1}", x as f32 / 1000.)
                )
            ),
            12.,
        ));
        p.spawn(kit::row()).with_children(|row| {
            for (text, a) in [
                ("<", Action::Move(ally, i, -1)),
                (">", Action::Move(ally, i, 1)),
                ("x", Action::Remove(ally, i)),
            ] {
                row.spawn(button(text, a, 45.));
            }
        });
    }
}
fn dummy(p: &mut ChildSpawnerCommands, c: &Catalog, lang: Lang) {
    p.spawn(kit::row()).with_children(|row| {
        row.spawn(button("-", Action::DummyCount(-1), 45.));
        row.spawn(kit::heading(c.dummy_count.to_string(), 17.));
        row.spawn(button("+", Action::DummyCount(1), 45.));
    });
    p.spawn(button(
        format!("{} HP", c.dummy_hp.hp()),
        Action::DummyHp,
        250.,
    ));
    p.spawn(button(
        lang.key(match c.dummy_mode {
            super::DummyMode::Finite => "training.finite",
            super::DummyMode::Renewable => "training.renewable",
        }),
        Action::DummyMode,
        250.,
    ));
}
fn practice(p: &mut ChildSpawnerCommands, c: &Catalog, lang: Lang) {
    p.spawn(kit::heading(lang.key("training.practice"), 22.));
    for (i, key) in ["easy", "medium", "hard", "impossible"]
        .into_iter()
        .enumerate()
    {
        p.spawn(button(
            format!(
                "{}{}",
                if c.difficulty == i as u8 { "> " } else { "" },
                lang.key(&format!("training.difficulty.{key}"))
            ),
            Action::Difficulty(i as u8),
            250.,
        ));
    }
    p.spawn(kit::muted(lang.key("training.difficulty_note"), 13.));
    p.spawn(button(
        content::arena_name(lang, c.arena),
        Action::Arena,
        250.,
    ));
    p.spawn(button(
        lang.key("training.play_practice"),
        Action::Practice,
        250.,
    ));
}
