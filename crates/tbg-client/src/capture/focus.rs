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
    row_widths: [f32; 2],
}

pub fn plugin(app: &mut App) {
    app.init_resource::<Flow>().add_systems(Update, run);
}

#[allow(clippy::too_many_arguments)] // one opt-in capture journey reads every resource it drives
fn run(
    mut commands: Commands,
    (time, state, view): (Res<Time<Real>>, Res<State<GameState>>, Res<LatestView>),
    inspected: Res<crate::ui::inspect::Inspected>,
    selected: Res<crate::ui::evolve::Selected>,
    mut queue: ResMut<ActionQueue>,
    choices: Query<(&UiAction, &ComputedNode)>,
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
    let branch = crate::config::url_param("capture_branch")
        .and_then(|value| value.parse::<u8>().ok())
        .unwrap_or(0)
        .min(1);
    let id = flow.id.unwrap_or_default();
    let tier = v.units.iter().find(|u| u.id == id).map(|u| u.tier);
    let capture = |commands: &mut Commands, name: &str| {
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(directory.join(name)));
    };
    let widths: Vec<_> = choices
        .iter()
        .filter_map(|(action, node)| {
            let relevant = if flow.stage <= 4 {
                matches!(action, UiAction::Evolve { unit, .. } if *unit == id)
            } else {
                matches!(action, UiAction::Fortify(_))
            };
            (relevant && node.size.x > 0.).then_some(node.size.x)
        })
        .collect();
    if (2..=7).contains(&flow.stage) {
        let family = usize::from(flow.stage > 4);
        let wanted = if family == 0 {
            flow.stage - 2
        } else {
            flow.stage - 5
        };
        let count = if wanted == 0 {
            2
        } else if wanted == 1 {
            1
        } else {
            0
        };
        // Availability and layout follow the view on their own systems; wait for their frame.
        if widths.len() != count {
            return;
        }
        if wanted == 0 {
            flow.row_widths[family] = widths.iter().sum();
        }
        if wanted == 1 {
            assert!(
                widths[0] >= flow.row_widths[family],
                "the single second-tier choice must fill the former two-card row"
            );
        }
    }
    match flow.stage {
        2..=4 => {
            let wanted = flow.stage - 2;
            if tier != Some(wanted) {
                return;
            }
            assert_eq!(selected.0, Some(id), "purchase must retain Evolution focus");
            assert_eq!(inspected.0, Some(id), "information card keeps its subject");
            capture(&mut commands, &format!("unit-tier-{wanted}.png"));
            if wanted < 2 {
                assert!(v.energy >= v.evolve_costs[usize::from(wanted)]);
                queue.0.push(UiAction::Evolve { unit: id, branch });
            } else {
                queue.0.push(UiAction::SelectTower);
            }
            eprintln!(
                "Focus capture: stage={} unit={id} tier={wanted} focus retained",
                flow.stage
            );
        }
        5..=7 => {
            let wanted = flow.stage - 5;
            let tower = crate::ui::fortify::own(v);
            if tower.tier != wanted {
                return;
            }
            if wanted < 2
                && crate::ui::fortify::offer(v, branch, false) != crate::ui::fortify::Offer::Open
            {
                return;
            }
            assert!(selected.0.is_none() && inspected.0.is_none());
            capture(&mut commands, &format!("tower-tier-{wanted}.png"));
            if wanted < 2 {
                queue.0.push(UiAction::Fortify(branch));
            } else {
                queue.0.push(UiAction::ShowArenaBanner);
            }
            eprintln!("Focus capture: tower tier={wanted} branch={branch}");
        }
        8 => capture(&mut commands, "arena-open.png"),
        9 => queue.0.push(UiAction::Deselect),
        10 => capture(&mut commands, "arena-dismissed.png"),
        _ => {
            eprintln!("Focus capture: champion and tower purchases, retained focus and outside dismissal completed");
            exit.write(AppExit::Success);
        }
    }
    if (2..=7).contains(&flow.stage) {
        eprintln!(
            "Focus capture: stage={} enabled choice widths={widths:?}",
            flow.stage
        );
    }
    flow.stage += 1;
    flow.since = now;
}
