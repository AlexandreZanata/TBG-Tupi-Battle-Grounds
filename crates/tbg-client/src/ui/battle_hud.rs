//! Desktop battle presentation over the canvas; values and actions use the existing match path.

use bevy::prelude::*;

/// Whether the HTML battle layer is showing (desktop landscape).
#[derive(Resource, Default)]
pub(crate) struct DesktopBattle(pub bool);

pub fn plugin(app: &mut App) {
    app.init_resource::<DesktopBattle>()
        .add_systems(Update, hide_fallback);
    #[cfg(target_arch = "wasm32")]
    app.add_systems(PreUpdate, web::update);
}

fn hide_fallback(
    desktop: Res<DesktopBattle>,
    state: Res<State<crate::GameState>>,
    mut hud: Query<&mut Visibility, With<super::hud::HudRoot>>,
    mut stats: Query<
        &mut Visibility,
        (
            With<crate::overlay::StatsText>,
            Without<super::hud::HudRoot>,
        ),
    >,
) {
    for mut visibility in &mut hud {
        *visibility = if desktop.0 {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
    let debug = crate::config::url_param("measure").is_some();
    for mut visibility in &mut stats {
        *visibility = if *state.get() == crate::GameState::Gallery || (desktop.0 && !debug) {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
}

#[cfg(any(target_arch = "wasm32", test))]
fn relative_hp(view: &tbg_core::MatchView) -> [i32; 2] {
    let me = view.you.index();
    [
        view.stronghold_hp[me].max(0),
        view.stronghold_hp[1 - me].max(0),
    ]
}

#[cfg(target_arch = "wasm32")]
mod web {
    use super::*;
    use crate::ui::{surrender::Surrender, ActionQueue, PendingBuy, UiAction};
    use crate::{
        camera,
        sync::{LatestView, MatchInfo},
        GameState,
    };
    use tbg_core::{
        augment::AugmentCounts,
        content::{augment_def, def, ENERGY_SCALE, OFFERS, STRONGHOLD_HP},
        FACTIONS,
    };
    use wasm_bindgen::JsCast;

    fn attr(root: &web_sys::HtmlElement, name: &str, value: &str) {
        if root.get_attribute(name).as_deref() != Some(value) {
            let _ = root.set_attribute(name, value);
        }
    }

    pub(super) fn update(
        window: Single<&Window>,
        state: Res<State<GameState>>,
        view: Res<LatestView>,
        details: (Res<MatchInfo>, Res<PendingBuy>, Res<Surrender>),
        inspected: Res<crate::ui::inspect::Inspected>,
        mut queue: ResMut<ActionQueue>,
        mut desktop: ResMut<DesktopBattle>,
    ) {
        let root = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("tbg-battle"))
            .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok());
        let Some(root) = root else {
            desktop.0 = false;
            return;
        };
        desktop.0 = camera::desktop(window.width(), window.height())
            && *state.get() == GameState::InMatch
            && view.0.is_some()
            && root.get_attribute("data-ready").as_deref() == Some("true");
        root.set_hidden(!desktop.0);
        if let Some(value) = root.get_attribute("data-action") {
            let _ = root.remove_attribute("data-action");
            if desktop.0 {
                if let Some(action) = action(&value) {
                    queue.0.push(action);
                }
            }
        }
        if !desktop.0 {
            return;
        }
        let (info, pending, surrender) = details;
        let Some(v) = &view.0 else {
            return;
        };
        let hp = relative_hp(v);
        attr(&root, "data-hp-own", &hp[0].to_string());
        attr(&root, "data-hp-enemy", &hp[1].to_string());
        attr(&root, "data-hp-max", &STRONGHOLD_HP.to_string());
        attr(&root, "data-you", &info.you_name);
        attr(&root, "data-opponent", &info.opponent);
        attr(
            &root,
            "data-ranked",
            if info.ranked { "true" } else { "false" },
        );
        attr(&root, "data-clock", &super::super::hud_update::clock(v));
        let unit = inspected
            .0
            .and_then(|id| crate::ui::inspect::report(v, id, &info.arcanas))
            .map(|r| r.record())
            .unwrap_or_default();
        attr(&root, "data-inspect", &unit);
        attr(&root, "data-energy", &v.energy.to_string());
        attr(&root, "data-energy-scale", &ENERGY_SCALE.to_string());
        attr(&root, "data-regen", &v.regen_bp.to_string());
        attr(
            &root,
            "data-augments-you",
            &augment_label(&v.augments[v.you.index()]),
        );
        attr(
            &root,
            "data-augments-enemy",
            &augment_label(&v.augments[v.you.other().index()]),
        );
        let (council, options) = match &v.augment_round {
            Some(round) if round.ticks_left > 0 => {
                let ids = round
                    .options
                    .iter()
                    .map(u8::to_string)
                    .collect::<Vec<_>>()
                    .join(",");
                let options = round
                    .options
                    .iter()
                    .enumerate()
                    .filter_map(|(i, id)| {
                        let def = augment_def(*id)?;
                        Some(format!(
                            "{}:{}:{}:{}",
                            def.id,
                            def.name,
                            def.text,
                            round.costs[i] / ENERGY_SCALE
                        ))
                    })
                    .collect::<Vec<_>>()
                    .join("|");
                (
                    format!(
                        "{}|{}|{}|{}",
                        round.round, round.ticks_left, round.resolved, ids
                    ),
                    options,
                )
            }
            _ => ("closed".to_string(), String::new()),
        };
        attr(&root, "data-council", &council);
        attr(&root, "data-council-options", &options);
        attr(
            &root,
            "data-army",
            // `Army N` (BR-U07), plus `| Brood M` while summons live (BR-U08).
            super::super::hud_update::army_label(v).trim_start_matches("Army "),
        );
        let synergies = FACTIONS
            .iter()
            .filter(|f| v.synergy[v.you.index()][f.index()] > 0)
            .map(|f| format!("{f:?} {}/2", v.synergy[v.you.index()][f.index()]))
            .collect::<Vec<_>>()
            .join(" | ");
        attr(&root, "data-synergy", &synergies);
        attr(
            &root,
            "data-surrender",
            if surrender.sent {
                "sent"
            } else if surrender.armed_until.is_some() {
                "armed"
            } else {
                "ready"
            },
        );
        for i in 0..OFFERS {
            let d = def(v.offers[i]);
            let pending = pending.0.is_some_and(|(offer, _)| usize::from(offer) == i);
            let available = super::super::hud_update::buyable(v, i) && !pending;
            let data = format!(
                "{:?}|{}|{:?}|{}|{:?}|{}|{}",
                v.offers[i],
                d.name,
                d.archetype,
                d.cost / ENERGY_SCALE,
                d.faction,
                available,
                pending
            );
            attr(&root, &format!("data-offer-{i}"), &data);
        }
    }

    fn action(value: &str) -> Option<UiAction> {
        if value == "surrender" {
            return Some(UiAction::Surrender);
        }
        if value == "deselect" {
            return Some(UiAction::Deselect);
        }
        if value == "close-inspect" {
            return Some(UiAction::CloseInspect);
        }
        if value == "forgo" {
            return Some(UiAction::ForgoAugment);
        }
        let (kind, index) = value.split_once(':')?;
        let index = index.parse::<u8>().ok()?;
        match kind {
            "buy" => (usize::from(index) < OFFERS).then_some(UiAction::Buy(index)),
            "inspect" => (usize::from(index) < OFFERS).then_some(UiAction::Inspect(index)),
            "aug" => (usize::from(index) < tbg_core::content::AUGMENT_OPTIONS)
                .then_some(UiAction::ChooseAugment(index)),
            _ => None,
        }
    }

    fn augment_label(counts: &AugmentCounts) -> String {
        counts
            .iter()
            .enumerate()
            .filter(|(_, copies)| **copies > 0)
            .filter_map(|(id, copies)| {
                let def = augment_def(id as u8)?;
                Some(if *copies > 1 {
                    format!("{} x{copies}", def.name)
                } else {
                    def.name.to_string()
                })
            })
            .collect::<Vec<_>>()
            .join(" | ")
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn stronghold_numbers_follow_the_view_seat() {
        // BR-M04 and ARCHITECTURE §4: show each Stronghold relative to the view's seat.
        let mut view = tbg_core::MatchState::new(7).view_for(tbg_core::Seat::B, &[]);
        view.stronghold_hp = [3200, 1800];
        assert_eq!(super::relative_hp(&view), [1800, 3200]);
    }
}
