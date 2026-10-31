//! Result-owned cosmetic formation, independent of simulation, combat entities and match metrics.

#[cfg(debug_assertions)]
mod capture;
mod layout;
mod scene;
pub(crate) mod ui;

use crate::{sync::MatchInfo, ui::UiAction, GameState};
use bevy::prelude::*;

pub const PAGE_SIZE: u64 = 64;

#[derive(Resource)]
pub(crate) struct Presentation {
    pub page: u64,
    pub hidden: bool,
    pub reduced: bool,
}
impl Default for Presentation {
    fn default() -> Self {
        Self {
            page: 0,
            hidden: false,
            reduced: crate::config::reduced_motion(),
        }
    }
}

pub fn total(info: &MatchInfo) -> Option<u64> {
    let winner = info.outcome?.winner?;
    if info.spectating || info.you != Some(winner) {
        return None;
    }
    info.celebration.as_ref()?.total(winner)
}

pub fn range(total: u64, page: u64) -> (u64, u64) {
    let page = page.min(total.saturating_sub(1) / PAGE_SIZE);
    let first = page * PAGE_SIZE;
    (first, first.saturating_add(PAGE_SIZE).min(total))
}

pub fn plugin(app: &mut App) {
    app.init_resource::<Presentation>();
    scene::plugin(app);
    ui::plugin(app);
    #[cfg(debug_assertions)]
    capture::plugin(app);
    app.add_systems(Update, keys.run_if(in_state(GameState::Result)));
}

pub(crate) fn control(state: &mut Presentation, info: &MatchInfo, action: UiAction) {
    let Some(total) = total(info) else {
        return;
    };
    match action {
        UiAction::CelebrationPrev => state.page = state.page.saturating_sub(1),
        UiAction::CelebrationNext => {
            state.page = state
                .page
                .saturating_add(1)
                .min(total.saturating_sub(1) / PAGE_SIZE)
        }
        UiAction::CelebrationHide => state.hidden = !state.hidden,
        UiAction::CelebrationMotion => {
            state.reduced = !state.reduced;
            crate::config::save_reduced_motion(state.reduced);
        }
        _ => {}
    }
}

fn keys(
    keys: Res<ButtonInput<KeyCode>>,
    info: Res<MatchInfo>,
    mut actions: ResMut<crate::ui::ActionQueue>,
) {
    for (key, action) in [
        (KeyCode::PageUp, UiAction::CelebrationPrev),
        (KeyCode::PageDown, UiAction::CelebrationNext),
        (KeyCode::KeyH, UiAction::CelebrationHide),
        (KeyCode::KeyM, UiAction::CelebrationMotion),
        (KeyCode::Escape, UiAction::ToMenu),
    ] {
        if keys.just_pressed(key) {
            actions.0.push(action);
        }
    }
    if keys.just_pressed(KeyCode::Enter) && !info.spectating {
        actions.0.push(if info.challenge {
            UiAction::RequestRematch
        } else {
            UiAction::PlayAgain
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn celebration_pages_cover_every_logical_slot_without_eager_expansion() {
        for n in [0u64, 1, 3, 64, 65, 128, 129] {
            let mut next = 0;
            for p in 0..=(n.saturating_sub(1) / PAGE_SIZE) {
                let (a, b) = range(n, p);
                assert_eq!(a, next);
                assert!(b - a <= 64);
                next = b;
            }
            assert_eq!(next, n);
        }
        let (a, b) = range(u64::MAX, u64::MAX);
        assert!(b - a <= 64);
        assert_eq!(b, u64::MAX);
    }
}
