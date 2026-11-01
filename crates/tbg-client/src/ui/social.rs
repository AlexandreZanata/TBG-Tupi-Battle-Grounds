//! Private invitation state and shared menu actions; activity never follows a directory snapshot implicitly.

use super::{
    kit,
    shell::{self, Panel},
};
use crate::{link::MatchLink, GameState};
use bevy::prelude::*;
use tbg_core::protocol::{ClientMsg, PlayerInviteNotice, PlayerInviteOutcome};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Name([u8; 16]);
impl Name {
    pub fn new(s: &str) -> Option<Self> {
        if !tbg_core::account::valid_name_shape(s) || s.len() > 16 {
            return None;
        }
        let mut b = [0; 16];
        b[..s.len()].copy_from_slice(s.as_bytes());
        Some(Self(b))
    }
    pub fn text(self) -> String {
        String::from_utf8_lossy(&self.0)
            .trim_end_matches('\0')
            .into()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Search,
    ClearSearch,
    Available,
    Page(i8),
    Challenge(Name),
    Answer([u8; 16], bool),
    Cancel([u8; 16]),
    CancelConnecting,
    Enabled,
    Profile(Name),
    FullProfile(Name),
    History(Name),
    Mine,
    HistoryNext,
    HistoryBack,
    HistoryList,
    HistoryDetail([u8; 16]),
    HistoryRefresh,
    HistoryMode,
    HistoryOutcome,
    Visibility(bool),
    Preview,
}
#[derive(Resource, Default)]
pub struct Requests(pub Vec<Action>);
#[derive(Resource, Default)]
pub struct Social {
    pub enabled: bool,
    pub outgoing: Option<PlayerInviteNotice>,
    pub incoming: Vec<PlayerInviteNotice>,
    pub notice: Option<PlayerInviteOutcome>,
    pub received_at: f64,
    epoch: u64,
    applied: Option<tbg_core::protocol::Token>,
    pending_send: Option<(Name, f64)>,
}
impl Social {
    pub fn receive(
        &mut self,
        enabled: bool,
        outgoing: Option<PlayerInviteNotice>,
        incoming: Vec<PlayerInviteNotice>,
        now: f64,
    ) {
        self.enabled = enabled;
        self.outgoing = outgoing;
        self.incoming = incoming.into_iter().take(5).collect();
        self.received_at = now;
    }
    pub fn remaining(&self, notice: &PlayerInviteNotice, now: f64) -> u16 {
        notice
            .remaining_s
            .saturating_sub((now - self.received_at).max(0.) as u16)
    }
}
pub fn plugin(app: &mut App) {
    app.init_resource::<Social>()
        .init_resource::<Requests>()
        .init_resource::<super::history::History>()
        .add_systems(Update, (apply, super::history::poll, countdown).chain());
}
#[allow(clippy::too_many_arguments)]
fn apply(
    mut requests: ResMut<Requests>,
    mut social: ResMut<Social>,
    mut history: ResMut<super::history::History>,
    mut data: ResMut<shell::data::Data>,
    mut shell: ResMut<shell::Shell>,
    mut form: NonSendMut<super::textform::ActiveForm>,
    mut link: NonSendMut<MatchLink>,
    time: Res<Time>,
    lang: Res<crate::i18n::Lang>,
    state: Res<State<GameState>>,
    mut personal: (
        ResMut<crate::sync::MatchInfo>,
        ResMut<crate::sync::RematchInfo>,
    ),
) {
    let now = time.elapsed_secs_f64();
    if social.epoch != crate::auth::epoch() {
        *social = Social {
            epoch: crate::auth::epoch(),
            ..default()
        };
        personal.0.personal = None;
        personal.0.history_save = None;
        personal.1.open = false;
        personal.1.opponent.clear();
        personal.1.dismissed = true;
        shell.dirty = true;
    }
    if let MatchLink::Net(net) = &*link {
        let token = crate::config::load_token();
        if matches!(net.status, crate::net::NetStatus::Ready)
            && token.is_some()
            && token != social.applied
            && !matches!(
                crate::auth::snapshot().status,
                crate::auth::Status::Restoring
                    | crate::auth::Status::Refreshing
                    | crate::auth::Status::Conflict
            )
        {
            social.applied = token;
            link.send(
                ClientMsg::SetPlayerChallengesEnabled {
                    enabled: !crate::config::load_flag("players.invites_disabled"),
                },
                now,
            );
        }
    }
    if !matches!(*state.get(), GameState::Menu | GameState::Result) {
        requests.0.clear();
        social.pending_send = None;
        return;
    }
    if let Some((target, until)) = social.pending_send {
        if now >= until {
            social.pending_send = None;
            social.notice = Some(PlayerInviteOutcome::Unavailable);
            shell.dirty = true;
        } else if matches!(link.status(), Some(crate::net::NetStatus::Ready)) {
            social.pending_send = None;
            link.send(
                ClientMsg::ChallengePlayer {
                    name: target.text(),
                },
                now,
            );
            shell.dirty = true;
        }
    }
    for action in std::mem::take(&mut requests.0) {
        shell.dirty = true;
        match action {
            Action::Search => super::textform::open(
                &mut form,
                super::textform::TextKind::PlayerSearch,
                &lang.key("p73.search"),
            ),
            Action::ClearSearch => data.search(String::new()),
            Action::Available => {
                data.online_available = !data.online_available;
                data.reset_online();
            }
            Action::Page(direction) => data.page(direction),
            Action::Challenge(name) => {
                social.notice = None;
                if matches!(*link, MatchLink::Idle) {
                    *link = MatchLink::Net(Box::new(crate::net::NetLink::new(
                        crate::net::Mode::Probe,
                        now,
                    )));
                }
                social.pending_send = Some((name, now + crate::net::GIVE_UP));
            }
            Action::Answer(id, accept) => {
                if accept
                    && !social
                        .incoming
                        .iter()
                        .any(|n| n.id == id && social.remaining(n, now) > 0)
                {
                    social.notice = Some(PlayerInviteOutcome::Expired);
                    continue;
                }
                social.notice = None;
                link.send(ClientMsg::AnswerPlayerChallenge { id, accept }, now);
            }
            Action::Cancel(id) => link.send(ClientMsg::CancelPlayerChallenge { id }, now),
            Action::CancelConnecting => social.pending_send = None,
            Action::Enabled => {
                let enabled = !social.enabled;
                crate::config::save_flag("players.invites_disabled", !enabled);
                social.applied = None;
            }
            Action::FullProfile(name) => {
                if let Some(url) =
                    crate::platform::page_url(&format!("/ranking/player.html?name={}", name.text()))
                {
                    crate::config::open_url(&url);
                }
            }
            Action::Profile(name) => {
                history.profile(name.text());
                shell.panel = Panel::PlayerProfile;
            }
            Action::History(name) => {
                history.open(name.text(), false);
                shell.panel = Panel::History;
            }
            Action::Mine => {
                history.mine();
                shell.panel = Panel::History;
            }
            Action::HistoryNext => history.next(),
            Action::HistoryBack => history.previous(),
            Action::HistoryList => history.return_list(),
            Action::HistoryDetail(id) => history.detail(id),
            Action::HistoryRefresh => history.refresh(),
            Action::HistoryMode => history.mode(),
            Action::HistoryOutcome => history.outcome(),
            Action::Visibility(public) => history.visibility(public),
            Action::Preview => history.preview(),
        }
    }
}

pub fn cards(
    parent: &mut ChildSpawnerCommands,
    social: &Social,
    lang: crate::i18n::Lang,
    now: f64,
) {
    parent.spawn(shell::button(
        format!(
            "{}: {}",
            lang.key("p73.invites"),
            lang.key(if social.enabled { "p73.on" } else { "p73.off" })
        ),
        shell::Action::Social(Action::Enabled),
        260.,
    ));
    if social.pending_send.is_some() {
        parent.spawn(kit::muted(lang.key("p73.connecting"), 16.));
        parent.spawn(shell::button(
            lang.key("p73.cancel"),
            shell::Action::Social(Action::CancelConnecting),
            180.,
        ));
    }
    if let Some(outcome) = social.notice {
        parent.spawn(kit::muted(
            lang.key(match outcome {
                PlayerInviteOutcome::Unavailable => "p73.unavailable",
                PlayerInviteOutcome::RateLimited => "p73.rate_limited",
                PlayerInviteOutcome::Declined => "p73.declined",
                PlayerInviteOutcome::Cancelled => "p73.cancelled",
                PlayerInviteOutcome::Expired => "p73.expired",
            }),
            16.,
        ));
    }
    if let Some(invite) = &social.outgoing {
        parent.spawn(kit::panel()).with_children(|p| {
            let label = format!("{} · {}", lang.key("p73.sent"), invite.peer_name);
            p.spawn((
                kit::heading(format!("{label} · {}s", social.remaining(invite, now)), 19.),
                Countdown {
                    deadline: social.received_at + f64::from(invite.remaining_s),
                    label,
                },
            ));
            p.spawn(shell::button(
                lang.key("p73.cancel"),
                shell::Action::Social(Action::Cancel(invite.id)),
                150.,
            ));
        });
    }
    for invite in &social.incoming {
        parent.spawn(kit::panel()).with_children(|p| {
            let label = format!("{} · {}", lang.key("p73.received"), invite.peer_name);
            p.spawn((
                kit::heading(format!("{label} · {}s", social.remaining(invite, now)), 19.),
                Countdown {
                    deadline: social.received_at + f64::from(invite.remaining_s),
                    label,
                },
            ));
            p.spawn(kit::row()).with_children(|r| {
                if social.remaining(invite, now) > 0 {
                    r.spawn(shell::button(
                        lang.key("p73.accept"),
                        shell::Action::Social(Action::Answer(invite.id, true)),
                        150.,
                    ));
                }
                r.spawn(shell::button(
                    lang.key("p73.decline"),
                    shell::Action::Social(Action::Answer(invite.id, false)),
                    150.,
                ));
            });
        });
    }
}

#[cfg(feature = "ui-acceptance")]
impl Social {
    pub fn visual_fixture(&mut self, now: f64) {
        *self = Self {
            epoch: crate::auth::epoch(),
            applied: self.applied,
            enabled: true,
            received_at: now,
            outgoing: Some(PlayerInviteNotice {
                id: [73; 16],
                peer_name: "FixtureRival".into(),
                peer_is_guest: false,
                remaining_s: 45,
            }),
            incoming: vec![PlayerInviteNotice {
                id: [74; 16],
                peer_name: "FixtureFriend".into(),
                peer_is_guest: false,
                remaining_s: 40,
            }],
            ..default()
        };
    }
}

#[derive(Component)]
struct Countdown {
    deadline: f64,
    label: String,
}
fn countdown(time: Res<Time>, mut labels: Query<(&Countdown, &mut Text)>) {
    for (countdown, mut text) in &mut labels {
        text.0 = format!(
            "{} · {}s",
            countdown.label,
            (countdown.deadline - time.elapsed_secs_f64())
                .max(0.)
                .ceil() as u16
        );
    }
}
