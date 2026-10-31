//! Evolution panel for the selected own unit: branch choices with their
//! effects and costs (BR-V01). Closes after a choice, with ×, or when the
//! unit is gone.

use bevy::prelude::*;
use tbg_core::content::{form_of, FormDef, ENERGY_SCALE, TICK_HZ};
use tbg_core::{OnHit, Phase, UnitId, UnitView};

use super::{button, label, UiAction, BUTTON, DISABLED, PANEL};
use crate::i18n::{content, Lang, Msg};
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
    lang: Res<Lang>,
    panels: Query<Entity, With<EvolvePanel>>,
) {
    let lang = *lang;
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
    let title = lang.tf(
        Msg::EvolveTitle,
        &[
            ("name", &content::unit_name(lang, u.kind)),
            ("tier", &u.tier),
        ],
    );
    let mut children = vec![commands.spawn(label(title, 20.0)).id()];
    for (branch, form, effects) in options(u, lang) {
        // The seat's real price (Augment and arena discounts, D-55).
        let cost = v.evolve_costs[usize::from(u.tier.min(1))];
        children.push(
            commands
                .spawn((
                    button(
                        &lang.tf(
                            Msg::EvolveOption,
                            &[
                                ("form", &form),
                                ("effects", &effects),
                                ("cost", &(cost / ENERGY_SCALE)),
                            ],
                        ),
                        choice_action(v, id, branch),
                        300.0,
                    ),
                    EvolutionChoice { unit: id, branch },
                ))
                .id(),
        );
    }
    if u.tier >= 2 {
        children.push(commands.spawn(label(lang.t(Msg::EvolveFully), 16.0)).id());
    }
    children.push(
        commands
            .spawn(button(
                lang.t(Msg::CommonClose),
                UiAction::CloseEvolve,
                140.0,
            ))
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
fn options(u: &UnitView, lang: Lang) -> Vec<(u8, &'static str, String)> {
    let branches: Vec<u8> = match u.tier {
        0 => vec![0, 1],
        1 => vec![u.branch],
        _ => vec![],
    };
    branches
        .into_iter()
        .filter_map(|b| {
            let form = form_of(u.kind, u.tier + 1, b)?;
            let name = content::form_name(lang, u.kind, u.tier + 1, b)?;
            Some((b, name, describe(form, lang)))
        })
        .collect()
}

/// The form's modifiers as one localized line (`+50% HP, hits slow 30% 1.5s`).
pub fn describe(f: &FormDef, lang: Lang) -> String {
    let mut parts = Vec::new();
    let pct = |v: i32, msg: Msg| (v != 0).then(|| lang.tf(msg, &[("value", &(v / 100))]));
    parts.extend(pct(f.hp_bp, Msg::EvolveEffectHp));
    parts.extend(pct(f.dmg_bp, Msg::EvolveEffectDmg));
    parts.extend(pct(f.atk_bp, Msg::EvolveEffectAtk));
    parts.extend(pct(f.speed_bp, Msg::EvolveEffectSpeed));
    parts.extend(pct(f.heal_bp, Msg::EvolveEffectHeal));
    let tiles = |v: i32, msg: Msg| {
        (v != 0).then(|| lang.tf(msg, &[("value", &lang.decimal(f64::from(v) / 1000.0, 1))]))
    };
    parts.extend(tiles(f.range_add, Msg::EvolveEffectRange));
    parts.extend(tiles(f.splash_add, Msg::EvolveEffectSplash));
    if f.dmg_add != 0 {
        parts.push(lang.tf(Msg::EvolveEffectDmgAdd, &[("value", &f.dmg_add)]));
    }
    let secs = |ticks: u16| lang.decimal(f64::from(ticks) / f64::from(TICK_HZ), 1);
    match f.on_hit {
        Some(OnHit::Slow { bp, ticks }) => parts.push(lang.tf(
            Msg::EvolveEffectSlow,
            &[("pct", &(bp / 100)), ("secs", &secs(ticks))],
        )),
        Some(OnHit::Root { ticks }) => {
            parts.push(lang.tf(Msg::EvolveEffectRoot, &[("secs", &secs(ticks))]))
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
    fn describe_keeps_core_numbers_in_both_languages() {
        // P64 T02: Fortress (Shieldbearer A2) = +50% HP and a 30% slow for 1.5 s.
        let fortress = form_of(tbg_core::UnitKind::Shieldbearer, 2, 0).unwrap();
        assert_eq!(describe(fortress, Lang::En), "+50% HP, hits slow 30% 1.5s");
        assert_eq!(
            describe(fortress, Lang::PtBr),
            "+50% PV, golpes causam lentid\u{E3}o de 30% por 1,5s"
        );
    }

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
            .init_resource::<Lang>()
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
