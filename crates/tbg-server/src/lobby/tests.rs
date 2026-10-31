//! P39 lobby lifecycle tests: expired invites, bounded cleanup, and rematch
//! notification without starting a match task.

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::{mpsc, watch};

use tbg_core::protocol::{
    ClientMsg, LiveId, LivePlayer, MatchMode, Profile, ServerMsg, SpectateRefusal,
};
use tbg_core::{Arena, StrongholdKind};

use super::live::{Listing, LiveMatch};
use super::{Lobby, RematchTicket, Session, SessionState};
use crate::chan::{ConnEvent, ConnHandle, LobbyCmd, MatchCmd};
use crate::limits::Limits;
use crate::live::{LiveProgress, LiveSnapshot};
use crate::metrics::Metrics;
use crate::Config;

fn lobby() -> (Lobby, mpsc::Receiver<LobbyCmd>) {
    let (self_tx, self_rx) = mpsc::channel(8);
    (
        Lobby {
            sessions: HashMap::new(),
            queue: VecDeque::new(),
            ranked_queue: VecDeque::new(),
            queued_at: HashMap::new(),
            rooms: Vec::new(),
            challenges: Vec::new(),
            rematches: Vec::new(),
            self_tx,
            config: Arc::new(Config::default()),
            metrics: Arc::new(Metrics::default()),
            store: None,
            limits: Limits::default(),
            draining: Arc::new(AtomicBool::new(false)),
            live: HashMap::new(),
            live_tx: watch::channel(Arc::default()).0,
            featured: None,
            ranked: None,
            ranked_seen: HashMap::new(),
        },
        self_rx,
    )
}

fn add_session(
    lobby: &mut Lobby,
    token: [u8; 16],
    state: SessionState,
) -> mpsc::Receiver<ConnEvent> {
    let (tx, rx) = mpsc::channel(8);
    lobby.sessions.insert(
        token,
        Session {
            token,
            name: "Tester".into(),
            conn: Some(ConnHandle { id: 1, tx }),
            state,
            account: None,
            ip: "127.0.0.1".parse().unwrap(),
            detached_at: None,
            arcanas: tbg_core::ArcanaPreset::DEFAULT,
            stronghold: tbg_core::StrongholdKind::DEFAULT,
            last_spectate: None,
        },
    );
    rx
}

fn expect(rx: &mut mpsc::Receiver<ConnEvent>, msg: ServerMsg) {
    match rx.try_recv() {
        Ok(ConnEvent::Send(got)) => assert_eq!(got, msg),
        Ok(ConnEvent::EnterMatch { .. }) => panic!("expected {msg:?}, got EnterMatch"),
        Ok(ConnEvent::LeaveMatch) => panic!("expected {msg:?}, got LeaveMatch"),
        Ok(ConnEvent::Close) => panic!("expected {msg:?}, got Close"),
        Ok(ConnEvent::Frame(_)) => panic!("expected {msg:?}, got Frame"),
        Err(error) => panic!("expected {msg:?}, got channel error {error:?}"),
    }
}

#[test]
fn cleanup_removes_expired_challenge_and_resets_owner() {
    let (mut lobby, _self_rx) = lobby();
    let invite = [7; 16];
    let owner = [1; 16];
    let mut owner_rx = add_session(
        &mut lobby,
        owner,
        SessionState::InChallenge { token: invite },
    );
    lobby
        .challenges
        .push((invite, owner, Instant::now() - Duration::from_secs(601)));

    lobby.cleanup();

    assert!(lobby.challenges.is_empty());
    assert!(matches!(
        lobby.sessions.get(&owner).map(|s| &s.state),
        Some(SessionState::Idle)
    ));
    expect(&mut owner_rx, ServerMsg::ChallengeUnavailable);
}

#[test]
fn cleanup_removes_many_expired_challenges_in_one_sweep() {
    let (mut lobby, _self_rx) = lobby();
    let owner = [1; 16];
    let _owner_rx = add_session(
        &mut lobby,
        owner,
        SessionState::InChallenge { token: [0; 16] },
    );
    for i in 0..100u8 {
        lobby
            .challenges
            .push(([i; 16], owner, Instant::now() - Duration::from_secs(601)));
    }

    lobby.cleanup();

    assert!(lobby.challenges.is_empty(), "expired invites must not grow");
}

