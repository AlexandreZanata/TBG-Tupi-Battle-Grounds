//! Evolution panel for the selected own unit: branch choices with their
//! effects and costs (BR-V01). Closes after a choice, with ×, or when the
//! unit is gone.

use bevy::prelude::*;
use tbg_core::content::{def, form_of, FormDef, ENERGY_SCALE, EVOLVE_COST};
use tbg_core::{OnHit, UnitId, UnitView};

use super::{button, label, UiAction, PANEL};
use crate::sync::LatestView;
use crate::GameState;

#[derive(Resource, Default)]
pub struct Selected(pub Option<UnitId>);

#[derive(Component)]
struct EvolvePanel;

pub fn plugin(app: &mut App) {
    app.init_resource::<Selected>().add_systems(
        Update,
        (drop_gone, rebuild.run_if(resource_changed::<Selected>))
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
        let cost = EVOLVE_COST[usize::from(u.tier.min(1))];
        let action = if v.energy >= cost {
            UiAction::Evolve { unit: id, branch }
        } else {
            UiAction::Disabled
        };
        children.push(
            commands
                .spawn(button(
                    &format!("{text} ({})", cost / ENERGY_SCALE),
                    action,
                    300.0,
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
