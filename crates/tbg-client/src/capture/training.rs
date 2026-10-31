//! Debug-only laboratory journey using the same identity-bound actions as visible controls.
use crate::{
    training::{Action, Requests, Tab, Trial},
    GameState,
};
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, journey);
}
fn journey(
    state: Res<State<GameState>>,
    time: Res<Time>,
    trial: Res<Trial>,
    mut queue: ResMut<Requests>,
    mut stage: Local<u8>,
    mut observed: Local<(u32, u32, f32)>,
) {
    if *stage == 0 && *state.get() == GameState::Menu {
        queue.0.push(Action::Open(Tab::Editor));
        *stage = 1;
        return;
    }
    if *stage == 1 && time.elapsed_secs() > 1. {
        queue.0.push(Action::Start);
        *stage = 2;
        return;
    }
    let Some(s) = &trial.session else { return };
    let id = s.id();
    match *stage {
        2 if s.active_ticks() >= 10 => {
            queue.0.push(Action::Control(id, 0));
            *observed = (id.0, s.active_ticks(), time.elapsed_secs());
            *stage = 3;
        }
        3 if time.elapsed_secs() > observed.2 + 0.5 => {
            assert!(matches!(
                s.lifecycle(),
                tbg_core::training::Lifecycle::Paused(_)
            ));
            observed.1 = s.active_ticks();
            queue.0.push(Action::Control(id, 1));
            *stage = 4;
        }
        4 if s.active_ticks() > observed.1 => {
            assert_eq!(s.active_ticks(), observed.1 + 1);
            queue.0.push(Action::Control(id, 2));
            *stage = 5;
        }
        5 if id.0 != observed.0 => {
            assert_eq!(s.active_ticks(), 0);
            queue.0.push(Action::Control(
                tbg_core::training::SessionId(observed.0),
                0,
            ));
            *stage = 6;
        }
        6 => {
            assert_eq!(
                s.active_ticks(),
                0,
                "stale controls must not start the reset trial"
            );
            queue.0.push(Action::Control(id, 0));
            *stage = 7;
        }
        7 if s.active_ticks() >= 20 => {
            assert_eq!(*state.get(), GameState::Training);
            assert!(s.state().outcome.is_none());
            eprintln!("Training journey passed: pause, exact single step, reset identity, stale action rejection and independent local rendering");
            *stage = 8;
        }
        _ => {}
    }
}