#[test]
fn cleanup_expired_rematch_notifies_both_players() {
    let (mut lobby, _self_rx) = lobby();
    let a = [1; 16];
    let b = [2; 16];
    let mut a_rx = add_session(&mut lobby, a, SessionState::Idle);
    let mut b_rx = add_session(&mut lobby, b, SessionState::Idle);
    lobby.rematches.push(RematchTicket {
        players: [a, b],
        requested: [true, false],
        expires: Some(Instant::now() - Duration::from_secs(1)),
    });

    lobby.cleanup();

    assert!(lobby.rematches.is_empty());
    expect(&mut a_rx, ServerMsg::RematchClosed { declined: false });
    expect(&mut b_rx, ServerMsg::RematchClosed { declined: false });
}

#[test]
fn cleanup_rematch_with_missing_session_notifies_remaining_player() {
    let (mut lobby, _self_rx) = lobby();
    let a = [1; 16];
    let missing = [2; 16];
    let mut a_rx = add_session(&mut lobby, a, SessionState::Idle);
    lobby.rematches.push(RematchTicket {
        players: [a, missing],
        requested: [false, true],
        expires: Some(Instant::now() + Duration::from_secs(60)),
    });

    lobby.cleanup();

    assert!(lobby.rematches.is_empty());
    expect(&mut a_rx, ServerMsg::RematchClosed { declined: false });
}

#[test]
fn login_takeover_moves_the_rematch_token() {
    let (mut lobby, _self_rx) = lobby();
    let old = [1; 16];
    let new = [3; 16];
    let other = [2; 16];
    lobby.rematches.push(RematchTicket {
        players: [old, other],
        requested: [false, false],
        expires: Some(Instant::now() + Duration::from_secs(60)),
    });

    lobby.replace_rematch_player(old, new);

    assert_eq!(lobby.rematches[0].players, [new, other]);
}

#[test]
fn resume_cannot_start_an_expired_rematch_before_cleanup() {
    // BR-R09: both requests must still be inside the offer's lifetime.
    let (mut lobby, _self_rx) = lobby();
    let a = [1; 16];
    let b = [2; 16];
    let _a_rx = add_session(&mut lobby, a, SessionState::Idle);
    let mut b_rx = add_session(&mut lobby, b, SessionState::Idle);
    lobby.disconnected(a, 1);
    lobby.rematches.push(RematchTicket {
        players: [a, b],
        requested: [true, true],
        expires: Some(Instant::now() - Duration::from_secs(1)),
    });
    let (tx, mut a_rx) = mpsc::channel(8);
    lobby.hello(
        Some(a),
        ConnHandle { id: 2, tx },
        "127.0.0.1".parse().unwrap(),
    );
    assert!(lobby.rematches.is_empty());
    expect(&mut a_rx, ServerMsg::RematchClosed { declined: false });
    expect(&mut b_rx, ServerMsg::RematchClosed { declined: false });
    assert!(matches!(lobby.sessions[&a].state, SessionState::Idle));
}

#[test]
fn reconnect_during_drain_keeps_rematch_from_starting() {
    let (mut lobby, _self_rx) = lobby();
    let a = [1; 16];
    let b = [2; 16];
    let _a_rx = add_session(&mut lobby, a, SessionState::Idle);
    let _b_rx = add_session(&mut lobby, b, SessionState::Idle);
    lobby.rematches.push(RematchTicket {
        players: [a, b],
        requested: [true, true],
        expires: Some(Instant::now() + Duration::from_secs(60)),
    });
    lobby
        .draining
        .store(true, std::sync::atomic::Ordering::Relaxed);
    lobby.try_start_rematch([a, b]);
    assert_eq!(lobby.rematches.len(), 1);
    assert!(matches!(lobby.sessions[&a].state, SessionState::Idle));
}

#[test]
fn delayed_account_preset_cannot_change_a_waiting_challenge() {
    // BR-AR05: even a database reply must respect the frozen selection.
    let (mut lobby, _self_rx) = lobby();
    let token = [1; 16];
    let mut rx = add_session(
        &mut lobby,
        token,
        SessionState::InChallenge { token: [7; 16] },
    );
    let profile = tbg_core::protocol::Profile {
        name: "Tester".into(),
        rating: 1000,
        ranked_games: 0,
        wins: 0,
        losses: 0,
        draws: 0,
    };
    lobby.sessions.get_mut(&token).unwrap().account = Some((1, profile));
    let preset =
        tbg_core::ArcanaPreset::complete([2, 2, 0, 0, 2, 2, 0, 0, 0, 0, 0, 0, 1, 1, 1]).unwrap();
    lobby.arcanas_loaded(token, 1, preset);
    assert_eq!(
        lobby.sessions[&token].arcanas,
        tbg_core::ArcanaPreset::DEFAULT
    );
    assert!(rx.try_recv().is_err());
}

