//! Evolution panel for the selected own unit: branch choices with their
//! effects and costs (BR-V01). Closes after a choice, with ×, or when the
//! unit is gone.

use bevy::prelude::*;
use tbg_core::content::{def, form_of, FormDef, ENERGY_SCALE};
use tbg_core::{OnHit, Phase, UnitId, UnitView};

use super::{button, label, UiAction, BUTTON, DISABLED, PANEL};
use crate::sync::LatestView;
use crate::GameState;

#[derive(Resource, Default)]
pub struct Selected(pub Option<UnitId>);

#[derive(Component)]
struct EvolvePanel;

#[derive(Component)]
struct EvolutionChoice {
    unit: UnitId,
    branch: u8,
}

pub fn plugin(app: &mut App) {
    app.init_resource::<Selected>().add_systems(
        Update,
        (
            drop_gone,
            rebuild.run_if(resource_changed::<Selected>),
            refresh_choices.run_if(resource_changed::<LatestView>),
        )
            .chain()
            .run_if(in_state(GameState::InMatch)),
    );
}

fn drop_gone(mut selected: ResMut<Selected>, view: Res<LatestView>) {
    let (Some(id), Some(v)) = (selected.0, &view.0) else {
        return;
    };
    if !v.units.iter().any(|u| u.id == id && u.seat == v.you) {
        selected.0 = None;
    }
}

fn rebuild(
    mut commands: Commands,
    selected: Res<Selected>,
    view: Res<LatestView>,
    panels: Query<Entity, With<EvolvePanel>>,
) {
    for e in &panels {
        commands.entity(e).despawn();
    }
    let (Some(id), Some(v)) = (selected.0, &view.0) else {
        return;
    };
    let Some(u) = v.units.iter().find(|u| u.id == id) else {
        return;
    };
    let panel = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: percent(18.0),
                justify_self: JustifySelf::Center,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(px(14.0)),
                row_gap: px(8.0),
                border_radius: BorderRadius::all(px(12.0)),
                ..default()
            },
            BackgroundColor(PANEL),
            EvolvePanel,
            DespawnOnExit(GameState::InMatch),
        ))
        .id();
    let title = format!("{} | tier {}", def(u.kind).name, u.tier);
    let mut children = vec![commands.spawn(label(title, 20.0)).id()];
    for (branch, text) in options(u) {
        // The seat's real price (Augment and arena discounts, D-55).
        let cost = v.evolve_costs[usize::from(u.tier.min(1))];
        children.push(
            commands
                .spawn((
                    button(
                        &format!("{text} ({})", cost / ENERGY_SCALE),
                        choice_action(v, id, branch),
                        300.0,
                    ),
                    EvolutionChoice { unit: id, branch },
                ))
                .id(),
        );
    }
    if u.tier >= 2 {
        children.push(commands.spawn(label("Fully evolved", 16.0)).id());
    }
    children.push(
        commands
            .spawn(button("Close", UiAction::CloseEvolve, 140.0))
            .id(),
    );
    commands.entity(panel).add_children(&children);
}

// Keep the same buttons while Energy regenerates or is spent in the Shop.
fn refresh_choices(
    view: Res<LatestView>,
    mut choices: Query<(&EvolutionChoice, &mut UiAction, &mut BackgroundColor)>,
) {
    let Some(v) = &view.0 else { return };
    for (choice, mut action, mut color) in &mut choices {
        *action = choice_action(v, choice.unit, choice.branch);
        color.0 = if *action == UiAction::Disabled {
            DISABLED
        } else {
            BUTTON
        };
    }
}

fn choice_action(v: &tbg_core::MatchView, unit: UnitId, branch: u8) -> UiAction {
    let available = v.units.iter().any(|u| {
        u.id == unit
            && u.seat == v.you
            && u.tier < 2
            && !u.is_summon()
            && (u.tier == 0 || u.branch == branch)
            && form_of(u.kind, u.tier + 1, branch).is_some()
            && v.energy >= v.evolve_costs[usize::from(u.tier.min(1))]
    }) && v.phase == Phase::Running;
    if available {
        UiAction::Evolve { unit, branch }
    } else {
        UiAction::Disabled
    }
}

