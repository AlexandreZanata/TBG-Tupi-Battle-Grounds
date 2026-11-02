//! Opt-in debug render captures for UI parity checks without OS automation.

use bevy::{
    prelude::*,
    render::view::screenshot::{save_to_disk, Screenshot, ScreenshotCaptured},
};

#[cfg(tbg_desktop)]
mod arenas;
#[cfg(tbg_desktop)]
mod focus;
#[cfg(tbg_desktop)]
mod polish;
#[cfg(tbg_desktop)]
mod polish_live;
#[cfg(tbg_desktop)]
mod polish_motion;
#[cfg(tbg_desktop)]
mod reserve;
// P73 synthetic fixtures exist only where their "LOCAL VISUAL FIXTURE" label
// is also compiled, never in a plain debug build.
#[cfg(tbg_desktop)]
mod barrel;
#[cfg(tbg_desktop)]
mod packing;
#[cfg(feature = "ui-acceptance")]
mod settings;
#[cfg(feature = "ui-acceptance")]
mod social;
#[cfg(tbg_desktop)]
mod training;
#[cfg(tbg_desktop)]
mod troops;

pub fn plugin(app: &mut App) {
    #[cfg(tbg_desktop)]
    if crate::config::url_param("capture_focus").is_some() {
        focus::plugin(app);
        return;
    }
    #[cfg(tbg_desktop)]
    if crate::config::url_param("capture_motion").is_some() {
        polish_motion::plugin(app);
        return;
    }
    #[cfg(tbg_desktop)]
    if crate::config::url_param("capture_polish_live").is_some() {
        polish_live::plugin(app);
        return;
    }
    #[cfg(tbg_desktop)]
    if crate::config::url_param("capture_polish").is_some() {
        polish::plugin(app);
        return;
    }
    if crate::config::url_param("capture").is_some() {
        #[cfg(tbg_desktop)]
        if crate::config::url_param("capture_panel").as_deref() == Some("troops") {
            troops::plugin(app);
            return;
        }
        app.add_systems(Update, capture);
        #[cfg(feature = "ui-acceptance")]
        if matches!(
            crate::config::url_param("capture_panel").as_deref(),
            Some("account-backup" | "updates")
        ) {
            settings::plugin(app);
        }
        #[cfg(feature = "ui-acceptance")]
        if crate::config::url_param("capture_panel").is_some_and(|v| v.starts_with("p73-")) {
            social::plugin(app);
        }
        #[cfg(tbg_desktop)]
        if crate::config::url_param("capture_panel").as_deref() == Some("training") {
            training::plugin(app);
        }
        #[cfg(tbg_desktop)]
        if crate::config::url_param("capture_panel").as_deref() == Some("arena_train") {
            arenas::plugin(app);
        }
        #[cfg(tbg_desktop)]
        if crate::config::url_param("capture_panel").as_deref() == Some("demo_transition") {
            app.add_systems(
                Update,
                |time: Res<Time>,
                 state: Res<State<crate::GameState>>,
                 mut actions: ResMut<crate::ui::ActionQueue>,
                 mut done: Local<bool>| {
                    if !*done && time.elapsed_secs() > 5. && *state.get() == crate::GameState::Menu
                    {
                        *done = true;
                        actions.0.push(crate::ui::UiAction::Practice);
                        eprintln!(
                            "Lobby demo transition: starting Practice through the normal UiAction."
                        );
                    }
                },
            );
        }
        #[cfg(tbg_desktop)]
        if crate::config::url_param("capture_panel").as_deref() == Some("ranking") {
            if crate::config::url_param("capture_ranking_fixture").is_some() {
                app.add_systems(Update, ranking_fixture);
            }
            app.add_systems(
                OnEnter(crate::GameState::Menu),
                |mut actions: ResMut<crate::ui::ActionQueue>| {
                    actions.0.push(crate::ui::UiAction::OpenRanking);
                },
            );
        }
        #[cfg(tbg_desktop)]
        if crate::config::url_param("capture_panel").as_deref() == Some("fortify") {
            app.add_systems(
                OnEnter(crate::GameState::InMatch),
                |mut actions: ResMut<crate::ui::ActionQueue>| {
                    actions.0.push(crate::ui::UiAction::SelectTower);
                },
            );
        }
        #[cfg(tbg_desktop)]
        if crate::config::url_param("capture_panel").as_deref() == Some("unit") {
            app.add_systems(
                Update,
                select_unit.run_if(in_state(crate::GameState::InMatch)),
            );
        }
        #[cfg(tbg_desktop)]
        if matches!(
            crate::config::url_param("capture_panel").as_deref(),
            Some("council" | "reserve")
        ) {
            app.add_systems(
                OnEnter(crate::GameState::Menu),
                |mut actions: ResMut<crate::ui::ActionQueue>| {
                    actions.0.push(crate::ui::UiAction::Practice);
                },
            );
            if crate::config::url_param("capture_panel").as_deref() == Some("reserve") {
                reserve::plugin(app);
            }
        }
    }
}

#[cfg(tbg_desktop)]
fn ranking_fixture(mut ranking: ResMut<crate::ui::ranking::Ranking>, mut done: Local<bool>) {
    if *done || !ranking.open {
        return;
    }
    *done = true;
    let rows = [
        ("CommanderOfStars", 1250, 98, 12, 3),
        ("SilverKnight", 1180, 40, 19, 0),
        ("BronzeGuardian", 1100, 28, 17, 2),
    ]
    .into_iter()
    .enumerate()
    .map(
        |(i, (name, rating, wins, losses, draws))| tbg_core::protocol::LeaderRow {
            rank: i as u32 + 1,
            name: name.into(),
            rating,
            wins,
            losses,
            draws,
        },
    )
    .collect();
    ranking.accept(3, 0, rows, None);
    eprintln!("Ranking capture uses explicit layout fixtures; these are not production accounts.");
}