// P47 (BR-LV01, BR-LV06, BR-LV07): the live registry and snapshot.

fn fake_live(players: [(&str, Option<u16>); 2], started: Instant) -> LiveMatch {
    LiveMatch {
        tx: mpsc::channel::<MatchCmd>(1).0,
        listing: Listing::Public,
        mode: MatchMode::Casual,
        arena: Arena::Forest,
        players: players.map(|(name, rating)| LivePlayer {
            name: name.into(),
            rating,
        }),
        kinds: [StrongholdKind::Classic; 2],
        started,
        progress: LiveProgress::default(),
        watchers: Vec::new(),
    }
}

fn listed_ids(snapshot: &LiveSnapshot) -> Vec<LiveId> {
    snapshot.matches.iter().map(|row| row.id).collect()
}

fn profile(name: &str, rating: u16) -> Profile {
    Profile {
        name: name.into(),
        rating,
        ranked_games: 1,
        wins: 0,
        losses: 0,
        draws: 0,
    }
}

fn room_code(rx: &mut mpsc::Receiver<ConnEvent>) -> tbg_core::protocol::RoomCode {
    match rx.try_recv() {
        Ok(ConnEvent::Send(ServerMsg::RoomCreated { code })) => code,
        _ => panic!("expected RoomCreated"),
    }
}

#[test]
fn featured_is_the_highest_mean_and_holds_thirty_seconds() {
    let (mut lobby, _self_rx) = lobby();
    let rx = lobby.live_tx.subscribe();
    let t0 = Instant::now();
    lobby.register_live(1, fake_live([("Ana", Some(1500)), ("Bia", Some(1400))], t0));
    lobby.register_live(2, fake_live([("Dan", Some(1200)), ("Eva", Some(1200))], t0));
    lobby.register_live(3, fake_live([("Guest", None), ("Guest", None)], t0));
    lobby.publish_live(t0);
    let snap = rx.borrow().clone();
    assert_eq!(snap.featured, Some(1));
    assert_eq!(
        listed_ids(&snap),
        vec![1, 2, 3],
        "featured first, then mean"
    );
    // A higher mean arrives, but the featured battle keeps its spot for 30 s.
    let later = t0 + Duration::from_secs(5);
    lobby.register_live(
        4,
        fake_live([("Fox", Some(2000)), ("Gil", Some(2000))], later),
    );
    lobby.publish_live(t0 + Duration::from_secs(10));
    assert_eq!(rx.borrow().featured, Some(1));
    lobby.publish_live(t0 + Duration::from_secs(31));
    assert_eq!(rx.borrow().featured, Some(4));
    // When the featured battle ends, the next best takes the spot at once.
    lobby.live_ended(4);
    lobby.publish_live(t0 + Duration::from_secs(32));
    assert_eq!(rx.borrow().featured, Some(1));
}

#[test]
fn a_progress_report_reaches_the_listed_row() {
    let (mut lobby, _self_rx) = lobby();
    let rx = lobby.live_tx.subscribe();
    let t0 = Instant::now();
    lobby.register_live(7, fake_live([("Ana", Some(1000)), ("Bia", None)], t0));
    let progress = LiveProgress {
        hp_bp: [6_000, 2_500],
        tiers: [2, 0],
        elapsed_s: 42,
        spectators: 0,
    };
    lobby.live_progress(7, progress);
    lobby.publish_live(t0);
    let row = &rx.borrow().matches[0];
    assert_eq!(row.hp_bp, [6_000, 2_500]);
    assert_eq!(row.players[0].tier, 2);
    assert_eq!(row.players[1].rating, None, "a guest has no rating");
}

