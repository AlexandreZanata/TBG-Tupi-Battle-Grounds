//! Result screen: victory / defeat / draw, why, and how long it took.

use bevy::prelude::*;
use tbg_core::content::TICK_HZ;
use tbg_core::EndReason;

use super::account::Account;
use super::{button, label, screen, UiAction, PANEL};
use crate::sync::{MatchInfo, MatchStats};
use crate::GameState;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Result), spawn)
        .add_systems(Update, count_rating.run_if(in_state(GameState::Result)));
}

#[derive(Component)]
struct RatingLabel;

fn spawn(mut commands: Commands, info: Res<MatchInfo>, stats: Res<MatchStats>) {
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
                // BR-M06/BR-M03: say which side gave up.
                EndReason::Forfeit => match o.winner {
                    Some(w) if Some(w) == info.you => "Your opponent surrendered or disconnected",
                    Some(_) => "You surrendered",
                    None => "A player left the match",
                },
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
    let summary = format!(
        "Units deployed {} | Evolutions {} | Damage to enemy Stronghold {}",
        stats.bought, stats.evolutions, stats.damage_dealt
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
                label(summary, 16.0),
                (label("", 18.0), RatingLabel),
                button("Play again", UiAction::PlayAgain, 220.0),
                button("Menu", UiAction::ToMenu, 220.0),
            ],
        )],
    ));
}

/// P12: fill the rating label once `RatingChanged` lands, counting the new
/// value up over 0.6 s. Casual matches (and unranked guests) show nothing.
fn count_rating(
    time: Res<Time>,
    account: Res<Account>,
    mut text: Single<&mut Text, With<RatingLabel>>,
) {
    let Some((before, after, arrived)) = account.rating_change else {
        return;
    };
    let t = ((time.elapsed_secs_f64() - arrived) / 0.6).clamp(0.0, 1.0);
    let shown = (f64::from(before) + (f64::from(after) - f64::from(before)) * t).round() as u16;
    let delta = i32::from(after) - i32::from(before);
    text.0 = format!("Rating {before} -> {shown} ({delta:+})");
}
