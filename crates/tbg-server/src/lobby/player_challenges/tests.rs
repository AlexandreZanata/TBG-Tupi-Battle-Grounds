//! BR-AP02–07: participant authorization, consent, bounds and activity races.

use super::*;
use crate::chan::ConnEvent;
use crate::lobby::tests::{add_session, lobby};
use crate::lobby::RematchTicket;
use tbg_core::protocol::{ClientMsg, Profile};

fn friends(l: &mut Lobby, t: Token, name: &str, id: i64) -> tokio::sync::mpsc::Receiver<ConnEvent> {
    let rx = add_session(l, t, SessionState::Idle);
    let profile = Profile {
        name: name.into(),
        rating: 1000,
        ranked_games: 0,
        wins: 0,
        losses: 0,
        draws: 0,
    };
    let s = l.sessions.get_mut(&t).unwrap();
    s.account = Some((id, profile.clone()));
    s.name = name.into();
    s.auth_binding = Some(crate::auth::Binding::fixture(id, profile));
    s.player_challenges_enabled = true;
    rx
}
fn send(l: &mut Lobby, t: Token, name: &str) {
    l.direct_action(t, ClientMsg::ChallengePlayer { name: name.into() });
}

#[tokio::test]
async fn explicit_recipient_consent_consumes_once_even_after_crossed_and_replayed_requests() {
    let (mut l, _rx) = lobby();
    let a = [1; 16];
    let b = [2; 16];
    let c = [3; 16];
    let _a = friends(&mut l, a, "Alpha", 1);
    let _b = friends(&mut l, b, "Bravo", 2);
    let _c = friends(&mut l, c, "Third", 3);
    send(&mut l, a, "BRAVO");
    let id = l.player_challenges[0].id;
    send(&mut l, a, "Bravo");
    send(&mut l, b, "Alpha");
    assert_eq!(l.player_challenges.len(), 1);
    l.direct_action(c, ClientMsg::AnswerPlayerChallenge { id, accept: true });
    l.direct_action(c, ClientMsg::CancelPlayerChallenge { id });
    assert_eq!(l.player_challenges.len(), 1);
    assert!(matches!(l.sessions[&a].state, SessionState::Idle));
    l.direct_action(b, ClientMsg::AnswerPlayerChallenge { id, accept: true });
    assert!(matches!(l.sessions[&a].state, SessionState::InMatch { .. }));
    assert!(matches!(l.sessions[&b].state, SessionState::InMatch { .. }));
    assert!(l.live.is_empty(), "direct matches remain private");
    assert_eq!(l.rematches.len(), 1);
    l.direct_action(b, ClientMsg::AnswerPlayerChallenge { id, accept: true });
    assert_eq!(l.rematches.len(), 1);
}

#[test]
fn busy_targets_disconnect_auth_expiry_and_opt_out_never_evict_an_activity() {
    let (mut l, _rx) = lobby();
    let a = [1; 16];
    let b = [2; 16];
    let _a = friends(&mut l, a, "Alpha", 1);
    let _b = friends(&mut l, b, "Bravo", 2);
    l.sessions.get_mut(&b).unwrap().state = SessionState::InRoom;
    send(&mut l, a, "Bravo");
    assert!(l.player_challenges.is_empty());
    assert!(matches!(l.sessions[&b].state, SessionState::InRoom));
    l.sessions.get_mut(&b).unwrap().state = SessionState::Idle;
    send(&mut l, a, "Bravo");
    let id = l.player_challenges[0].id;
    l.sessions.get_mut(&a).unwrap().conn = None;
    l.direct_action(b, ClientMsg::AnswerPlayerChallenge { id, accept: true });
    assert_eq!(
        l.player_challenges.len(),
        1,
        "temporary detach preserves the original offer"
    );
    assert!(matches!(l.sessions[&b].state, SessionState::Idle));
    l.cancel_direct(a);
    l.player_challenge_cooldowns.clear();
    l.sessions.get_mut(&a).unwrap().conn = Some(crate::chan::ConnHandle::new(
        7,
        tokio::sync::mpsc::channel(32).0,
    ));
    send(&mut l, a, "Bravo");
    let id = l.player_challenges[0].id;
    l.sessions
        .get_mut(&a)
        .unwrap()
        .auth_binding
        .as_mut()
        .unwrap()
        .expires_at = 0;
    l.direct_action(b, ClientMsg::AnswerPlayerChallenge { id, accept: true });
    assert!(l.player_challenges.is_empty());
    assert!(matches!(l.sessions[&b].state, SessionState::Idle));
}