#[tokio::test]
async fn public_queues_are_listed_and_rooms_and_challenges_never_are() {
    let (mut lobby, _self_rx) = lobby();
    let rx = lobby.live_tx.subscribe();
    add_session(&mut lobby, [1; 16], SessionState::Idle);
    add_session(&mut lobby, [2; 16], SessionState::Idle);
    lobby.quick([1; 16]);
    lobby.quick([2; 16]);
    assert_eq!(lobby.live.len(), 1, "a casual quick match is listed");
    assert!(lobby.live.values().all(|m| m.listing == Listing::Public));

    let mut c = add_session(&mut lobby, [3; 16], SessionState::Idle);
    add_session(&mut lobby, [4; 16], SessionState::Idle);
    lobby.create_room([3; 16]);
    let code = room_code(&mut c);
    lobby.join_room([4; 16], code);
    assert_eq!(lobby.live.len(), 1, "a private room is never listed");

    let mut e = add_session(&mut lobby, [5; 16], SessionState::Idle);
    add_session(&mut lobby, [6; 16], SessionState::Idle);
    lobby.create_challenge([5; 16]);
    let Ok(ConnEvent::Send(ServerMsg::ChallengeWaiting { token: invite })) = e.try_recv() else {
        panic!("expected ChallengeWaiting");
    };
    lobby.join_challenge([6; 16], invite);
    assert_eq!(lobby.live.len(), 1, "a challenge is never listed");

    lobby.publish_live(Instant::now());
    let snap = rx.borrow().clone();
    assert_eq!(snap.matches.len(), 1);
    // Three running matches (quick, room, challenge) = six seats; only the
    // quick match is listed, but presence counts every running seat (BR-LV06).
    assert_eq!(snap.counts.in_battle, 6);
}

#[tokio::test]
async fn ranked_matches_are_listed_with_ratings_and_their_end_unlists_them() {
    let (mut lobby, _self_rx) = lobby();
    let rx = lobby.live_tx.subscribe();
    for (token, name, rating) in [([7; 16], "Ana", 1_300), ([8; 16], "Bia", 1_100)] {
        let mut s_rx = add_session(&mut lobby, token, SessionState::Idle);
        let s = lobby.sessions.get_mut(&token).unwrap();
        s.account = Some((i64::from(token[0]), profile(name, rating)));
        drop(s_rx.try_recv());
    }
    lobby.ranked([7; 16]);
    lobby.ranked([8; 16]);
    let id = *lobby.live.keys().next().expect("a ranked match is listed");
    lobby.publish_live(Instant::now());
    let snap = rx.borrow().clone();
    assert_eq!(snap.featured, Some(id));
    assert_eq!(snap.matches[0].mode, MatchMode::Ranked);
    assert_eq!(snap.matches[0].players[0].rating, Some(1_300));
    lobby.handle(LobbyCmd::MatchEnded {
        tokens: [[7; 16], [8; 16]],
        live: Some(id),
    });
    lobby.publish_live(Instant::now());
    assert!(
        rx.borrow().matches.is_empty(),
        "the end removes the listing"
    );
    assert_eq!(rx.borrow().featured, None);
}

#[test]
fn presence_counts_online_sessions_and_accounts_once() {
    let (mut lobby, _self_rx) = lobby();
    let rx = lobby.live_tx.subscribe();
    add_session(&mut lobby, [1; 16], SessionState::Idle);
    add_session(&mut lobby, [2; 16], SessionState::Idle);
    add_session(&mut lobby, [3; 16], SessionState::Idle);
    add_session(&mut lobby, [4; 16], SessionState::Idle);
    let (in_match, _match_rx) = mpsc::channel(1);
    let mut playing = add_session(&mut lobby, [5; 16], SessionState::Idle);
    drop(playing.try_recv());
    lobby.sessions.get_mut(&[5; 16]).unwrap().state = SessionState::InMatch {
        tx: in_match,
        seat: tbg_core::Seat::A,
    };
    for (token, account) in [([1; 16], 1), ([2; 16], 1)] {
        lobby.sessions.get_mut(&token).unwrap().account = Some((account, profile("Ana", 1_200)));
    }
    // A disconnected account is not online.
    lobby.sessions.get_mut(&[4; 16]).unwrap().conn = None;
    lobby.sessions.get_mut(&[4; 16]).unwrap().account = Some((9, profile("Gone", 1_000)));
    lobby.publish_live(Instant::now());
    let counts = rx.borrow().counts;
    assert_eq!(counts.players, 4, "sessions with a connection: 1, 2, 3, 5");
    assert_eq!(counts.accounts, 1, "accounts are counted once");
    assert_eq!(counts.in_battle, 1);
    assert_eq!(counts.watching, 0);
    assert_eq!(rx.borrow().accounts.len(), 1);
    assert_eq!(rx.borrow().accounts[0].name, "Ana");
}

