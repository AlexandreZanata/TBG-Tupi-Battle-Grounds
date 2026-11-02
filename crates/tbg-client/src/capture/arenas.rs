//! Debug-only smoke capture of all Arena training buttons and one actual Practice action.

use crate::{
    ui::{ActionQueue, UiAction},
    GameState,
};
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, (start, launch).run_if(in_state(GameState::Menu)));
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
    if choices.len() != tbg_core::Arena::ALL.len() {
        return;
    }
    let mut arenas: Vec<_> = choices.iter().map(|(arena, _)| *arena).collect();
    arenas.sort_by_key(|a| a.index());
    arenas.dedup();
    assert_eq!(
        arenas.len(),
        tbg_core::Arena::ALL.len(),
        "Every Arena must expose its own training action"
    );
    let target = crate::config::url_param("capture_arena").unwrap_or("storm".into());
    let Some((arena, action)) = choices.iter().find(|(arena, _)| arena.slug() == target) else {
        return;
    };
    eprintln!(
        "Arena training capture: distinct buttons={arenas:?}; saved before={}; choosing={}",
        crate::config::load_arena().slug(),
        arena.slug()
    );
    queue.0.push(*action);
    *stage = 1;
}

/// Follow the same difficulty-setup Play action a player sees after Train here.
fn launch(
    buttons: Query<&crate::ui::shell::Action>,
    catalog: Res<crate::training::Catalog>,
    mut requests: ResMut<crate::training::Requests>,
    mut done: Local<bool>,
) {
    if *done
        || !buttons.iter().any(|action| {
            matches!(
                action,
                crate::ui::shell::Action::Catalog(crate::training::Action::Practice)
            )
        })
    {
        return;
    }
    let target = crate::config::url_param("capture_arena").unwrap_or("storm".into());
    assert_eq!(
        catalog.arena.slug(),
        target,
        "Practice setup must display the clicked Arena"
    );
    assert_eq!(crate::config::load_arena().slug(), target);
    requests.0.push(crate::training::Action::Practice);
    eprintln!(
        "Arena training capture: setup agrees with saved arena={target}; starting actual Practice"
    );
    *done = true;
}
