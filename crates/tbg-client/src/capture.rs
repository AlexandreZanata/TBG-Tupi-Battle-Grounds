//! Opt-in debug render captures for UI parity checks without OS automation.

use bevy::{
    prelude::*,
    render::view::screenshot::{save_to_disk, Screenshot, ScreenshotCaptured},
};

#[cfg(target_os = "linux")]
mod arenas;
#[cfg(target_os = "linux")]
mod reserve;

pub fn plugin(app: &mut App) {
    if crate::config::url_param("capture").is_some() {
        app.add_systems(Update, capture);
        #[cfg(target_os = "linux")]
        if crate::config::url_param("capture_panel").as_deref() == Some("arena_train") {
            arenas::plugin(app);
        }
        #[cfg(target_os = "linux")]
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
        #[cfg(target_os = "linux")]
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
        #[cfg(target_os = "linux")]
        if crate::config::url_param("capture_panel").as_deref() == Some("fortify") {
            app.add_systems(
                OnEnter(crate::GameState::InMatch),
                |mut actions: ResMut<crate::ui::ActionQueue>| {
                    actions.0.push(crate::ui::UiAction::SelectTower);
                },
            );
        }
        #[cfg(target_os = "linux")]
        if crate::config::url_param("capture_panel").as_deref() == Some("unit") {
            app.add_systems(
                Update,
                select_unit.run_if(in_state(crate::GameState::InMatch)),
            );
        }
        #[cfg(target_os = "linux")]
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

#[cfg(target_os = "linux")]
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

fn capture(
    mut commands: Commands,
    time: Res<Time>,
    state: Res<State<crate::GameState>>,
    #[cfg(target_os = "linux")] view: Res<crate::sync::LatestView>,
    #[cfg(target_os = "linux")] tower: Res<crate::ui::fortify::TowerOpen>,
    #[cfg(target_os = "linux")] inspected: Res<crate::ui::inspect::Inspected>,
    #[cfg(target_os = "linux")] reserve: Option<Res<reserve::Status>>,
    #[cfg(target_os = "linux")] units: Query<&crate::visuals::UnitVisual>,
    #[cfg(target_os = "linux")] stats: Res<crate::sync::MatchStats>,
    #[cfg(target_os = "linux")] match_info: Res<crate::sync::MatchInfo>,
    #[cfg(target_os = "linux")] loading: Res<crate::ui::native_loading::Status>,
    mut started: Local<bool>,
) {
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
    #[cfg(target_os = "linux")]
    {
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
    #[cfg(target_os = "linux")]
    eprintln!("Loading capture: boot_ready={}, scene_waiting={}, pending_images={}, pending_texts={}, failed_images={}", loading.boot_ready, loading.scene_waiting, loading.images, loading.texts, loading.failed);
    #[cfg(target_os = "linux")]
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
    #[cfg(target_os = "linux")]
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

#[cfg(target_os = "linux")]
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
