//! Native update driver: detects a launcher-managed install, reports the
//! healthy boot, queues `notes` then `check`, follows `stage` progress,
//! pauses a download when online play starts, and applies on request by
//! exiting with the launcher's apply code. Downloads, verification and
//! extraction run in the helper process, never here.

use std::collections::VecDeque;
use std::time::Duration;

use bevy::prelude::*;
use serde_json::Value;

use super::helper::{Managed, Running};
use super::{UiAction, UpdState, Updater};
use crate::GameState;

/// The launcher activates a staged release when the game exits with this.
const EXIT_APPLY: u8 = 75;
/// After the healthy report, give the launcher this long to commit.
const NOTES_AFTER: [Duration; 2] = [Duration::from_secs(2), Duration::from_secs(6)];
/// Idle recheck period (P66 contracts §3: at most every 6 hours).
const RECHECK: Duration = Duration::from_secs(6 * 60 * 60);

#[derive(Default)]
pub(super) struct Inner {
    managed: Option<Managed>,
    running: Option<(String, Running)>,
    queued: VecDeque<Vec<String>>,
    reported_at: Option<f64>,
    notes_asked: usize,
    last_check: Option<f64>,
}

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, detect).add_systems(
        Update,
        (
            read_idle,
            report_health,
            schedule,
            pump,
            pause_online_download,
            automatic,
            super::view::overlay,
            super::view::badge,
        )
            .chain(),
    );
}

fn detect(mut updater: ResMut<Updater>) {
    updater.inner.managed = Managed::from_env();
    if updater.inner.managed.is_some() {
        updater.state = UpdState::Checking;
    }
}

fn read_idle(
    mut updater: ResMut<Updater>,
    state: Res<State<GameState>>,
    link: NonSend<crate::link::MatchLink>,
    form: NonSend<crate::ui::textform::ActiveForm>,
    account: Res<crate::ui::account::Account>,
) {
    updater.idle = *state.get() == GameState::Menu
        && !crate::ui::textform::is_open(&form)
        && matches!(account.screen, crate::ui::account::AccountScreen::None)
        && crate::config::resume_ready()
        && crate::auth::idle_for_update()
        && match &*link {
            crate::link::MatchLink::Net(n) => n.safe_for_update(),
            _ => true,
        };
}

pub(super) fn stage_busy(updater: &Updater) -> bool {
    updater
        .inner
        .running
        .as_ref()
        .is_some_and(|(command, _)| command == "stage")
        || updater
            .inner
            .queued
            .iter()
            .any(|args| args.first().is_some_and(|a| a == "stage"))
}

/// Healthy = the menu is up and the packaged assets loaded (P66 §3.2):
/// write `{nonce, build}` once so the launcher can commit a candidate.
fn report_health(
    mut updater: ResMut<Updater>,
    state: Res<State<GameState>>,
    status: Option<Res<crate::ui::native_loading::Status>>,
    time: Res<Time>,
) {
    let inner = &mut updater.inner;
    if inner.reported_at.is_some() || *state.get() != GameState::Menu {
        return;
    }
    if !status.is_some_and(|s| s.boot_ready && s.failed == 0) {
        return;
    }
    let Some(m) = inner.managed.clone() else {
        return;
    };
    if let Some((path, nonce)) = &m.health {
        let body = serde_json::json!({"nonce": nonce, "build": m.build}).to_string();
        let temp = path.with_extension("tmp");
        let written = std::fs::write(&temp, body).and_then(|_| std::fs::rename(&temp, path));
        if let Err(e) = written {
            warn!("could not report a healthy start: {e}");
        }
    }
    inner.reported_at = Some(time.elapsed_secs_f64());
}

