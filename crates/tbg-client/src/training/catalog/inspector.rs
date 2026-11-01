//! Champion facts and exact-team editor, separate from the presentation layout.

use super::*;

pub(super) fn details(p: &mut ChildSpawnerCommands, c: &Catalog, lang: Lang) {
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
    p.spawn(Node {
        width: percent(100),
        flex_wrap: FlexWrap::Wrap,
        row_gap: px(7),
        ..default()
    })
    .with_children(|grid| {
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
            grid.spawn((
                Node {
                    width: percent(50),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(3),
                    ..default()
                },
                BackgroundColor(kit::INK),
            ))
            .with_children(|cell| {
                cell.spawn(kit::muted(lang.key(&format!("training.stat.{key}")), 11.));
                cell.spawn(crate::ui::label(value, 17.));
            });
        }
    });
    let on_hit = match s.on_hit {
        Some(tbg_core::OnHit::Slow { bp, ticks }) => lang.tf(
            crate::i18n::Msg::InspectSlow,
            &[
                ("pct", &(bp / 100)),
                ("secs", &lang.decimal(f64::from(ticks) / 20., 1)),
            ],
        ),
        Some(tbg_core::OnHit::Root { ticks }) => lang.tf(
            crate::i18n::Msg::InspectRoot,
            &[("secs", &lang.decimal(f64::from(ticks) / 20., 1))],
        ),
        None => "—".into(),
    };
    p.spawn(kit::muted(
        format!("{}: {on_hit}", lang.key("training.on_hit")),
        13.,
    ));
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
    p.spawn(skin::control(
        lang.key("training.add_ally"),
        Action::Add(true),
        0.,
        false,
    ));
    p.spawn(skin::control(
        lang.key("training.editor"),
        Action::Tab(Tab::Editor),
        0.,
        false,
    ));
}
pub(super) fn editor(p: &mut ChildSpawnerCommands, c: &Catalog, lang: Lang) {
    p.spawn(kit::heading(lang.key("training.editor"), 22.));
    p.spawn(kit::muted(lang.key("training.local_note"), 12.));
    team(p, &c.allies, true, lang);
    p.spawn(skin::control(
        lang.key("training.add_ally"),
        Action::Add(true),
        0.,
        false,
    ));
    p.spawn(skin::control(
        lang.key(if c.dummies {
            "training.opponent_dummies"
        } else {
            "training.opponent_champions"
        }),
        Action::Opponent,
        0.,
        false,
    ));
    if c.dummies {
        dummy(p, c, lang);
    } else {
        team(p, &c.enemies, false, lang);
        p.spawn(skin::control(
            lang.key("training.add_enemy"),
            Action::Add(false),
            0.,
            false,
        ));
    }
    p.spawn(skin::control(
        content::arena_name(lang, c.arena),
        Action::Arena,
        0.,
        false,
    ));
    p.spawn(skin::control(
        format!("Seed: {}", c.seed),
        Action::Seed,
        0.,
        false,
    ));
    p.spawn(skin::control(
        lang.key("training.start"),
        Action::Start,
        0.,
        true,
    ));
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
    p.spawn(skin::control(
        format!("{} HP", c.dummy_hp.hp()),
        Action::DummyHp,
        0.,
        false,
    ));
    p.spawn(skin::control(
        lang.key(match c.dummy_mode {
            super::super::DummyMode::Finite => "training.finite",
            super::super::DummyMode::Renewable => "training.renewable",
        }),
        Action::DummyMode,
        0.,
        false,
    ));
}