#[test]
fn expiry_decline_cancel_and_preference_are_terminal_and_bounded() {
    let (mut l, _rx) = lobby();
    let a = [1; 16];
    let b = [2; 16];
    let _a = friends(&mut l, a, "Alpha", 1);
    let _b = friends(&mut l, b, "Bravo", 2);
    send(&mut l, a, "Bravo");
    l.player_challenges[0].deadline = Instant::now() - Duration::from_secs(1);
    l.sweep_direct();
    assert!(l.player_challenges.is_empty());
    l.player_challenge_cooldowns.clear();
    send(&mut l, a, "Bravo");
    let id = l.player_challenges[0].id;
    l.direct_action(b, ClientMsg::AnswerPlayerChallenge { id, accept: false });
    assert!(l.player_challenges.is_empty());
    l.player_challenge_cooldowns.clear();
    send(&mut l, a, "Bravo");
    let id = l.player_challenges[0].id;
    l.direct_action(a, ClientMsg::CancelPlayerChallenge { id });
    assert!(l.player_challenges.is_empty());
    l.player_challenge_cooldowns.clear();
    send(&mut l, a, "Bravo");
    l.direct_action(b, ClientMsg::SetPlayerChallengesEnabled { enabled: false });
    assert!(l.player_challenges.is_empty());
    assert!(!l.direct_ready(b));
    for _ in 0..1000 {
        send(&mut l, a, "Invented");
    }
    assert!(l.player_challenges.len() <= 512 && l.player_challenge_cooldowns.len() <= 512);
    assert!(l.sessions[&a].player_challenge_actions.len() <= 30);
}

#[test]
fn detach_retains_only_the_original_deadline_and_new_activity_cancels() {
    let (mut l, _rx) = lobby();
    let a = [1; 16];
    let b = [2; 16];
    let _a = friends(&mut l, a, "Alpha", 1);
    let _b = friends(&mut l, b, "Bravo", 2);
    send(&mut l, a, "Bravo");
    let deadline = l.player_challenges[0].deadline;
    l.disconnected(a, 1);
    assert_eq!(l.player_challenges[0].deadline, deadline);
    l.leave_waiting(b);
    assert!(l.player_challenges.is_empty());
}

#[test]
fn account_rate_survives_controller_replacement_and_opt_out_survives_rate_refusal() {
    let (mut l, _rx) = lobby();
    let a = [1; 16];
    let b = [2; 16];
    let new = [3; 16];
    let _a = friends(&mut l, a, "Alpha", 1);
    let _b = friends(&mut l, b, "Bravo", 2);
    send(&mut l, a, "Bravo");
    for _ in 0..40 {
        l.direct_limit(a);
    }
    l.sessions.remove(&a);
    let _new = friends(&mut l, new, "Alpha", 1);
    l.sessions.get_mut(&new).unwrap().ip = "127.0.0.2".parse().unwrap();
    assert!(
        !l.direct_limit(new),
        "BR-AP06: a new controller does not reset the account budget"
    );
    for _ in 0..40 {
        l.direct_limit(b);
    }
    l.direct_action(b, ClientMsg::SetPlayerChallengesEnabled { enabled: false });
    assert!(!l.sessions[&b].player_challenges_enabled);
    assert!(l.player_challenges.is_empty());
}

