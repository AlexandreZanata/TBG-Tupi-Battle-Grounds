//! Applies `UiAction`s: start practice/online play, send intentions,
//! open/close panels, copy the room link.

use bevy::prelude::*;
use tbg_core::protocol::ClientMsg;
use tbg_core::PlayerIntention;

use super::account::Account;
use super::textform::ActiveForm;
use super::{evolve, PendingBuy, UiAction};
use crate::audio::{Sound, Sounds};
use crate::link::MatchLink;
use crate::local::LocalMatch;
use crate::net::{Mode, NetLink};
use crate::sync::{LatestView, QueueInfo};
use crate::{config, GameState};

#[derive(Resource, Default)]
pub struct ActionQueue(pub Vec<UiAction>);

#[derive(Resource, Default)]
pub(super) struct IntentSeq(u32);

/// What "Play again" repeats.
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq)]
pub(super) enum LastMode {
    #[default]
    Practice,
    Quick,
    Ranked,
}

pub fn plugin(app: &mut App) {
    app.init_resource::<ActionQueue>()
        .init_resource::<IntentSeq>()
        .init_resource::<LastMode>();
}

#[allow(clippy::too_many_arguments)] // the one place every action is applied
pub(super) fn apply(
    mut queue: ResMut<ActionQueue>,
    mut link: NonSendMut<MatchLink>,
    mut next: ResMut<NextState<GameState>>,
    mut seq: ResMut<IntentSeq>,
    mut pending: ResMut<PendingBuy>,
    mut selected: ResMut<evolve::Selected>,
    mut last: ResMut<LastMode>,
    mut queue_info: ResMut<QueueInfo>,
    time: Res<Time>,
    mut extra: (
        Res<LatestView>,
        ResMut<super::Toasts>,
        ResMut<Sounds>,
        ResMut<crate::sync::RematchInfo>,
    ),
    mut account: ResMut<Account>,
    (mut surrender, mut fortifying, mut tower_open): (
        ResMut<super::surrender::Surrender>,
        ResMut<super::fortify::PendingFortify>,
        ResMut<super::fortify::TowerOpen>,
    ),
    mut form: NonSendMut<ActiveForm>,
    mut ranking: ResMut<super::ranking::Ranking>,
    mut inspected: ResMut<super::inspect::Inspected>,
    (mut lang_res, mut arena): (ResMut<crate::i18n::Lang>, ResMut<crate::arena::ActiveArena>),
) {
    let now = time.elapsed_secs_f64();
    // BR-AG12: while an Augment selection pauses the battle, only the War
    // Council answers; the server would reject the rest anyway.
    let paused = extra
        .0
         .0
        .as_ref()
        .is_some_and(|v| v.augment_round.is_some());
    if paused {
        selected.0 = None;
    }
    for action in std::mem::take(&mut queue.0) {
        if paused
            && matches!(
                action,
                UiAction::Reserve { .. }
                    | UiAction::Release(_)
                    | UiAction::Reroll
                    | UiAction::Buy(_)
                    | UiAction::Evolve { .. }
                    | UiAction::SelectUnit(_)
                    | UiAction::Fortify(_)
            )
        {
            continue;
        }
        if !matches!(
            action,
            UiAction::Disabled
                | UiAction::CloseEvolve
                | UiAction::SelectUnit(_)
                | UiAction::SelectTower
                | UiAction::Deselect
                | UiAction::CloseInspect
        ) {
            extra.2.play(Sound::Click);
        }
        let mut online = |mode: Mode, link: &mut MatchLink, last: &mut LastMode| {
            // P12: ranked stays ranked for Play again; rooms re-queue casual.
            *last = if mode == Mode::Ranked {
                LastMode::Ranked
            } else {
                LastMode::Quick
            };
            // Reuse the probe socket when it is still idle and open.
            if let MatchLink::Net(n) = link {
                if n.reuse(mode, now) {
                    *queue_info = QueueInfo::default();
                    next.set(GameState::Queue);
                    return;
                }
            }
            *link = MatchLink::Net(Box::new(NetLink::new(mode, now)));
            *queue_info = QueueInfo::default();
            next.set(GameState::Queue);
        };
        match action {
            UiAction::Practice => {
                *last = LastMode::Practice;
                start_practice(&mut link, now)
            }
            UiAction::PlayAgain if *last == LastMode::Practice => start_practice(&mut link, now),
            // P12: Play again re-queues the same mode.
            UiAction::PlayAgain if *last == LastMode::Ranked => {
                online(Mode::Ranked, &mut link, &mut last)
            }
            UiAction::Quick | UiAction::PlayAgain => online(Mode::Quick, &mut link, &mut last),
            UiAction::Ranked => online(Mode::Ranked, &mut link, &mut last),
            UiAction::CreateRoom => online(Mode::Create, &mut link, &mut last),
            UiAction::JoinRoom(code) => online(Mode::Join(code), &mut link, &mut last),
            UiAction::CreateChallenge => online(Mode::CreateChallenge, &mut link, &mut last),
            UiAction::JoinChallenge(token) => {
                online(Mode::JoinChallenge(token), &mut link, &mut last);
                queue_info.joining_challenge = true;
            }
            UiAction::ChallengeInvalid => extra.1.push("Challenge link invalid"),
            UiAction::ShareChallenge => {
                if let Some(token) = queue_info.challenge {
                    let url = config::challenge_link(&token);
                    config::share_text("Join my TBG challenge", &url);
                }
            }
            UiAction::RequestRematch => link.send(ClientMsg::RequestRematch, now),
            UiAction::Watch(id) => {
                config::save_watch(id);
                *queue_info = QueueInfo::default();
                *link = MatchLink::Net(Box::new(NetLink::new(Mode::Watch(id), now)));
            }
            UiAction::LeaveWatch => {
                link.send(ClientMsg::StopSpectating, now);
                let parked = if let MatchLink::Net(n) = &mut *link {
                    n.reuse(Mode::Probe, now)
                } else {
                    false
                };
                if !parked {
                    *link = MatchLink::Idle;
                }
                config::clear_watch();
                config::clear_url_param("watch");
                next.set(GameState::Menu);
            }
            UiAction::Cancel | UiAction::ToMenu => {
                link.send(ClientMsg::Cancel, now);
                let parked = if let MatchLink::Net(n) = &mut *link {
                    n.reuse(Mode::Probe, now)
                } else {
                    false
                };
                if !parked {
                    *link = MatchLink::Idle;
                }
                *queue_info = QueueInfo::default();
                queue_info.cancelled = true;
                extra.3.dismissed = true;
                next.set(GameState::Menu);
            }
            // BR-M06: first tap arms (4 s), second forfeits. Practice leaves
            // at once; online waits for the server's MatchEnd.
            UiAction::Surrender => {
                if !surrender.sent {
                    if surrender.armed_until.take().is_some() {
                        surrender.sent = true;
                        if matches!(&*link, MatchLink::Local(_)) {
                            *link = MatchLink::Idle;
                            next.set(GameState::Menu);
                        } else {
                            link.send(ClientMsg::Surrender, now);
                        }
                    } else {
                        surrender.armed_until = Some(now + super::surrender::ARMED_SECS);
                    }
                }
            }
            UiAction::CopyLink => {
                if let Some(token) = queue_info.challenge {
                    config::copy_text(&config::challenge_link(&token));
                } else if let Some(code) = queue_info.room {
                    config::copy_text(&config::room_link(&String::from_utf8_lossy(&code)));
                }
            }
            UiAction::ChooseAugment(option) => {
                if let Some(round) = extra.0 .0.as_ref().and_then(|v| v.augment_round.as_ref()) {
                    if !round.resolved && round.ticks_left > 0 {
                        seq.0 += 1;
                        link.send(
                            ClientMsg::Intent {
                                seq: seq.0,
                                intent: PlayerIntention::ChooseAugment {
                                    round: round.round,
                                    option,
                                },
                            },
                            now,
                        );
                    }
                }
            }
            UiAction::ForgoAugment => {
                if let Some(round) = extra.0 .0.as_ref().and_then(|v| v.augment_round.as_ref()) {
                    if !round.resolved && round.ticks_left > 0 {
                        seq.0 += 1;
                        link.send(
                            ClientMsg::Intent {
                                seq: seq.0,
                                intent: PlayerIntention::Forgo { round: round.round },
                            },
                            now,
                        );
                    }
                }
            }
            UiAction::Reserve { offer, .. } | UiAction::Release(offer) => {
                if pending.0.is_some() {
                    continue;
                }
                let intent = match action {
                    UiAction::Reserve { kind, .. } => PlayerIntention::Reserve { offer, kind },
                    _ => PlayerIntention::Release { offer },
                };
                seq.0 += 1;
                link.send(ClientMsg::Intent { seq: seq.0, intent }, now);
                pending.0 = Some((u8::MAX, seq.0));
            }
            UiAction::Reroll => {
                if pending.0.is_some() {
                    continue;
                }
                if let Some(cost) = extra.0 .0.as_ref().and_then(|v| v.reroll_cost) {
                    seq.0 += 1;
                    link.send(
                        ClientMsg::Intent {
                            seq: seq.0,
                            intent: PlayerIntention::Reroll { cost },
                        },
                        now,
                    );
                    pending.0 = Some((u8::MAX, seq.0));
                }
            }
            UiAction::Buy(offer) => {
                if pending.0.is_some_and(|(slot, _)| slot == u8::MAX) {
                    continue;
                }
                seq.0 += 1;
                link.send(
                    ClientMsg::Intent {
                        seq: seq.0,
                        intent: PlayerIntention::Purchase { offer },
                    },
                    now,
                );
                pending.0 = Some((offer, seq.0));
            }
            // Your Stronghold behaves like a unit: its panel replaces the other one.
            UiAction::SelectTower => {
                inspected.0 = None;
                selected.0 = None;
                tower_open.0 = true;
            }
            UiAction::SelectUnit(id) => {
                tower_open.0 = false;
                inspected.0 = Some(id);
                // Evolution only for own units; enemies just show their status.
                let own = extra
                    .0
                     .0
                    .as_ref()
                    .is_some_and(|v| v.units.iter().any(|u| u.id == id && u.seat == v.you));
                selected.0 = own.then_some(id);
            }
            UiAction::Evolve { unit, branch } => {
                seq.0 += 1;
                link.send(
                    ClientMsg::Intent {
                        seq: seq.0,
                        intent: PlayerIntention::Evolve { unit, branch },
                    },
                    now,
                );
                selected.0 = None;
            }
            // D-46: one Fortify in flight; the tier shown changes only when
            // the authoritative view does.
            UiAction::Fortify(branch) => {
                if fortifying.0.is_none() {
                    seq.0 += 1;
                    link.send(
                        ClientMsg::Intent {
                            seq: seq.0,
                            intent: PlayerIntention::Fortify { branch },
                        },
                        now,
                    );
                    fortifying.0 = Some(seq.0);
                }
            }
            UiAction::CloseEvolve => selected.0 = None,
            UiAction::Deselect => {
                selected.0 = None;
                inspected.0 = None;
                tower_open.0 = false;
            }
            UiAction::CloseInspect => inspected.0 = None,
            UiAction::Inspect(offer) => {
                if let Some(v) = &extra.0 .0 {
                    let kind = v.offers[usize::from(offer).min(tbg_core::content::OFFERS - 1)];
                    extra.1.push_for(super::tips::unit_details(kind), 4.5);
                }
            }
            UiAction::ToggleSound => extra.2.toggle_mute(),
            // P41: menu-only switch (the Settings dialog lives in the menu).
            UiAction::SetLang(lang) => *lang_res = lang,
            UiAction::PreviewArena(a) => {
                arena.set_if_neq(crate::arena::ActiveArena::of(a));
            }
            UiAction::Disabled => {}
            UiAction::OpenRanking
            | UiAction::RankingPrev
            | UiAction::RankingNext
            | UiAction::OpenFullRanking
            | UiAction::CloseRanking => super::ranking::apply_action(action, &mut ranking),
            // P11: account buttons (state lives in ui/account.rs).
            other => {
                super::account_ui::apply_action(
                    other,
                    &mut link,
                    &mut account,
                    &mut form,
                    &mut extra.1,
                    now,
                );
            }
        }
        if matches!(action, UiAction::Practice | UiAction::PlayAgain) {
            pending.0 = None;
            selected.0 = None;
        }
    }
}

fn start_practice(link: &mut MatchLink, now: f64) {
    let seed = (now * 1e9) as u64 ^ 0x9e37_79b9_7f4a_7c15;
    let arcanas = config::load_arcanas();
    let stronghold = config::load_stronghold();
    // P30-A debug only: bounded so a typo cannot freeze the tab on the open-army purchase loop.
    let army = config::url_param("army")
        .and_then(|value| value.parse::<usize>().ok())
        .map(|n| n.min(300));
    let arena = config::load_arena();
    *link = MatchLink::Local(Box::new(LocalMatch::new(
        seed, arcanas, stronghold, army, arena,
    )));
}
