//! Explicit local P73 presentation fixtures; backend behavior is verified by real-client tests.

use crate::{
    sync::{MatchInfo, RematchInfo},
    ui::{
        history::History,
        shell::{Panel, Shell},
        social::Social,
    },
    GameState,
};
use bevy::prelude::*;
use tbg_core::protocol::{HistorySaveState, PersonalMatchSummary};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, fixture);
}
fn fixture(
    time: Res<Time>,
    mut done: Local<bool>,
    mut shell: ResMut<Shell>,
    mut history: ResMut<History>,
    mut social: ResMut<Social>,
    mut rematch: ResMut<RematchInfo>,
    mut info: ResMut<MatchInfo>,
    mut next: ResMut<NextState<GameState>>,
) {
    if *done || time.elapsed_secs() < 3. {
        return;
    }
    let Some(kind) = crate::config::url_param("capture_panel") else {
        return;
    };
    *done = true;
    eprintln!("P73 local visual fixture: {kind}; synthetic presentation only, not production accounts or persistence evidence.");
    match kind.as_str() {
        "p73-invites" => {
            social.visual_fixture(time.elapsed_secs_f64());
            shell.panel = Panel::Live;
        }
        "p73-rematch" => {
            shell.panel = Panel::Live;
            *rematch = RematchInfo {
                open: true,
                opponent: "FixtureRival".into(),
                opponent_requested: true,
                ..default()
            };
        }
        "p73-history-private" | "p73-history-public" | "p73-history-detail" => {
            history.visual_fixture(kind == "p73-history-public", kind == "p73-history-detail");
            shell.panel = Panel::History;
        }
        "p73-result" => {
            let summary: PersonalMatchSummary = serde_json::from_str(include_str!(
                "../../tests/fixtures/p73-personal-summary.json"
            ))
            .expect("checked local presentation fixture");
            *info = MatchInfo {
                result_id: Some([73; 16]),
                you: Some(tbg_core::Seat::B),
                you_name: "FixtureOwner".into(),
                opponent: "FixtureRival".into(),
                outcome: Some(tbg_core::Outcome {
                    winner: Some(tbg_core::Seat::A),
                    reason: tbg_core::EndReason::Forfeit,
                }),
                personal: Some(Box::new(summary)),
                history_save: Some(HistorySaveState::Stored),
                ranked: true,
                last_tick: 4000,
                ..default()
            };
            *rematch = RematchInfo {
                open: true,
                opponent: "FixtureRival".into(),
                opponent_requested: true,
                ..default()
            };
            next.set(GameState::Result);
        }
        _ => {}
    }
    shell.dirty = true;
}
