//! Opt-in local smoke scenario for real Shop reservations through normal UiActions.

use crate::{
    sync::LatestView,
    ui::{ActionQueue, PendingBuy, UiAction},
    GameState,
};
use bevy::prelude::*;
use tbg_core::{content::def, MatchView, Phase, UnitKind};

#[derive(Resource, Default)]
pub(super) struct Status {
    pub done: bool,
    phase: u8,
    expected: Option<UnitKind>,
    charges: u8,
    failed: bool,
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Status>()
        .add_systems(Update, step.run_if(in_state(GameState::InMatch)));
}

fn step(
    view: Res<LatestView>,
    pending: Res<PendingBuy>,
    mut status: ResMut<Status>,
    mut actions: ResMut<ActionQueue>,
) {
    if status.done || status.failed || pending.0.is_some() {
        return;
    }
    let Some(v) = &view.0 else { return };
    if v.phase != Phase::Running || v.augment_round.is_some() {
        return;
    }
    match status.phase {
        0 => {
            status.expected = Some(v.offers[0]);
            status.charges = 3;
            actions.0.push(UiAction::Reserve {
                offer: 0,
                kind: v.offers[0],
            });
            status.phase = 1;
        }
        1 => verify_renewal(v, &mut status, &mut actions),
        2 if v.reserve[1] == 3 => {
            actions.0.push(UiAction::Release(1));
            status.phase = 3;
        }
        3 if v.reserve[1] == 0 && v.held_mask & 2 != 0 => {
            actions.0.push(UiAction::Reserve {
                offer: 2,
                kind: v.offers[2],
            });
            status.phase = 4;
        }
        4 if v.reserve[2] == 3 => {
            status.done = true;
            eprintln!("Reserve smoke passed: held slot survived three purchases (3 -> 2 -> 1 -> 0); release acknowledged; another card is locked with 3 charges.");
        }
        _ => {}
    }
}

fn verify_renewal(v: &MatchView, status: &mut Status, actions: &mut ActionQueue) {
    if v.reserve[0] != status.charges {
        return;
    }
    if Some(v.offers[0]) != status.expected {
        status.failed = true;
        eprintln!("Reserve smoke failed: the held card changed before the fourth Shop redraw.");
        return;
    }
    if status.charges == 0 {
        if v.held_mask & 1 == 0 {
            status.failed = true;
            eprintln!("Reserve smoke failed: expired card lost its nonrenewable marker.");
            return;
        }
        actions.0.push(UiAction::Reserve {
            offer: 1,
            kind: v.offers[1],
        });
        status.phase = 2;
        return;
    }
    let offer = (1..v.offers.len())
        .filter(|i| v.energy >= def(v.offers[*i]).cost)
        .min_by_key(|i| def(v.offers[*i]).cost);
    if let Some(offer) = offer {
        actions.0.push(UiAction::Buy(offer as u8));
        status.charges -= 1;
    }
}
