//! Local editor commands, identity-checked trial controls and canonical view delivery.

use super::{Action, Catalog, Requests, Trial};
use crate::{
    i18n::Lang,
    link::MatchLink,
    sync::{LatestView, MatchInfo, UnitEntities, ViewBuffer},
    ui::{
        shell::{Panel, Shell},
        textform::{self, ActiveForm, TextEntered, TextKind},
    },
    GameState,
};
use bevy::prelude::*;
use tbg_core::{
    training::{
        Lifecycle, Opponent, SessionId, TrainingCommand, TrainingScenario, TrainingSession,
        MAX_DUMMIES, MAX_UNITS_PER_SIDE, SCHEMA,
    },
    Seat,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, apply)
        .add_systems(FixedUpdate, tick.run_if(in_state(GameState::Training)))
        .add_systems(Update, feed.run_if(in_state(GameState::Training)))
        .add_systems(OnExit(GameState::Training), cleanup);
}

#[allow(clippy::too_many_arguments)]
fn apply(
    mut requests: ResMut<Requests>,
    mut catalog: ResMut<Catalog>,
    mut trial: ResMut<Trial>,
    mut shell: ResMut<Shell>,
    mut next: ResMut<NextState<GameState>>,
    mut link: NonSendMut<MatchLink>,
    mut form: NonSendMut<ActiveForm>,
    mut submitted: MessageReader<TextEntered>,
    mut queue: ResMut<crate::ui::ActionQueue>,
    lang: Res<Lang>,
    mut epoch: Local<u32>,
) {
    for text in submitted
        .read()
        .filter(|t| t.kind == TextKind::CatalogSearch)
    {
        catalog.search = text.value.chars().take(40).collect();
        textform::close(&mut form);
    }
    for action in std::mem::take(&mut requests.0) {
        if viewer(action, &mut catalog) || teams(action, &mut catalog) {
            continue;
        }
        if filter(action, &mut catalog) {
            continue;
        }
        match action {
            Action::Open(tab) => {
                catalog.tab = tab;
                shell.panel = Panel::Troops;
                shell.dirty = true;
                next.set(GameState::Menu);
            }
            Action::Search => textform::open(
                &mut form,
                TextKind::CatalogSearch,
                &lang.key("training.search"),
            ),
            Action::Start => {
                *epoch = epoch.wrapping_add(1);
                match TrainingSession::new(scenario(&catalog), SessionId(*epoch)) {
                    Ok(mut session) => {
                        let _ = session.command(session.id(), TrainingCommand::Start);
                        *link = MatchLink::Idle;
                        *trial = Trial {
                            session: Some(session),
                            dirty: true,
                            reset: true,
                            ..default()
                        };
                        next.set(GameState::Training);
                    }
                    Err(e) => catalog.notice = format!("{}: {e:?}", lang.key("training.invalid")),
                }
            }
            Action::Back => {
                shell.panel = Panel::Troops;
                shell.dirty = true;
                next.set(GameState::Menu);
            }
            Action::Practice => {
                crate::platform::storage::set(
                    crate::platform::storage::Scope::Local,
                    "practice.difficulty",
                    &catalog.difficulty.to_string(),
                );
                queue.0.push(crate::ui::UiAction::Practice);
            }
            Action::Target(id) => trial.target = Some(id),
            Action::Page(delta) => {
                trial.page = trial.page.saturating_add_signed(isize::from(delta))
            }
            other => command(other, &mut trial),
        }
    }
}

fn viewer(action: Action, c: &mut Catalog) -> bool {
    match action {
        Action::Select(kind) => {
            c.selected = kind;
            c.form = (0, 0);
        }
        Action::Form(t, b) => c.form = (t, b),
        Action::Clip(clip) => {
            c.clip = clip;
            c.revision = c.revision.wrapping_add(1);
        }
        Action::Tab(tab) => c.tab = tab,
        Action::Rotate(d) => c.yaw += f32::from(d) * 0.25,
        Action::Zoom(d) => c.zoom = (c.zoom + f32::from(d) * 0.1).clamp(0.6, 1.7),
        Action::Motion => {
            c.reduced = !c.reduced;
            crate::config::save_reduced_motion(c.reduced);
        }
        _ => return false,
    }
    true
}

fn filter(action: Action, c: &mut Catalog) -> bool {
    match action {
        Action::Faction => {
            let i = c.faction.map_or(0, |f| f.index() + 1);
            c.faction = tbg_core::FACTIONS.get(i).copied();
        }
        Action::Role => {
            let i = c
                .role
                .and_then(|r| tbg_core::ARCHETYPES.iter().position(|a| *a == r))
                .map_or(0, |i| i + 1);
            c.role = tbg_core::ARCHETYPES.get(i).copied();
        }
        Action::Arena => {
            c.arena = tbg_core::Arena::ALL[(c.arena.index() + 1) % tbg_core::Arena::ALL.len()];
            crate::config::save_arena(c.arena);
        }
        Action::Seed => c.seed = c.seed.wrapping_add(1),
        Action::Difficulty(d) => c.difficulty = d.min(3),
        _ => return false,
    }
    true
}

