//! Opt-in native Practice UI regression: retain Evolution focus and separate purchase cards.
use crate::{
    sync::LatestView,
    ui::{ActionQueue, UiAction},
    GameState,
};
use bevy::{
    prelude::*,
    render::view::screenshot::{save_to_disk, Screenshot},
};
use std::path::PathBuf;

#[derive(Resource, Default)]
struct Flow {
    stage: u8,
    since: f64,
    id: Option<u32>,
}

pub fn plugin(app: &mut App) {
    app.init_resource::<Flow>().add_systems(Update, run);
}

fn run(
    mut commands: Commands,
    time: Res<Time<Real>>,
    state: Res<State<GameState>>,
    view: Res<LatestView>,
    inspected: Res<crate::ui::inspect::Inspected>,
    selected: Res<crate::ui::evolve::Selected>,
    mut queue: ResMut<ActionQueue>,
    mut flow: ResMut<Flow>,
    mut exit: MessageWriter<AppExit>,
) {
    let now = time.elapsed_secs_f64();
    if now > 45.0 {
        panic!("focus capture timed out at stage {}", flow.stage);
    }
    if flow.stage == 0 {
        if *state.get() == GameState::Menu && now > 3.0 {
            queue.0.push(UiAction::Practice);
            flow.stage = 1;
        }
        return;
    }
    let Some(v) = &view.0 else { return };
    if v.phase != tbg_core::Phase::Running {
        return;
    }
    let directory = PathBuf::from(crate::config::url_param("capture_focus").unwrap_or_default());
    if flow.stage == 1 {
        let Some(u) = v.units.iter().find(|u| u.seat == v.you && !u.is_summon()) else {
            return;
        };
        flow.id = Some(u.id);
        queue.0.push(UiAction::SelectUnit(u.id));
        flow.stage = 2;
        flow.since = now;
        return;
    }
    if now - flow.since < 0.8 {
        return;
    }
    let id = flow.id.unwrap_or_default();
    let tier = v.units.iter().find(|u| u.id == id).map(|u| u.tier);
    let capture = |commands: &mut Commands, name: &str| {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(directory.join(name)));
    };
    match flow.stage {
        2 | 3 | 4 => {
            let wanted = flow.stage - 2;
            if tier != Some(wanted) {
                return;
            }
            assert_eq!(selected.0, Some(id), "purchase must retain Evolution focus");
            assert_eq!(inspected.0, Some(id), "information card keeps its subject");
            capture(&mut commands, &format!("unit-tier-{wanted}.png"));
            if wanted < 2 {
                assert!(v.energy >= v.evolve_costs[usize::from(wanted)]);
                queue.0.push(UiAction::Evolve {
                    unit: id,
                    branch: 0,
                });
            } else {
                queue.0.push(UiAction::SelectTower);
            }
            eprintln!(
                "Focus capture: stage={} unit={id} tier={wanted} focus retained",
                flow.stage
            );
        }
        5 => {
            assert!(selected.0.is_none() && inspected.0.is_none());
            capture(&mut commands, "tower-choices.png");
            queue.0.push(UiAction::ShowArenaBanner);
        }
        6 => {
            capture(&mut commands, "arena-open.png");
            queue.0.push(UiAction::Deselect);
        }
        7 => {
            capture(&mut commands, "arena-dismissed.png");
        }
        _ => {
            eprintln!("Focus capture: two purchases and outside dismissal completed");
            exit.write(AppExit::Success);
        }
    }
    flow.stage += 1;
    flow.since = now;
}