/// Queue `notes` shortly after the healthy report, then `check`; recheck
/// while idle in the lobby every few hours.
fn schedule(mut updater: ResMut<Updater>, time: Res<Time>) {
    let now = time.elapsed_secs_f64();
    let idle = updater.idle;
    let inner = &mut updater.inner;
    let Some(reported) = inner.reported_at else {
        return;
    };
    if let Some(after) = NOTES_AFTER.get(inner.notes_asked) {
        if now - reported >= after.as_secs_f64() {
            inner.notes_asked += 1;
            inner.queued.push_back(vec!["notes".into()]);
        }
    }
    let due = inner
        .last_check
        .is_none_or(|t| now - t >= RECHECK.as_secs_f64());
    if idle && due && inner.notes_asked > 0 && inner.running.is_none() {
        inner.last_check = Some(now);
        inner.queued.push_back(vec!["check".into()]);
    }
    if idle && inner.running.is_none() {
        if let (Some(args), Some(m)) = (inner.queued.pop_front(), inner.managed.clone()) {
            let refs: Vec<&str> = args.iter().map(String::as_str).collect();
            match Running::start(&m, &refs) {
                Ok(r) => inner.running = Some((args[0].clone(), r)),
                Err(e) => warn!("update helper unavailable: {e}"),
            }
        }
    }
}

/// Apply helper events to the visible state.
fn pump(mut updater: ResMut<Updater>) {
    let Some((command, running)) = updater.inner.running.as_mut() else {
        return;
    };
    let command = command.clone();
    let events = running.drain();
    let done = running.finished() && events.is_empty();
    for event in events {
        on_event(&mut updater, &command, event);
    }
    if done {
        // Drain anything the reader thread delivered after the exit.
        let late = updater
            .inner
            .running
            .as_ref()
            .map(|(_, r)| r.drain())
            .unwrap_or_default();
        for event in late {
            on_event(&mut updater, &command, event);
        }
        updater.inner.running = None;
        if matches!(
            updater.state,
            UpdState::Checking | UpdState::Downloading { .. }
        ) {
            updater.state = UpdState::Error("other".into());
        }
    }
}

