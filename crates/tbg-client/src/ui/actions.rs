//! Applies `UiAction`s: start practice/online play, send intentions,
//! open/close panels, copy the room link.

use bevy::prelude::*;
use tbg_core::protocol::{valid_room_code, ClientMsg, RoomCode};
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
        .init_resource::<LastMode>()
        .add_systems(Startup, (join_from_url, probe_on_load));
}

/// P9: with a stored session token and no `?room=`, open a listen-only
/// link at once; a live match resumes with no tap, otherwise the socket
/// is reused by the next Quick Match / room action.
/// P11: a stored account key rides along and logs in after `Welcome`.
fn probe_on_load(mut link: NonSendMut<MatchLink>, mut account: ResMut<Account>) {
    if config::url_param("room").is_some() {
        return;
    }
    if config::load_token().is_some() {
        *link = MatchLink::Net(Box::new(NetLink::new(Mode::Probe, 0.0)));
        arm_auto_login(&mut link, &mut account);
    }
}

/// P11: a stored key logs in right after the probe's `Welcome`.
fn arm_auto_login(link: &mut MatchLink, account: &mut Account) {
    let MatchLink::Net(n) = link else {
        return;
    };
    if let Some(key) = config::load_account_key() {
        n.login_after_welcome(key);
        account.auto_login = true;
    }
}

/// `?room=CODE` joins that private room right away.
fn join_from_url(mut queue: ResMut<ActionQueue>) {
    let Some(code) = config::url_param("room") else {
        return;
    };
    let bytes = code.to_ascii_uppercase().into_bytes();
    if let Ok(code) = RoomCode::try_from(bytes.as_slice()) {
        if valid_room_code(&code) {
            queue.0.push(UiAction::JoinRoom(code));
        }
    }
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
    mut extra: (Res<LatestView>, ResMut<super::Toasts>, ResMut<Sounds>),
    mut account: ResMut<Account>,
    mut form: NonSendMut<ActiveForm>,
    mut ranking: ResMut<super::ranking::Ranking>,
) {
    let now = time.elapsed_secs_f64();
    for action in std::mem::take(&mut queue.0) {
        if !matches!(
            action,
            UiAction::Disabled | UiAction::CloseEvolve | UiAction::SelectUnit(_)
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
            UiAction::Cancel | UiAction::ToMenu => {
                link.send(ClientMsg::Cancel, now);
                *link = MatchLink::Idle;
                next.set(GameState::Menu);
            }
            UiAction::CopyLink => {
                if let Some(code) = queue_info.room {
                    config::copy_text(&config::room_link(&String::from_utf8_lossy(&code)));
                }
            }
            UiAction::Buy(offer) => {
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
            UiAction::SelectUnit(id) => selected.0 = Some(id),
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
            UiAction::CloseEvolve => selected.0 = None,
            UiAction::Inspect(offer) => {
                if let Some(v) = &extra.0 .0 {
                    let kind = v.offers[usize::from(offer.min(2))];
                    extra.1.push_for(super::tips::unit_details(kind), 4.5);
                }
            }
            UiAction::ToggleSound => extra.2.toggle_mute(),
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
    *link = MatchLink::Local(Box::new(LocalMatch::new(seed)));
}
