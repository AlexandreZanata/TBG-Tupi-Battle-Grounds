//! Result screen: victory / defeat / draw, why, how long it took, and the
//! P39 rematch offer for a Challenge match.

use bevy::prelude::*;
use tbg_core::content::TICK_HZ;
use tbg_core::EndReason;

use super::account::Account;
use super::{button, label, screen, UiAction, PANEL};
use crate::i18n::{Lang, Msg};
use crate::sync::{MatchInfo, MatchStats, RematchInfo};
use crate::GameState;

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Result), spawn)
        .add_systems(
            Update,
            (count_rating, hide_for_challenge, update_rematch).run_if(in_state(GameState::Result)),
        );
}

#[derive(Component)]
struct RatingLabel;

#[derive(Component)]
struct RematchLabel;

#[derive(Component)]
struct RematchButton;

#[derive(Component)]
struct HideForChallenge;

fn spawn(mut commands: Commands, info: Res<MatchInfo>, stats: Res<MatchStats>, lang: Res<Lang>) {
    // P47: a watcher reads the result as a bystander, not as a seat.
    let (title, reason) = match info.outcome {
        Some(o) if info.spectating => {
            let reason = match o.reason {
                EndReason::StrongholdDestroyed => "A Stronghold was destroyed",
                EndReason::Forfeit => "A player surrendered or left the match",
            };
            ("MATCH OVER", Some(reason))
        }
        Some(o) => {
            let title = match o.winner {
                None => "DRAW",
                Some(w) if Some(w) == info.you => "VICTORY",
                Some(_) => "DEFEAT",
            };
            let reason = match o.reason {
                EndReason::StrongholdDestroyed => "A Stronghold was destroyed",
                // BR-M06/BR-M03: say which side gave up.
                EndReason::Forfeit => match o.winner {
                    Some(w) if Some(w) == info.you => "Your opponent surrendered or disconnected",
                    Some(_) => "You surrendered",
                    None => "A player left the match",
                },
            };
            (title, Some(reason))
        }
        None => ("MATCH OVER", None),
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
    commands
        .spawn((
            screen(JustifyContent::Center),
            DespawnOnExit(GameState::Result),
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(px(24.0)),
                    row_gap: px(12.0),
                    border_radius: BorderRadius::all(px(16.0)),
                    ..default()
                },
                BackgroundColor(PANEL),
            ))
            .with_children(|panel| {
                panel.spawn(label(title, 40.0));
                if let Some(reason) = reason {
                    panel.spawn(label(reason, 16.0));
                    panel.spawn(label(duration, 16.0));
                    panel.spawn(label(summary, 16.0));
                } else if !info.opponent.is_empty() {
                    panel.spawn(label(format!("vs {}", info.opponent), 16.0));
                }
                if info.spectating {
                    panel.spawn(button(
                        lang.t(Msg::SpectateBackToLobby),
                        UiAction::ToMenu,
                        220.0,
                    ));
                    return;
                }
                panel.spawn((
                    label("", 18.0),
                    Node::default(),
                    RatingLabel,
                    HideForChallenge,
                ));
                panel.spawn((label("", 18.0), Node::default(), RematchLabel));
                panel.spawn((
                    button("Request rematch", UiAction::RequestRematch, 220.0),
                    RematchButton,
                ));
                panel.spawn((
                    button("Play again", UiAction::PlayAgain, 220.0),
                    HideForChallenge,
                ));
                panel.spawn(button("Menu", UiAction::ToMenu, 220.0));
            });
        });
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

/// Challenge matches never show the casual Play again / rating controls.
fn hide_for_challenge(info: Res<MatchInfo>, mut nodes: Query<&mut Node, With<HideForChallenge>>) {
    for mut node in &mut nodes {
        node.display = if info.challenge {
            Display::None
        } else {
            Display::Flex
        };
    }
}

/// P39: the rematch button and its state, only for a Challenge result.
#[allow(clippy::type_complexity)]
fn update_rematch(
    info: Res<MatchInfo>,
    rematch: Res<RematchInfo>,
    mut set: ParamSet<(
        Query<(&mut Text, &mut Node), With<RematchLabel>>,
        Query<(&Children, &mut Node), With<RematchButton>>,
        Query<&mut Text>,
    )>,
) {
    let (label, button_text, button_visible) = if !info.challenge {
        ("", "", false)
    } else if !rematch.open {
        let text = match rematch.closed {
            Some(true) => "Opponent declined the rematch",
            Some(false) => "Rematch expired",
            None => "Preparing rematch...",
        };
        (text, "", false)
    } else if rematch.you_requested && rematch.opponent_requested {
        ("Starting rematch...", "", false)
    } else if rematch.you_requested {
        ("Waiting for opponent...", "", false)
    } else if rematch.opponent_requested {
        ("Opponent wants a rematch", "Accept rematch", true)
    } else {
        ("", "Request rematch", true)
    };
    for (mut text, mut node) in &mut set.p0() {
        text.0 = label.into();
        node.display = if info.challenge && !label.is_empty() {
            Display::Flex
        } else {
            Display::None
        };
    }
    let mut children = Vec::new();
    for (kids, mut node) in &mut set.p1() {
        node.display = if button_visible {
            Display::Flex
        } else {
            Display::None
        };
        children.extend(kids.iter());
    }
    for child in children {
        if let Ok(mut text) = set.p2().get_mut(child) {
            text.0 = button_text.into();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rematch_controls_follow_the_offer_without_query_conflicts() {
        // Runs the actual Bevy queries: a B0001 regression fails on update.
        let mut app = App::new();
        app.insert_resource(MatchInfo {
            challenge: true,
            ..default()
        })
        .init_resource::<MatchStats>()
        .init_resource::<RematchInfo>()
        .init_resource::<Account>()
        .init_resource::<crate::i18n::Lang>()
        .insert_resource(Time::<()>::default())
        .add_systems(Startup, spawn)
        .add_systems(Update, (count_rating, hide_for_challenge, update_rematch));
        app.update();
        let mut buttons = app
            .world_mut()
            .query_filtered::<&Node, With<RematchButton>>();
        assert_eq!(buttons.single(app.world()).unwrap().display, Display::None);
        {
            let mut offer = app.world_mut().resource_mut::<RematchInfo>();
            offer.open = true;
            offer.opponent_requested = true;
        }
        app.update();
        assert_eq!(buttons.single(app.world()).unwrap().display, Display::Flex);
        let mut texts = app.world_mut().query::<&Text>();
        assert!(texts.iter(app.world()).any(|t| t.0 == "Accept rematch"));
        app.world_mut().resource_mut::<RematchInfo>().you_requested = true;
        app.update();
        assert_eq!(buttons.single(app.world()).unwrap().display, Display::None);
        assert!(texts
            .iter(app.world())
            .any(|t| t.0 == "Starting rematch..."));
        {
            let mut offer = app.world_mut().resource_mut::<RematchInfo>();
            offer.open = false;
            offer.closed = Some(true);
        }
        app.update();
        assert!(texts
            .iter(app.world())
            .any(|t| t.0 == "Opponent declined the rematch"));
        app.world_mut().resource_mut::<MatchInfo>().challenge = false;
        app.update();
        let mut normal = app
            .world_mut()
            .query_filtered::<&Node, With<HideForChallenge>>();
        assert!(normal.iter(app.world()).all(|n| n.display == Display::Flex));
        let mut labels = app
            .world_mut()
            .query_filtered::<&Node, With<RematchLabel>>();
        assert_eq!(labels.single(app.world()).unwrap().display, Display::None);
    }
}
