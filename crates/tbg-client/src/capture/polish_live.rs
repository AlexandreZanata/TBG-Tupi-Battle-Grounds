//! Opt-in P70 roster recordings drive ordinary laboratory actions and observe canonical events.

use crate::{
    camera::ArenaCamera,
    config,
    sync::LatestView,
    training::{Action, Catalog, Requests, Tab, Trial},
    GameState,
};
use bevy::prelude::*;
use serde_json::{json, Value};
use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};
use tbg_core::{
    training::{Lifecycle, TrainingUnitSpec},
    Arena, Seat, UnitKind,
};

#[derive(Resource)]
struct Settings {
    directory: PathBuf,
    unit: UnitKind,
    form: (u8, u8),
    seat: Seat,
    seed: u64,
    distance: i32,
    seconds: f64,
}
impl Settings {
    fn read() -> Result<Self, String> {
        let name = config::url_param("unit").ok_or("missing unit")?;
        let unit = tbg_core::FULL_POOL
            .into_iter()
            .find(|kind| format!("{kind:?}").eq_ignore_ascii_case(&name))
            .ok_or("unit must belong to the frozen 46-champion roster")?;
        let tier = number("tier", 0)?;
        let branch = number("branch", 0)?;
        if tier > 2 || branch > 1 || (tier == 0 && branch != 0) {
            return Err("invalid pilot form".into());
        }
        let seat = match config::url_param("trial_seat").as_deref() {
            None | Some("A") => Seat::A,
            Some("B") => Seat::B,
            _ => return Err("trial_seat must be A or B".into()),
        };
        let distance = number(
            "trial_distance",
            match unit {
                UnitKind::OldFinn => 4_300,
                UnitKind::PyreWarden => 2_000,
                UnitKind::Trident => 1_500,
                _ => (tbg_core::content::effective_stats(unit, tier as u8, branch as u8, 0).range
                    + 500)
                    .clamp(1_500, 12_000) as u64,
            },
        )?;
        let seconds = number("trial_seconds", 16)?;
        if !(500..=12_000).contains(&distance) || !(12..=18).contains(&seconds) {
            return Err("distance must be 500..12000; seconds must be 12..18".into());
        }
        Ok(Self {
            directory: config::url_param("capture_polish_live")
                .ok_or("missing output directory")?
                .into(),
            unit,
            form: (tier as u8, branch as u8),
            seat,
            seed: number("trial_seed", 70)?,
            distance: distance as i32,
            seconds: seconds as f64,
        })
    }
}
fn number(name: &str, default: u64) -> Result<u64, String> {
    config::url_param(name).map_or(Ok(default), |v| {
        v.parse().map_err(|_| format!("invalid {name}"))
    })
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Stage {
    Configure,
    Pause,
    Reset,
    Ready,
    Recorder,
    Starting,
    Running,
    Finished,
}
#[derive(Resource)]
struct Recording {
    stage: Stage,
    ready_frames: u32,
    started: f64,
    initial_tick: u32,
    last_tick: Option<u32>,
    frames: u64,
    journal: File,
    finished: f64,
}

pub fn plugin(app: &mut App) {
    let result = Settings::read().and_then(|settings| {
        let journal = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(settings.directory.join("events.jsonl"))
            .map_err(|e| e.to_string())?;
        Ok((settings, journal))
    });
    let (settings, journal) = match result {
        Ok(value) => value,
        Err(message) => {
            error!("P70 live capture configuration: {message}");
            app.add_systems(Startup, |mut exit: MessageWriter<AppExit>| {
                exit.write(AppExit::error());
            });
            return;
        }
    };
    app.insert_resource(settings)
        .insert_resource(Recording {
            stage: Stage::Configure,
            ready_frames: 0,
            started: 0.0,
            initial_tick: 0,
            last_tick: None,
            frames: 0,
            journal,
            finished: 0.0,
        })
        .add_systems(
            Update,
            (prepare, record)
                .chain()
                .after(crate::training::runtime::feed),
        );
}

/// These are editor placements, never direct writes to a running MatchState.
fn composition(settings: &Settings) -> (Vec<TrainingUnitSpec>, Vec<TrainingUnitSpec>) {
    let start = (tbg_core::content::LANE_LEN - settings.distance) / 2;
    let mirror = |x| {
        if settings.seat == Seat::A {
            x
        } else {
            tbg_core::content::LANE_LEN - x
        }
    };
    let mut shooters = vec![TrainingUnitSpec {
        kind: settings.unit,
        tier: settings.form.0,
        branch: settings.form.1,
        x: Some(mirror(start)),
    }];
    let definition = tbg_core::content::def(settings.unit);
    let frontline = matches!(settings.unit, UnitKind::PyreWarden | UnitKind::OldFinn)
        || matches!(
            definition.archetype,
            tbg_core::Archetype::Support | tbg_core::Archetype::Control
        )
        || definition.range > 1_500;
    if frontline {
        // An ordinary high-HP ally stays in Pyre's initial segment and protects Finn's repeated casts.
        shooters.push(TrainingUnitSpec {
            x: Some(mirror(start + settings.distance / 2)),
            ..TrainingUnitSpec::base(UnitKind::Capybara)
        });
    }
    let target = if settings.unit == UnitKind::Trident {
        UnitKind::Capybara
    } else {
        UnitKind::BoneBrute
    };
    let targets = vec![TrainingUnitSpec {
        x: Some(mirror(start + settings.distance)),
        ..TrainingUnitSpec::base(target)
    }];
    if settings.seat == Seat::A {
        (shooters, targets)
    } else {
        (targets, shooters)
    }
}

#[allow(clippy::too_many_arguments)] // The capture observes readiness and uses the existing editor/action resources.
fn prepare(
    settings: Res<Settings>,
    mut recording: ResMut<Recording>,
    state: Res<State<GameState>>,
    mut catalog: ResMut<Catalog>,
    trial: Res<Trial>,
    mut requests: ResMut<Requests>,
    loading: Res<crate::ui::native_loading::Status>,
    time: Res<Time<Real>>,
    window: Single<&Window>,
    camera: Single<&Transform, With<ArenaCamera>>,
    mut exit: MessageWriter<AppExit>,
) {
    if recording.stage == Stage::Configure && *state.get() == GameState::Menu && loading.boot_ready
    {
        let (allies, enemies) = composition(&settings);
        catalog.seed = settings.seed;
        catalog.selected = settings.unit;
        catalog.form = settings.form;
        catalog.allies = allies;
        catalog.enemies = enemies;
        catalog.dummies = false;
        catalog.arena = Arena::Forest;
        requests
            .0
            .extend([Action::Open(Tab::Editor), Action::Start]);
        recording.stage = Stage::Pause;
        return;
    }
    let Some(session) = &trial.session else {
        return;
    };
    let id = session.id();
    match recording.stage {
        Stage::Pause if *state.get() == GameState::Training => {
            requests.0.push(Action::Control(id, 0));
            recording.stage = Stage::Reset;
        }
        Stage::Reset if matches!(session.lifecycle(), Lifecycle::Paused(_)) => {
            requests.0.push(Action::Control(id, 2));
            recording.stage = Stage::Ready;
        }
        Stage::Ready if session.lifecycle() == Lifecycle::Ready => {
            recording.ready_frames = if loading.boot_ready && !loading.scene_waiting {
                recording.ready_frames.saturating_add(1)
            } else {
                0
            };
            if recording.ready_frames >= 30 {
                let metadata = json!({
                    "unit": format!("{:?}", settings.unit), "tier": settings.form.0, "branch": settings.form.1,
                    "shooter_seat": format!("{:?}", settings.seat), "viewer_seat": "A", "seed": settings.seed,
                    "initial_distance": settings.distance, "requested_wall_seconds": settings.seconds,
                    "subject_id": session.state().units.iter().find(|unit|
                        unit.kind == settings.unit && unit.seat == settings.seat
                        && (unit.tier, unit.branch) == settings.form).map(|unit| unit.id),
                    "active_ticks": session.active_ticks(), "scenario": format!("{:?}", session.scenario()),
                    "quality": config::url_param("quality").unwrap_or_else(|| "standard".into()),
                    "viewport": [window.physical_width(), window.physical_height()], "dpi": window.scale_factor(),
                    "camera": {"preset": "normal-training-match", "translation": camera.translation.to_array(),
                        "rotation": camera.rotation.to_array()},
                    "clock": "real-time, unmodified canonical training fixed step",
                    "limitations": "One configured engagement; no full clip, result Victory or transition coverage claim."
                });
                if let Err(error) = write_new(&settings.directory, "ready.json", &metadata) {
                    fail(&mut recording, &mut exit, error);
                    return;
                }
                eprintln!("P70_LIVE_READY {metadata}");
                recording.stage = Stage::Recorder;
            }
        }
        Stage::Recorder if settings.directory.join("recording.start").is_file() => {
            requests.0.push(Action::Control(id, 0));
            recording.stage = Stage::Starting;
        }
        Stage::Starting if session.lifecycle() == Lifecycle::Running => {
            recording.started = time.elapsed_secs_f64();
            recording.initial_tick = session.active_ticks();
            recording.last_tick = None;
            recording.stage = Stage::Running;
            let metadata = json!({"app_real_seconds": recording.started, "initial_active_ticks": recording.initial_tick});
            if let Err(error) = write_new(&settings.directory, "started.json", &metadata) {
                fail(&mut recording, &mut exit, error);
            }
        }
        _ => {}
    }
}

fn record(
    settings: Res<Settings>,
    mut recording: ResMut<Recording>,
    time: Res<Time<Real>>,
    trial: Res<Trial>,
    latest: Res<LatestView>,
    mut requests: ResMut<Requests>,
    mut exit: MessageWriter<AppExit>,
) {
    if recording.stage == Stage::Finished {
        if time.elapsed_secs_f64() >= recording.finished + 2.0 {
            exit.write(AppExit::Success);
        }
        return;
    }
    if recording.stage != Stage::Running {
        return;
    }
    let Some(session) = &trial.session else {
        return;
    };
    let elapsed = time.elapsed_secs_f64() - recording.started;
    recording.frames += 1;
    if let Some(view) = &latest.0 {
        if recording.last_tick != Some(view.tick) {
            recording.last_tick = Some(view.tick);
            let sample = json!({"wall_seconds": elapsed, "active_ticks": session.active_ticks(),
                "simulation_tick": view.tick, "events": view.events, "units": view.units});
            if let Err(error) = writeln!(recording.journal, "{sample}") {
                fail(&mut recording, &mut exit, error);
                return;
            }
        }
    }
    if elapsed < settings.seconds {
        return;
    }
    let rows: Vec<_> = session
        .metrics()
        .rows()
        .iter()
        .map(|row| {
            json!({
                "id": row.id, "unit": format!("{:?}", row.kind), "seat": format!("{:?}", row.seat),
                "tier": row.tier, "branch": row.branch, "initial_hp": row.initial_hp,
                "dealt": row.dealt_total(), "dealt_friendly": row.dealt_friendly,
                "healed_allies": row.healed_allies, "healed_self": row.healed_self,
                "summoner": row.summoner, "shield_absorbed": row.shield_absorbed,
                "plates_absorbed": row.plates_absorbed,
                "taken": row.taken_total(), "died_at": row.died_at,
            })
        })
        .collect();
    let summary = json!({
        "wall_seconds": elapsed, "active_ticks": session.active_ticks(),
        "observed_active_ticks": session.active_ticks().saturating_sub(recording.initial_tick),
        "tick_hz": tbg_core::content::TICK_HZ, "render_updates": recording.frames,
        "lifecycle": format!("{:?}", session.lifecycle()), "metrics": rows,
        "metrics_dropped": session.metrics().dropped,
        "claim": "Observed real training engagement only; inspect events and video for cue coverage."
    });
    if let Err(error) = recording
        .journal
        .flush()
        .and_then(|_| write_new(&settings.directory, "finished.json", &summary))
    {
        fail(&mut recording, &mut exit, error);
        return;
    }
    eprintln!("P70_LIVE_FINISHED {summary}");
    if session.lifecycle() == Lifecycle::Running {
        requests.0.push(Action::Control(session.id(), 0));
    }
    recording.finished = time.elapsed_secs_f64();
    recording.stage = Stage::Finished;
}
fn write_new(directory: &Path, name: &str, value: &Value) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(directory.join(name))?;
    writeln!(file, "{value}")
}
fn fail(recording: &mut Recording, exit: &mut MessageWriter<AppExit>, error: std::io::Error) {
    error!("P70 live capture output: {error}");
    recording.stage = Stage::Finished;
    recording.finished = f64::INFINITY;
    exit.write(AppExit::error());
}
