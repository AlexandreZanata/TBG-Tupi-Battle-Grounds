//! Identity-bound laboratory controls and read-only, paged instance/team measurements.

use super::{catalog::button, Action, Trial};
use crate::{
    i18n::{content, Lang},
    ui::kit,
    GameState,
};
use bevy::prelude::*;
use tbg_core::{
    training::{Lifecycle, Pause, SessionId},
    Seat,
};

const ROWS: usize = 12;
#[derive(Component)]
struct Root;
#[derive(Component)]
enum Field {
    Selected,
    Status,
    Clock,
    Team(Seat),
    Unit(u32),
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        (build, update)
            .chain()
            .run_if(in_state(GameState::Training)),
    );
}
fn panel(left: f32, top: f32, width: f32, height: f32) -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            left: percent(left),
            top: percent(top),
            width: percent(width),
            height: percent(height),
            padding: UiRect::all(px(12)),
            border: UiRect::all(px(1)),
            flex_direction: FlexDirection::Column,
            row_gap: px(8),
            overflow: Overflow::scroll_y(),
            ..default()
        },
        BackgroundColor(kit::PANEL),
        BorderColor::all(kit::GOLD),
    )
}
fn build(
    mut commands: Commands,
    mut trial: ResMut<Trial>,
    lang: Res<Lang>,
    roots: Query<Entity, With<Root>>,
    mut last: Local<Option<(SessionId, usize, usize)>>,
) {
    let Some(session) = trial.session.as_ref() else {
        return;
    };
    let rows = session.metrics().rows().len();
    let id = session.id();
    let page = trial.page.min(rows.saturating_sub(1) / ROWS);
    if trial.page != page {
        trial.page = page;
    }
    let key = (id, rows, page);
    if *last == Some(key) && !roots.is_empty() {
        return;
    }
    *last = Some(key);
    for root in &roots {
        commands.entity(root).despawn();
    }
    let lang = *lang;
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                ..default()
            },
            Root,
            DespawnOnExit(GameState::Training),
            ZIndex(20),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: percent(2),
                    top: percent(2),
                    width: percent(96),
                    padding: UiRect::all(px(10)),
                    column_gap: px(10),
                    row_gap: px(6),
                    flex_wrap: FlexWrap::Wrap,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(kit::PANEL),
                BorderColor::all(kit::GOLD),
            ))
            .with_children(|bar| {
                bar.spawn(kit::heading(lang.key("training.editor"), 21.));
                bar.spawn((kit::muted("", 13.), Field::Clock));
                for (key, action) in [
                    ("pause_resume", Action::Control(id, 0)),
                    ("step", Action::Control(id, 1)),
                    ("reset", Action::Control(id, 2)),
                    ("back", Action::Back),
                ] {
                    bar.spawn(button(lang.key(&format!("training.{key}")), action, 155.));
                }
                bar.spawn((kit::muted("", 13.), Field::Status));
            });
            root.spawn(panel(2., 17., 23., 74.)).with_children(|p| {
                p.spawn(kit::heading(lang.key("training.measurements"), 20.));
                p.spawn(kit::muted(lang.key("training.measurement_note"), 12.));
                p.spawn((kit::muted("", 12.), Field::Selected));
                for seat in Seat::BOTH {
                    p.spawn((crate::ui::label("", 14.), Field::Team(seat)));
                }
            });
            root.spawn(panel(76., 17., 22., 74.)).with_children(|p| {
                p.spawn(kit::heading(lang.key("training.instances"), 20.));
                p.spawn(kit::row()).with_children(|r| {
                    r.spawn(button("<", Action::Page(-1), 45.));
                    r.spawn(kit::muted(
                        format!(
                            "{}-{} / {rows}",
                            if rows == 0 { 0 } else { page * ROWS + 1 },
                            ((page + 1) * ROWS).min(rows)
                        ),
                        12.,
                    ));
                    r.spawn(button(">", Action::Page(1), 45.));
                });
                if let Some(session) = &trial.session {
                    for row in session.metrics().rows().iter().skip(page * ROWS).take(ROWS) {
                        p.spawn(button(
                            format!(
                                "#{} {}",
                                row.id,
                                if row.dummy {
                                    lang.key("training.dummy")
                                } else {
                                    content::unit_name(lang, row.kind).into()
                                }
                            ),
                            Action::Target(row.id),
                            230.,
                        ));
                        p.spawn((kit::muted("", 11.), Field::Unit(row.id)));
                    }
                }
            });
        });
}
fn update(
    trial: Res<Trial>,
    view: Res<crate::sync::LatestView>,
    lang: Res<Lang>,
    mut texts: Query<(&Field, &mut Text)>,
) {
    let Some(s) = &trial.session else { return };
    let seconds = f64::from(s.active_ticks()) / f64::from(tbg_core::content::TICK_HZ);
    for (field, mut text) in &mut texts {
        let value = match field {
            Field::Selected => {
                trial
                    .target
                    .and_then(|id| s.metrics().row(id))
                    .map_or_else(String::new, |r| {
                        format!(
                            "#{} · {}\n{} {} · {} {}\n{} {} · {} {}\n{} {:.2}s · {} {}",
                            r.id,
                            content::unit_name(*lang, r.kind),
                            lang.key("training.restored"),
                            r.restored,
                            lang.key("training.absorbed"),
                            r.shield_absorbed + r.plates_absorbed,
                            lang.key("training.rescaled"),
                            r.rescaled,
                            lang.key("training.remaining"),
                            r.expected_hp(),
                            lang.key("training.spawned"),
                            f64::from(r.spawned_at) / 20.,
                            lang.key("training.died"),
                            r.died_at.map_or_else(
                                || "—".into(),
                                |t| format!("{:.2}s", f64::from(t) / 20.)
                            )
                        )
                    })
            }
            Field::Status => match s.lifecycle() {
                Lifecycle::Ready => lang.key("training.ready"),
                Lifecycle::Running => lang.key("training.running"),
                Lifecycle::Paused(Pause::Player) => lang.key("training.paused"),
                Lifecycle::Paused(_) => lang.key("training.limit"),
                Lifecycle::Finished(f) => lang.key(match f {
                    tbg_core::training::Finish::AlliesWon => "training.finish.allies",
                    tbg_core::training::Finish::OpponentsWon => "training.finish.enemies",
                    tbg_core::training::Finish::Draw => "training.finish.draw",
                    tbg_core::training::Finish::TargetsDestroyed => "training.finish.targets",
                    tbg_core::training::Finish::AlliesEliminated => "training.finish.eliminated",
                }),
            },
            Field::Clock => format!(
                "{seconds:.2} s · {} · Seed {}",
                content::arena_name(*lang, s.scenario().arena),
                s.scenario().seed
            ),
            Field::Team(seat) => {
                let t = s.metrics().team(*seat);
                format!(
                    "{}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\nDPS: {:.1}\n{}: {}",
                    lang.key(if *seat == Seat::A {
                        "training.allies"
                    } else {
                        "training.enemies"
                    }),
                    lang.key("training.dealt"),
                    t.dealt,
                    lang.key("training.friendly"),
                    t.dealt_friendly,
                    lang.key("training.taken"),
                    t.taken,
                    lang.key("training.healed"),
                    t.healed,
                    t.dealt as f64 / seconds.max(0.05),
                    lang.key("training.unattributed"),
                    t.taken_unattributed
                )
            }
            Field::Unit(id) => {
                let Some(row) = s.metrics().row(*id) else {
                    continue;
                };
                let hp = s
                    .state()
                    .units
                    .iter()
                    .find(|u| u.id == *id)
                    .map_or(0, |u| u.hp);
                format!(
                    "HP {hp} · {} {} · {} {}\n{} {} · {} {}",
                    lang.key("training.dealt"),
                    row.dealt_total(),
                    lang.key("training.taken"),
                    row.taken_total(),
                    lang.key("training.healed"),
                    row.healed_allies + row.healed_self,
                    lang.key("training.unattributed"),
                    row.taken_unattributed
                )
            }
        };
        let value = if matches!(field, Field::Selected) {
            let effects = trial
                .target
                .and_then(|id| {
                    view.0.as_ref().and_then(|v| {
                        v.units.iter().find(|u| u.id == id).map(|u| {
                            format!(
                                "{}\n{}",
                                crate::i18n::content::status(*lang, u.status),
                                crate::ui::effect_details::describe(v, u)
                            )
                        })
                    })
                })
                .unwrap_or_default();
            format!("{value}\n{effects}")
        } else {
            value
        };
        if text.0 != value {
            text.0 = value;
        }
    }
}