#[allow(clippy::too_many_arguments)] // Bevy declares independent resources and component filters in the system signature.
fn capture(
    mut commands: Commands,
    time: Res<Time>,
    state: Res<State<crate::GameState>>,
    #[cfg(tbg_desktop)] view: Res<crate::sync::LatestView>,
    #[cfg(tbg_desktop)] tower: Res<crate::ui::fortify::TowerOpen>,
    #[cfg(tbg_desktop)] inspected: Res<crate::ui::inspect::Inspected>,
    #[cfg(tbg_desktop)] reserve: Option<Res<reserve::Status>>,
    #[cfg(tbg_desktop)] units: Query<&crate::visuals::UnitVisual>,
    #[cfg(tbg_desktop)] stats: Res<crate::sync::MatchStats>,
    #[cfg(tbg_desktop)] match_info: Res<crate::sync::MatchInfo>,
    #[cfg(tbg_desktop)] loading: Res<crate::ui::native_loading::Status>,
    #[cfg(tbg_desktop)] support_probe: Option<Res<training::SupportProbe>>,
    #[cfg(tbg_desktop)] barrel_probe: Option<Res<barrel::Status>>,
    #[cfg(tbg_desktop)] packing_probe: Option<Res<packing::Status>>,
    mut started: Local<bool>,
    #[cfg(tbg_desktop)] mut ready_at: Local<Option<f32>>,
) {
    #[cfg(tbg_desktop)]
    {
        if !loading.boot_ready || loading.scene_waiting {
            *ready_at = None;
            return;
        }
        let ready = *ready_at.get_or_insert(time.elapsed_secs());
        if time.elapsed_secs() - ready < 0.8 {
            return;
        }
    }
    let delay = crate::config::url_param("capture_after")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(8.0)
        .clamp(1.0, 600.0);
    let result_only = crate::config::url_param("capture_on").as_deref() == Some("result");
    if *started
        || time.elapsed_secs() < delay
        || (result_only && *state.get() != crate::GameState::Result)
    {
        return;
    }
    #[cfg(tbg_desktop)]
    {
        // Shader/font work can outlive the requested delay on a busy machine.
        // Capture the ready scene, never the loading overlay in front of it.
        if !loading.boot_ready || loading.scene_waiting {
            return;
        }
        if packing_probe.is_some_and(|probe| !probe.ready) {
            return;
        }
        if barrel_probe.is_some_and(|probe| !probe.ready) {
            return;
        }
        if support_probe.is_some_and(|probe| !probe.ready) {
            return;
        }
        if crate::config::url_param("capture_panel").as_deref() == Some("arena_train")
            && *state.get() != crate::GameState::InMatch
        {
            return;
        }
        if crate::config::url_param("capture_panel").as_deref() == Some("reserve")
            && reserve.is_none_or(|status| !status.done)
        {
            return;
        }
        if crate::config::url_param("capture_panel").as_deref() == Some("council")
            && view.0.as_ref().is_none_or(|v| v.augment_round.is_none())
        {
            return;
        }
        if crate::config::url_param("capture_panel").as_deref() == Some("fortify") && !tower.0 {
            return;
        }
        if crate::config::url_param("capture_panel").as_deref() == Some("unit")
            && inspected.0.is_none()
        {
            return;
        }
        // Capture actual match phases instead of racing startup/loading time.
        let wanted = match crate::config::url_param("capture_on").as_deref() {
            Some("countdown") => Some(tbg_core::Phase::Countdown),
            Some("running") => Some(tbg_core::Phase::Running),
            _ => None,
        };
        if wanted.is_some_and(|phase| view.0.as_ref().is_none_or(|v| v.phase != phase)) {
            return;
        }
    }
    let Some(path) = crate::config::url_param("capture") else {
        return;
    };
    *started = true;
    #[cfg(tbg_desktop)]
    eprintln!("Loading capture: boot_ready={}, scene_waiting={}, pending_images={}, pending_texts={}, failed_images={}", loading.boot_ready, loading.scene_waiting, loading.images, loading.texts, loading.failed);
    #[cfg(tbg_desktop)]
    if crate::config::url_param("capture_panel").as_deref() == Some("arena_train") {
        let expected = crate::config::url_param("capture_arena").unwrap_or("storm".into());
        assert_eq!(
            match_info.arena.slug(),
            expected,
            "Practice must start in the clicked Arena"
        );
        eprintln!(
            "Arena training capture: actual Practice arena={}",
            match_info.arena.slug()
        );
    }
    #[cfg(tbg_desktop)]
    if *state.get() == crate::GameState::Menu {
        let kinds: Vec<_> = units
            .iter()
            .filter(|v| !v.dying)
            .map(|v| v.kind())
            .collect();
        eprintln!("Lobby capture: live units={kinds:?}; real LatestView present={}; real match purchases={}, evolutions={}, damage={}", view.0.is_some(), stats.bought, stats.evolutions, stats.damage_dealt);
    }
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk(path))
        .observe(
            |_: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>| {
                exit.write(AppExit::Success);
            },
        );
}

#[cfg(tbg_desktop)]
fn select_unit(
    view: Res<crate::sync::LatestView>,
    mut actions: ResMut<crate::ui::ActionQueue>,
    mut done: Local<bool>,
) {
    if *done {
        return;
    }
    let Some(v) = &view.0 else { return };
    if let Some(unit) = v
        .units
        .iter()
        .find(|unit| unit.seat == v.you && !unit.is_summon())
    {
        actions.0.push(crate::ui::UiAction::SelectUnit(unit.id));
        *done = true;
    }
}
