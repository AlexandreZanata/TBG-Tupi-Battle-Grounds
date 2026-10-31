//! Debug-only smoke capture of all Arena training buttons and one actual Practice action.

use crate::{
    ui::{ActionQueue, UiAction},
    GameState,
};
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, start.run_if(in_state(GameState::Menu)));
}

fn start(
    time: Res<Time>,
    buttons: Query<&UiAction>,
    mut queue: ResMut<ActionQueue>,
    mut done: Local<bool>,
) {
    if *done || time.elapsed_secs() < 1. {
        return;
    }
    let choices: Vec<_> = buttons
        .iter()
        .filter_map(|action| match action {
            UiAction::PracticeArena(arena) => Some((*arena, *action)),
            _ => None,
        })
        .collect();
    if choices.len() != 5 {
        return;
    }
    let mut arenas: Vec<_> = choices.iter().map(|(arena, _)| *arena).collect();
    arenas.sort_by_key(|a| a.index());
    arenas.dedup();
    assert_eq!(
        arenas.len(),
        5,
        "Every Arena must expose its own training action"
    );
    let target = crate::config::url_param("capture_arena").unwrap_or("storm".into());
    let Some((arena, action)) = choices.iter().find(|(arena, _)| arena.slug() == target) else {
        return;
    };
    eprintln!(
        "Arena training capture: five distinct buttons={arenas:?}; saved before={}; choosing={}",
        crate::config::load_arena().slug(),
        arena.slug()
    );
    queue.0.push(*action);
    *done = true;
}
