//! Applies `UiAction`s: start practice/online play, send intentions,
//! open/close panels, copy the room link.

use bevy::prelude::*;
use tbg_core::protocol::{valid_room_code, ClientMsg, RoomCode};
use tbg_core::PlayerIntention;

use super::{evolve, PendingBuy, UiAction};
use crate::link::MatchLink;
use crate::local::LocalMatch;
use crate::net::{Mode, NetLink};
use crate::sync::QueueInfo;
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
    Online,
}

pub fn plugin(app: &mut App) {
    app.init_resource::<ActionQueue>()
        .init_resource::<IntentSeq>()
        .init_resource::<LastMode>()
        .add_systems(Startup, join_from_url);
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
) {
    let now = time.elapsed_secs_f64();
    for action in std::mem::take(&mut queue.0) {
        let replay_practice = *last == LastMode::Practice;
        let mut online = |mode: Mode, link: &mut MatchLink, last: &mut LastMode| {
            *link = MatchLink::Net(Box::new(NetLink::new(mode, now)));
            *queue_info = QueueInfo::default();
            *last = LastMode::Online;
            next.set(GameState::Queue);
        };
        match action {
            UiAction::Practice => {
                *last = LastMode::Practice;
                start_practice(&mut link, now)
            }
            UiAction::PlayAgain if replay_practice => start_practice(&mut link, now),
            UiAction::Quick | UiAction::PlayAgain => online(Mode::Quick, &mut link, &mut last),
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
            UiAction::Disabled => {}
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