fn teams(action: Action, c: &mut Catalog) -> bool {
    match action {
        Action::Add(ally) => {
            if !tbg_core::FULL_POOL.contains(&c.selected) {
                return true;
            }
            let spec = c.spec();
            let team = if ally { &mut c.allies } else { &mut c.enemies };
            if team.len() < MAX_UNITS_PER_SIDE {
                team.push(spec);
            }
        }
        Action::Remove(ally, i) => {
            let team = if ally { &mut c.allies } else { &mut c.enemies };
            if i < team.len() {
                team.remove(i);
            }
        }
        Action::Move(ally, i, d) => move_unit(c, ally, i, d),
        Action::Opponent => c.dummies = !c.dummies,
        Action::DummyCount(d) => {
            c.dummy_count = c
                .dummy_count
                .saturating_add_signed(isize::from(d))
                .clamp(1, MAX_DUMMIES)
        }
        Action::DummyHp => {
            c.dummy_hp = match c.dummy_hp {
                super::DummyHp::Light => super::DummyHp::Heavy,
                super::DummyHp::Heavy => super::DummyHp::Light,
            }
        }
        Action::DummyMode => {
            c.dummy_mode = match c.dummy_mode {
                super::DummyMode::Finite => super::DummyMode::Renewable,
                super::DummyMode::Renewable => super::DummyMode::Finite,
            }
        }
        _ => return false,
    }
    true
}
fn move_unit(c: &mut Catalog, ally: bool, i: usize, d: i8) {
    let side = if ally {
        tbg_core::training::Side::Allies
    } else {
        tbg_core::training::Side::Opponents
    };
    let team = if ally { &mut c.allies } else { &mut c.enemies };
    if let Some(u) = team.get_mut(i) {
        let x =
            u.x.unwrap_or_else(|| tbg_core::training::default_x(side, false, i));
        u.x = Some(
            (x + i32::from(d) * 1_000)
                .clamp(tbg_core::training::LANE_MIN, tbg_core::training::LANE_MAX),
        );
    }
}
fn scenario(c: &Catalog) -> TrainingScenario {
    TrainingScenario {
        schema: SCHEMA,
        seed: c.seed,
        arena: c.arena,
        allies: c.allies.clone(),
        opponent: if c.dummies {
            Opponent::Dummies {
                mode: c.dummy_mode,
                hp: c.dummy_hp,
                positions: vec![None; c.dummy_count],
            }
        } else {
            Opponent::Champions(c.enemies.clone())
        },
    }
}
fn command(action: Action, trial: &mut Trial) {
    let Some(session) = &mut trial.session else {
        return;
    };
    let code = match action {
        Action::Control(id, code) if id == session.id() => code,
        _ => return,
    };
    let cmd = match code {
        0 => match session.lifecycle() {
            Lifecycle::Ready => TrainingCommand::Start,
            Lifecycle::Running => TrainingCommand::Pause,
            _ => TrainingCommand::Resume,
        },
        1 => TrainingCommand::Step,
        2 => TrainingCommand::Reset,
        _ => return,
    };
    if session.command(session.id(), cmd).is_ok() {
        trial.dirty = true;
        trial.reset = code == 2;
    }
}
fn tick(mut trial: ResMut<Trial>) {
    let Some(session) = &mut trial.session else {
        return;
    };
    let before = session.active_ticks();
    session.tick();
    if session.active_ticks() != before {
        trial.dirty = true;
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn feed(
    mut commands: Commands,
    time: Res<Time>,
    mut trial: ResMut<Trial>,
    mut latest: ResMut<LatestView>,
    mut buffer: ResMut<ViewBuffer>,
    mut entities: ResMut<UnitEntities>,
    mut mounts: ResMut<crate::visuals::mount::Mounts>,
    mut info: ResMut<MatchInfo>,
    mut arena: ResMut<crate::arena::ActiveArena>,
) {
    if !trial.dirty {
        return;
    }
    if trial.reset {
        clear(&mut commands, &mut entities, &mut mounts);
        *buffer = default();
        trial.reset = false;
    }
    trial.dirty = false;
    let Some(session) = &mut trial.session else {
        return;
    };
    let events = session.take_events();
    let view = session.state().view_for(Seat::A, &events);
    *info = MatchInfo {
        you: Some(Seat::A),
        you_name: "Laboratory".into(),
        opponent: "Training".into(),
        arena: session.scenario().arena,
        ..default()
    };
    arena.set_if_neq(crate::arena::ActiveArena::of(info.arena));
    latest.0 = Some(view.clone());
    buffer.timeline.push(time.elapsed_secs_f64(), view);
}
fn clear(
    commands: &mut Commands,
    entities: &mut UnitEntities,
    mounts: &mut crate::visuals::mount::Mounts,
) {
    for (_, e) in entities.0.drain() {
        commands.entity(e).try_despawn();
    }
    mounts.clear();
}
fn cleanup(
    mut commands: Commands,
    mut trial: ResMut<Trial>,
    mut entities: ResMut<UnitEntities>,
    mut mounts: ResMut<crate::visuals::mount::Mounts>,
    mut buffer: ResMut<ViewBuffer>,
    mut latest: ResMut<LatestView>,
) {
    clear(&mut commands, &mut entities, &mut mounts);
    *trial = default();
    *buffer = default();
    latest.0 = None;
}