fn text(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn on_event(updater: &mut Updater, command: &str, e: Value) {
    match (command, e.get("event").and_then(Value::as_str)) {
        ("notes", Some("notes")) => {
            if e.get("pending").and_then(Value::as_bool) == Some(true) {
                updater.notes = Some(e);
            }
        }
        ("check", Some("status")) if text(&e, "state") == "available" => {
            crate::platform::storage::set(
                crate::platform::storage::Scope::Session,
                "update.required",
                if e.get("required").and_then(Value::as_bool) == Some(true) {
                    "1"
                } else {
                    "0"
                },
            );
            updater.state = UpdState::Available {
                build: text(&e, "build"),
                version: text(&e, "version"),
                size: e.get("size").and_then(Value::as_u64).unwrap_or(0),
                required: e.get("required").and_then(Value::as_bool).unwrap_or(false),
            };
        }
        ("check", Some("status")) => updater.state = UpdState::Off,
        ("stage", Some("progress")) => {
            if let UpdState::Downloading { done, total, .. } = &mut updater.state {
                *done = e.get("done").and_then(Value::as_u64).unwrap_or(*done);
                *total = e.get("total").and_then(Value::as_u64).unwrap_or(*total);
            }
        }
        ("stage", Some("staged")) => {
            let version = match &updater.state {
                UpdState::Downloading { version, .. } => version.clone(),
                _ => text(&e, "build"),
            };
            updater.state = UpdState::Staged { version };
        }
        ("stage", Some("cancelled")) => updater.state = UpdState::Off,
        (_, Some("error")) => {
            // A failed check never invalidates the installed game.
            updater.state = if command == "check" && !updater.opened {
                UpdState::Off
            } else {
                UpdState::Error(text(&e, "code"))
            };
        }
        _ => {}
    }
}

/// BR-UP02: a download never competes with online play. Leaving the lobby
/// cancels it; the modal offers it again once the player is idle.
fn pause_online_download(mut updater: ResMut<Updater>) {
    if updater.idle {
        return;
    }
    if let Some((command, running)) = updater.inner.running.as_mut() {
        if command == "stage" {
            running.cancel();
        }
    }
}

/// Handle one of the overlay's buttons.
pub(super) fn apply(
    updater: &mut Updater,
    action: UiAction,
    state: GameState,
    exit: &mut MessageWriter<AppExit>,
) {
    match action {
        UiAction::UpdateCheck if updater.idle => {
            updater.opened = true;
            if updater.inner.managed.is_none() {
                updater.state = UpdState::Error("setup".into());
                return;
            }
            updater.inner.last_check = None;
            updater.snoozed = None;
            updater.state = UpdState::Checking;
        }
        UiAction::UpdateOpen if updater.idle => {
            updater.opened = true;
            updater.snoozed = None;
            if updater.inner.managed.is_none() {
                updater.state = UpdState::Error("setup".into());
            } else {
                updater.inner.last_check = None;
                updater.state = UpdState::Checking;
            }
        }
        UiAction::UpdateLater => {
            updater.opened = false;
            match &updater.state {
                UpdState::Available { build, .. } => updater.snoozed = Some(build.clone()),
                UpdState::Staged { version } => updater.snoozed = Some(version.clone()),
                UpdState::Error(_) => updater.state = UpdState::Off,
                _ => {}
            }
        }
        UiAction::UpdateNow if state == GameState::Menu && updater.idle => {
            if let UpdState::Available {
                build,
                version,
                size,
                ..
            } = updater.state.clone()
            {
                updater
                    .inner
                    .queued
                    .push_back(vec!["stage".into(), build.clone()]);
                updater.state = UpdState::Downloading {
                    build,
                    version,
                    done: 0,
                    total: size,
                };
            }
        }
        UiAction::UpdateCancel => {
            if let UpdState::Downloading { build, .. } = &updater.state {
                updater.snoozed = Some(build.clone());
            }
            updater.opened = false;
            updater
                .inner
                .queued
                .retain(|args| args.first().is_none_or(|a| a != "stage"));
            if let Some((command, running)) = updater
                .inner
                .running
                .as_mut()
                .filter(|(command, _)| command == "stage")
            {
                let _ = command;
                running.cancel();
            } else {
                updater.state = UpdState::Off;
            }
        }
        // Only from the idle lobby; `take_actions` then drops every other
        // queued action so no match request races the exit.
        UiAction::UpdateRestart if state == GameState::Menu && updater.idle => {
            if matches!(updater.state, UpdState::Staged { .. }) {
                updater.state = UpdState::Applying;
                if let Some(code) = std::num::NonZeroU8::new(EXIT_APPLY) {
                    exit.write(AppExit::Error(code));
                }
            }
        }
        UiAction::UpdateNotesOk => {
            let build = updater.notes.as_ref().map(|n| text(n, "build"));
            updater.notes = None;
            if let Some(build) = build.filter(|b| !b.is_empty()) {
                updater.inner.queued.push_front(vec!["ack".into(), build]);
            }
        }
        _ => {}
    }
}

fn automatic(updater: Res<Updater>, mut queue: ResMut<super::ActionQueue>) {
    if !(updater.idle && updater.automatic) || queue.0.iter().any(super::starts_online) {
        return;
    }
    let action = match &updater.state {
        UpdState::Available { build, .. } if updater.snoozed.as_deref() != Some(build) => {
            Some(UiAction::UpdateNow)
        }
        UpdState::Staged { version } if updater.snoozed.as_deref() != Some(version) => {
            Some(UiAction::UpdateRestart)
        }
        _ => None,
    };
    if let Some(action) = action {
        queue.0.push(action);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn br_up02_automatic_updates_wait_for_idle_and_preserve_play_requests() {
        let mut app = App::new();
        app.insert_resource(Updater {
            automatic: true,
            idle: false,
            state: UpdState::Available {
                build: "test-build".into(),
                version: "0.1.4".into(),
                size: 1,
                required: false,
            },
            ..Default::default()
        })
        .init_resource::<super::super::ActionQueue>()
        .add_systems(Update, automatic);
        app.update();
        assert!(
            app.world()
                .resource::<super::super::ActionQueue>()
                .0
                .is_empty(),
            "BR-UP02: no update starts during online activity"
        );
        app.world_mut().resource_mut::<Updater>().idle = true;
        app.update();
        assert_eq!(
            app.world_mut()
                .resource_mut::<super::super::ActionQueue>()
                .0
                .pop(),
            Some(UiAction::UpdateNow)
        );
        app.world_mut().resource_mut::<Updater>().state = UpdState::Staged {
            version: "0.1.4".into(),
        };
        app.update();
        assert_eq!(
            app.world_mut()
                .resource_mut::<super::super::ActionQueue>()
                .0
                .pop(),
            Some(UiAction::UpdateRestart)
        );
        app.world_mut().resource_mut::<Updater>().automatic = false;
        app.update();
        assert!(app
            .world()
            .resource::<super::super::ActionQueue>()
            .0
            .is_empty());
    }

    #[test]
    fn br_up01_manual_check_reports_failures_without_invalidating_the_game() {
        let mut updater = Updater {
            opened: true,
            ..Default::default()
        };
        on_event(
            &mut updater,
            "check",
            serde_json::json!({"event":"error","code":"unavailable"}),
        );
        assert_eq!(updater.state, UpdState::Error("unavailable".into()));
        assert!(!updater.applying());
        on_event(
            &mut updater,
            "check",
            serde_json::json!({"event":"status","state":"current"}),
        );
        assert_eq!(updater.state, UpdState::Off);
        assert!(
            updater.opened,
            "Settings keeps its in-app status modal visible"
        );
    }
}

#[cfg(test)]
mod play_priority_tests {
    use super::*;
    #[test]
    fn br_up02_ranked_search_cancels_a_queued_download_and_is_not_discarded() {
        let mut updater = Updater {
            idle: true,
            automatic: true,
            state: UpdState::Downloading {
                build: "test".into(),
                version: "0.1.4".into(),
                done: 0,
                total: 1,
            },
            ..Default::default()
        };
        updater
            .inner
            .queued
            .push_back(vec!["stage".into(), "test".into()]);
        let mut app = App::new();
        app.insert_resource(updater)
            .insert_resource(State::new(GameState::Menu))
            .insert_resource(super::super::ActionQueue(vec![UiAction::Ranked]))
            .add_message::<AppExit>()
            .add_systems(Update, super::super::take_actions);
        app.update();
        assert_eq!(
            app.world().resource::<super::super::ActionQueue>().0,
            vec![UiAction::Ranked]
        );
        assert!(!stage_busy(app.world().resource::<Updater>()));
        assert!(!app.world().resource::<Updater>().applying());
    }
}

#[cfg(test)]
mod staged_priority_tests {
    use super::*;
    #[test]
    fn br_up02_ranked_click_wins_a_same_frame_automatic_restart() {
        let updater = Updater {
            idle: true,
            automatic: true,
            state: UpdState::Staged {
                version: "0.1.4".into(),
            },
            ..Default::default()
        };
        let mut app = App::new();
        app.insert_resource(updater)
            .insert_resource(State::new(GameState::Menu))
            .insert_resource(super::super::ActionQueue(vec![
                UiAction::UpdateRestart,
                UiAction::Ranked,
            ]))
            .add_message::<AppExit>()
            .add_systems(Update, super::super::take_actions);
        app.update();
        assert_eq!(
            app.world().resource::<super::super::ActionQueue>().0,
            vec![UiAction::Ranked]
        );
        assert!(matches!(
            app.world().resource::<Updater>().state,
            UpdState::Staged { .. }
        ));
    }
}