#[test]
fn receiving_opt_out_does_not_remove_the_senders_explicit_outgoing_consent() {
    let (mut l, _rx) = lobby();
    let a = [1; 16];
    let b = [2; 16];
    let _a = friends(&mut l, a, "Alpha", 1);
    let _b = friends(&mut l, b, "Bravo", 2);
    l.sessions.get_mut(&a).unwrap().player_challenges_enabled = false;
    send(&mut l, a, "Bravo");
    assert_eq!(l.player_challenges.len(), 1);
    assert!(
        !l.direct_ready(a),
        "the opted-out sender is still closed to incoming invitations"
    );
}

#[tokio::test]
async fn signing_a_guest_into_an_account_withdraws_the_guests_offers() {
    // BR-AP03/07: the recipient agreed to play the guest, not the account.
    let (mut l, _rx) = lobby();
    let guest = [1; 16];
    let b = [2; 16];
    let _g = add_session(&mut l, guest, SessionState::Idle);
    let _b = friends(&mut l, b, "Bravo", 2);
    send(&mut l, guest, "Bravo");
    assert_eq!(l.player_challenges.len(), 1, "precondition: guest offer");
    l.rematches.push(RematchTicket {
        players: [guest, b],
        requested: [false; 2],
        expires: Some(Instant::now() + Duration::from_secs(60)),
    });
    let profile = Profile {
        name: "Xray".into(),
        rating: 1000,
        ranked_games: 0,
        wins: 0,
        losses: 0,
        draws: 0,
    };
    l.auth_claim(guest, 1, crate::auth::Binding::fixture(9, profile), false);
    assert_eq!(
        l.sessions[&guest].account.as_ref().map(|(id, _)| *id),
        Some(9)
    );
    assert!(l.player_challenges.is_empty());
    assert!(!l.rematches.iter().any(|r| r.players.contains(&guest)));
}

#[test]
fn declining_and_cancelling_survive_an_exhausted_create_budget() {
    let (mut l, _rx) = lobby();
    let a = [1; 16];
    let b = [2; 16];
    let _a = friends(&mut l, a, "Alpha", 1);
    let _b = friends(&mut l, b, "Bravo", 2);
    send(&mut l, a, "Bravo");
    let id = l.player_challenges[0].id;
    for _ in 0..40 {
        l.direct_limit(b);
    }
    l.direct_action(b, ClientMsg::AnswerPlayerChallenge { id, accept: false });
    assert!(
        l.player_challenges.is_empty(),
        "decline is never rate limited"
    );
    l.player_challenge_cooldowns.clear();
    send(&mut l, b, "Alpha");
    assert!(l.player_challenges.is_empty(), "creating still is");
    l.sessions
        .get_mut(&b)
        .unwrap()
        .player_challenge_actions
        .clear();
    l.player_challenge_accounts.clear();
    l.limits = Default::default();
    send(&mut l, b, "Alpha");
    let id = l.player_challenges[0].id;
    for _ in 0..40 {
        l.direct_limit(b);
    }
    l.direct_action(b, ClientMsg::CancelPlayerChallenge { id });
    assert!(
        l.player_challenges.is_empty(),
        "cancel is never rate limited"
    );
}

#[test]
fn losing_the_account_to_another_slot_closes_its_rematch_offer() {
    let (mut l, _rx) = lobby();
    let a = [1; 16];
    let b = [2; 16];
    let _a = friends(&mut l, a, "Alpha", 1);
    let _b = friends(&mut l, b, "Bravo", 2);
    l.rematches.push(RematchTicket {
        players: [a, b],
        requested: [false, true],
        expires: Some(Instant::now() + Duration::from_secs(60)),
    });
    l.owner_lost(a, 1);
    assert!(l.sessions[&a].account.is_none());
    assert!(l.rematches.is_empty(), "the opponent agreed to the account");
}
