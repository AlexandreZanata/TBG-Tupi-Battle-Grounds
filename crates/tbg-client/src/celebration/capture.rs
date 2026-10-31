//! Opt-in debug-only P63 rendering/lifecycle fixture using real core purchases and outcomes.

use super::{scene::Owned, Presentation};
use crate::{camera::ArenaCamera, sync::MatchInfo, GameState};
use bevy::prelude::*;

#[derive(Resource, Default)]
struct Probe {
    started: bool,
    cycled: u32,
    frames: u32,
    baseline: Option<usize>,
    ready: bool,
}

pub(super) fn plugin(app: &mut App) {
    if crate::config::url_param("p63_fixture").is_none() {
        return;
    }
    app.init_resource::<Probe>()
        .add_systems(Startup, size)
        .add_systems(Update, drive);
}

fn number(name: &str, fallback: u32) -> u32 {
    crate::config::url_param(name)
        .and_then(|s| s.parse().ok())
        .unwrap_or(fallback)
}

fn size(mut window: Single<&mut Window>) {
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(size) = crate::config::url_param("p63_size") {
        if let Some((w, h)) = size
            .split_once('x')
            .and_then(|(w, h)| Some((w.parse::<u32>().ok()?, h.parse::<u32>().ok()?)))
        {
            window.resolution = bevy::window::WindowResolution::new(w, h);
        }
    }
    let _ = &mut window;
}

fn fixture(cycle: u32) -> MatchInfo {
    use tbg_core::{MatchState, Phase, PlayerIntention, Seat};
    let seat = if crate::config::url_param("p63_seat").as_deref() == Some("B") {
        Seat::B
    } else {
        Seat::A
    };
    let count = number("p63_count", 9).min(128);
    let mut state = MatchState::new(63);
    while state.phase != Phase::Running {
        state.step();
    }
    state.players[seat.index()].energy = 10_000_000;
    let kinds = tbg_core::FULL_POOL;
    for i in 0..count {
        state.players[seat.index()].offers[0] = kinds[i as usize % kinds.len()];
        state
            .apply_intention(seat, i * 3 + 1, PlayerIntention::Purchase { offer: 0 })
            .expect("trusted capture purchase");
        let id = state.units.last().expect("purchased capture unit").id;
        if i % 3 == 0 {
            for seq in [i * 3 + 2, i * 3 + 3] {
                state
                    .apply_intention(
                        seat,
                        seq,
                        PlayerIntention::Evolve {
                            unit: id,
                            branch: (i % 2) as u8,
                        },
                    )
                    .expect("trusted capture Evolution");
            }
        }
    }
    for (i, u) in state.units.iter_mut().enumerate() {
        if i % 2 == 0 {
            u.hp = 0;
        }
    }
    state.step();
    state.forfeit(seat.other());
    let cast = state.celebration_cast().expect("exact capture history");
    assert_eq!(cast.total(seat), Some(u64::from(count)));
    let mut id = [63; 16];
    id[..4].copy_from_slice(&cycle.to_le_bytes());
    MatchInfo {
        result_id: Some(id),
        celebration: Some(cast),
        you: Some(seat),
        you_name: "P63 fixture".into(),
        opponent: "Preview fixture".into(),
        outcome: state.outcome,
        last_tick: state.tick,
        arena: state.arena,
        ..default()
    }
}

#[allow(clippy::too_many_arguments)]
fn drive(
    mut commands: Commands,
    time: Res<Time>,
    state: Res<State<GameState>>,
    mut next: ResMut<NextState<GameState>>,
    mut info: ResMut<MatchInfo>,
    mut options: ResMut<Presentation>,
    mut probe: ResMut<Probe>,
    owned: Query<Entity, With<Owned>>,
    all: Query<Entity>,
    cameras: Query<Entity, With<ArenaCamera>>,
    stats: Res<crate::sync::MatchStats>,
) {
    if time.elapsed_secs() < 1. {
        return;
    }
    if !probe.started {
        probe.started = true;
        *info = fixture(0);
        next.set(GameState::Result);
        return;
    }
    let cycles = number("p63_cycles", 0).min(20);
    if *state.get() == GameState::Menu && probe.cycled > 0 {
        assert!(owned.is_empty());
        assert_eq!(cameras.iter().count(), 1);
        let entities = all.iter().count();
        if let Some(baseline) = probe.baseline {
            assert_eq!(
                entities, baseline,
                "celebration leaked entities across Menu cycles"
            );
        } else {
            probe.baseline = Some(entities);
        }
        *info = fixture(probe.cycled);
        next.set(GameState::Result);
        probe.frames = 0;
        return;
    }
    if *state.get() != GameState::Result {
        return;
    }
    probe.frames += 1;
    if probe.cycled < cycles && probe.frames > 4 {
        probe.cycled += 1;
        next.set(GameState::Menu);
        return;
    }
    if probe.frames == 1 {
        let count = super::total(&info).expect("fixture eligible");
        options.page =
            u64::from(number("p63_page", 0)).min(count.saturating_sub(1) / super::PAGE_SIZE);
    }
    if time.elapsed_secs() < number("p63_after", 7) as f32 || probe.frames < 5 || probe.ready {
        return;
    }
    let count = super::total(&info).expect("fixture eligible");
    let (a, b) = super::range(count, options.page);
    assert_eq!(owned.iter().count(), (b - a) as usize);
    assert_eq!(
        (stats.bought, stats.evolutions, stats.damage_dealt),
        (0, 0, 0)
    );
    assert_eq!(cameras.iter().count(), 1);
    probe.ready = true;
    #[cfg(not(target_arch = "wasm32"))]
    {
        eprintln!("P63 fixture: seat={:?}; count={count}; range={a}..{b}; figures={}; cycles={}; entities={}; game_metrics=0; reduced={}",info.you,owned.iter().count(),probe.cycled,all.iter().count(),options.reduced);
        if let Some(path) = crate::config::url_param("p63_capture") {
            use bevy::render::view::screenshot::{save_to_disk, Screenshot, ScreenshotCaptured};
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(path))
                .observe(
                    |_: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>| {
                        exit.write(AppExit::Success);
                    },
                );
        }
    }
    #[cfg(target_arch = "wasm32")]
    if let Some(body) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.body())
    {
        let _ = body.set_attribute("data-p63-ready", "true");
        let _ = body.set_attribute("data-p63-figures", &(b - a).to_string());
        let _ = body.set_attribute("data-p63-cycles", &probe.cycled.to_string());
    }
    let _ = &mut commands;
}