/// Branch choices: tier 0 → A and B; tier 1 → next form of its branch.
fn options(u: &UnitView) -> Vec<(u8, String)> {
    let branches: Vec<u8> = match u.tier {
        0 => vec![0, 1],
        1 => vec![u.branch],
        _ => vec![],
    };
    branches
        .into_iter()
        .filter_map(|b| {
            form_of(u.kind, u.tier + 1, b).map(|f| (b, format!("{}: {}", f.name, describe(f))))
        })
        .collect()
}

pub fn describe(f: &FormDef) -> String {
    let mut parts = Vec::new();
    let pct = |v: i32, what: &str| (v != 0).then(|| format!("+{}% {what}", v / 100));
    parts.extend(pct(f.hp_bp, "HP"));
    parts.extend(pct(f.dmg_bp, "dmg"));
    parts.extend(pct(f.atk_bp, "attack speed"));
    parts.extend(pct(f.speed_bp, "speed"));
    parts.extend(pct(f.heal_bp, "heal"));
    let tiles = |v: i32, what: &str| (v != 0).then(|| format!("+{:.1} {what}", v as f32 / 1000.0));
    parts.extend(tiles(f.range_add, "range"));
    parts.extend(tiles(f.splash_add, "splash"));
    if f.dmg_add != 0 {
        parts.push(format!("+{} dmg attack", f.dmg_add));
    }
    match f.on_hit {
        Some(OnHit::Slow { bp, ticks }) => parts.push(format!(
            "hits slow {}% {:.1}s",
            bp / 100,
            f32::from(ticks) / 20.0
        )),
        Some(OnHit::Root { ticks }) => {
            parts.push(format!("hits root {:.1}s", f32::from(ticks) / 20.0))
        }
        None => {}
    }
    parts.join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tbg_core::content::EVOLVE_COST;

    #[test]
    fn open_panel_tracks_energy_without_reopening_or_replacing_buttons() {
        // BR-E02/E03 and BR-V02: regeneration enables Evolution; spending disables it.
        let mut view = tbg_core::MatchState::new(11).view_for(tbg_core::Seat::A, &[]);
        view.phase = Phase::Running;
        view.energy = EVOLVE_COST[0] - 1;
        view.units.push(UnitView {
            id: 7,
            seat: view.you,
            kind: tbg_core::UnitKind::Archer,
            tier: 0,
            branch: 0,
            x: 0,
            hp: 100,
            max_hp: 100,
            status: 0,
            fx: 0,
        });
        let mut app = App::new();
        app.insert_resource(LatestView(Some(view)))
            .insert_resource(Selected(Some(7)))
            .add_systems(
                Update,
                (
                    drop_gone,
                    rebuild.run_if(resource_changed::<Selected>),
                    refresh_choices.run_if(resource_changed::<LatestView>),
                )
                    .chain(),
            );
        app.update();
        let mut choices = app.world_mut().query::<(Entity, &EvolutionChoice)>();
        let buttons: Vec<_> = choices
            .iter(app.world())
            .map(|(e, c)| (e, c.branch))
            .collect();
        assert_eq!(buttons.len(), 2);
        for energy in [EVOLVE_COST[0] - 1, EVOLVE_COST[0], EVOLVE_COST[0] - 1] {
            app.world_mut()
                .resource_mut::<LatestView>()
                .0
                .as_mut()
                .unwrap()
                .energy = energy;
            app.update();
            for &(entity, branch) in &buttons {
                let affordable = energy >= EVOLVE_COST[0];
                assert_eq!(
                    app.world().get::<UiAction>(entity),
                    Some(&if affordable {
                        UiAction::Evolve { unit: 7, branch }
                    } else {
                        UiAction::Disabled
                    })
                );
                assert_eq!(
                    app.world().get::<BackgroundColor>(entity),
                    Some(&BackgroundColor(if affordable { BUTTON } else { DISABLED }))
                );
            }
        }
    }
}
