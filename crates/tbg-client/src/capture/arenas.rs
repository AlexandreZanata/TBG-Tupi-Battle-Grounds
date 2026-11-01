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
    catalog: Res<crate::training::Catalog>,
    shell: Res<crate::ui::shell::Shell>,
    mut training: ResMut<crate::training::Requests>,
    mut stage: Local<u8>,
) {
    if *stage >= 2 || time.elapsed_secs() < 1. {
        return;
    }
    if *stage == 1 {
        if shell.panel == crate::ui::shell::Panel::Troops
            && catalog.tab == crate::training::Tab::Practice
        {
            let target = crate::config::url_param("capture_arena").unwrap_or("storm".into());
            assert_eq!(
                catalog.arena.slug(),
                target,
                "Practice setup must show the Arena selected on its page"
            );
            training.0.push(crate::training::Action::Practice);
            *stage = 2;
        }
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
    *stage = 1;
}