// P47 (BR-LV02, BR-LV09): the watch lifecycle and its refusals.

fn listed_match(lobby: &mut Lobby, id: LiveId, listing: Listing) -> mpsc::Receiver<MatchCmd> {
    let (tx, match_rx) = mpsc::channel::<MatchCmd>(8);
    let mut m = fake_live([("Ana", Some(1000)), ("Bia", None)], Instant::now());
    m.tx = tx;
    m.listing = listing;
    lobby.register_live(id, m);
    match_rx
}

#[test]
fn watch_refusals_follow_the_spec_order() {
    let (mut lobby, _self_rx) = lobby();
    let mut match_rx = listed_match(&mut lobby, 1, Listing::Public);
    let _private_rx = listed_match(&mut lobby, 2, Listing::Private);
    let mut w = add_session(&mut lobby, [1; 16], SessionState::Idle);
    let mut q = add_session(&mut lobby, [2; 16], SessionState::Queued);
    let mut w2 = add_session(&mut lobby, [3; 16], SessionState::Idle);

    lobby.spectate([1; 16], 1);
    assert!(matches!(
        match_rx.try_recv(),
        Ok(MatchCmd::Watch { conn }) if conn.id == 1
    ));
    assert!(matches!(
        lobby.sessions[&[1; 16]].state,
        SessionState::Watching { id: 1 }
    ));
    assert_eq!(lobby.live[&1].watchers, vec![[1; 16]]);

    lobby.spectate([1; 16], 1);
    expect(
        &mut w,
        ServerMsg::SpectateUnavailable {
            reason: SpectateRefusal::RateLimited,
        },
    );
    lobby.spectate([2; 16], 1);
    expect(
        &mut q,
        ServerMsg::SpectateUnavailable {
            reason: SpectateRefusal::NotIdle,
        },
    );
    lobby.spectate([3; 16], 99);
    expect(
        &mut w2,
        ServerMsg::SpectateUnavailable {
            reason: SpectateRefusal::NotFound,
        },
    );
    lobby.spectate([3; 16], 2);
    expect(
        &mut w2,
        ServerMsg::SpectateUnavailable {
            reason: SpectateRefusal::RateLimited,
        },
    );
    lobby.sessions.get_mut(&[3; 16]).unwrap().last_spectate = None;
    lobby.spectate([3; 16], 2);
    expect(
        &mut w2,
        ServerMsg::SpectateUnavailable {
            reason: SpectateRefusal::NotWatchable,
        },
    );

    lobby.config = Arc::new(Config {
        spectators_per_match: 1,
        ..Config::default()
    });
    lobby.sessions.get_mut(&[3; 16]).unwrap().last_spectate = None;
    lobby.spectate([3; 16], 1);
    expect(
        &mut w2,
        ServerMsg::SpectateUnavailable {
            reason: SpectateRefusal::Full,
        },
    );
}

#[test]
fn a_watcher_returns_to_idle_on_stop_disconnect_and_match_end() {
    let (mut lobby, _self_rx) = lobby();
    let mut match_rx = listed_match(&mut lobby, 1, Listing::Public);
    add_session(&mut lobby, [1; 16], SessionState::Idle);
    add_session(&mut lobby, [2; 16], SessionState::Idle);
    lobby.spectate([1; 16], 1);
    lobby.spectate([2; 16], 1);
    while match_rx.try_recv().is_ok() {}

    lobby.client([1; 16], ClientMsg::StopSpectating);
    assert!(matches!(lobby.sessions[&[1; 16]].state, SessionState::Idle));
    assert!(matches!(
        match_rx.try_recv(),
        Ok(MatchCmd::Unwatch { conn_id: 1 })
    ));
    assert_eq!(lobby.live[&1].watchers, vec![[2; 16]]);

    lobby.disconnected([2; 16], 1);
    assert!(matches!(lobby.sessions[&[2; 16]].state, SessionState::Idle));
    assert!(lobby.live[&1].watchers.is_empty());

    lobby.sessions.get_mut(&[2; 16]).unwrap().last_spectate = None;
    lobby.spectate([2; 16], 1);
    lobby.live_ended(1);
    assert!(matches!(lobby.sessions[&[2; 16]].state, SessionState::Idle));
}
