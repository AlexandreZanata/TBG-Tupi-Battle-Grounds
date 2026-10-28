//! Result screen: victory / defeat / draw, why, and how long it took.

use bevy::prelude::*;
use tbg_core::content::TICK_HZ;
use tbg_core::EndReason;

use super::{button, label, screen, UiAction, PANEL};
use crate::sync::MatchInfo;
use crate::GameState;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Result), spawn);
}

fn spawn(mut commands: Commands, info: Res<MatchInfo>) {
    let (title, reason) = match info.outcome {
        Some(o) => {
            let title = match o.winner {
                None => "DRAW",
                Some(w) if Some(w) == info.you => "VICTORY",
                Some(_) => "DEFEAT",
            };
            let reason = match o.reason {
                EndReason::StrongholdDestroyed => "A Stronghold was destroyed",
                EndReason::TimeUp => "Time is up: Stronghold HP decided",
                EndReason::Forfeit => "A player left the match",
            };
            (title, reason)
        }
        None => ("MATCH OVER", ""),
    };
    let secs = info.last_tick / TICK_HZ;
    let duration = format!(
        "Duration {}:{:02} vs {}",
        secs / 60,
        secs % 60,
        info.opponent
    );
    commands.spawn((
        screen(JustifyContent::Center),
        DespawnOnExit(GameState::Result),
        children![(
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(px(24.0)),
                row_gap: px(12.0),
                border_radius: BorderRadius::all(px(16.0)),
                ..default()
            },
            BackgroundColor(PANEL),
            children![
                label(title, 40.0),
                label(reason, 16.0),
                label(duration, 16.0),
                button("Play again", UiAction::PlayAgain, 220.0),
                button("Menu", UiAction::ToMenu, 220.0),
            ],
        )],
    ));
}
