//! Shared lobby navigation and screen lifecycle; gameplay uses existing UiActions.

mod arcanas;
mod arenas;
#[cfg(tbg_desktop)]
mod catalog;
pub(crate) mod data;
mod header;
mod input;
mod live;
mod lobby;
mod patch_notes;
mod settings;
mod strongholds;
#[cfg(tbg_desktop)]
pub(super) use data::Data as LobbyData;

#[cfg(tbg_desktop)]
pub(super) use strongholds::{branch_effect, expand, stronghold_data};

use super::{
    account::{Account, AccountScreen},
    kit,
    ranking::Ranking,
    textform::{self, ActiveForm, TextEntered, TextKind},
    UiAction,
};
use crate::{audio::Sounds, i18n::Lang, GameState};
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Panel {
    #[default]
    Lobby,
    Friends,
    Settings,
    Arenas,
    Strongholds,
    Arcanas,
    Live,
    History,
    PlayerProfile,
    Kit,
    Troops,
    PatchNotes,
}
#[derive(Component, Clone, Copy, Debug)]
pub enum Action {
    Open(Panel),
    Social(super::social::Action),
    Catalog(crate::training::Action),
    RoomInput,
    Stronghold(tbg_core::StrongholdKind),
    Arena(tbg_core::Arena),
    Arcana(usize, i8),
    ResetArcanas,
    DefaultArcanas,
    News(i8),
    Fullscreen,
    Vsync,
    Quality,
    CopyWatch(u64),
    Download,
    Volume(u8),
}
#[derive(Resource)]
pub struct Shell {
    pub panel: Panel,
    pub dirty: bool,
    pub news: usize,
    pub draft: tbg_core::ArcanaPreset,
    pub notice: String,
}
impl Default for Shell {
    fn default() -> Self {
        let panel = match crate::config::url_param("panel").as_deref() {
            Some("troops") => Panel::Troops,
            Some("patch-notes") => Panel::PatchNotes,
            Some("arcanas") => Panel::Arcanas,
            Some("stronghold") => Panel::Strongholds,
            Some("arenas") => Panel::Arenas,
            Some("settings") => Panel::Settings,
            Some("live") => Panel::Live,
            _ if crate::config::url_param("uikit").is_some() => Panel::Kit,
            _ => Panel::Lobby,
        };
        Self {
            panel,
            dirty: true,
            news: 0,
            draft: crate::config::load_arcanas(),
            notice: String::new(),
        }
    }
}
#[derive(Component)]
struct Root;
#[derive(Component)]
struct Body;

/// Every Linux menu page reserves the same clear region below the shared header.
#[cfg(tbg_desktop)]
pub(super) fn page_node() -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        row_gap: px(16),
        overflow: Overflow::scroll_y(),
        ..lobby::chrome::region(2., 17., 96., 79.)
    }
}

pub fn plugin(app: &mut App) {
    app.init_resource::<Shell>()
        .init_resource::<data::Data>()
        .add_systems(
            Update,
            (data::poll, input::mouse_and_keys, room_text, render)
                .chain()
                .run_if(in_state(GameState::Menu)),
        )
        .add_systems(
            Update,
            input::mouse_and_keys.run_if(in_state(GameState::Training)),
        )
        .add_systems(OnEnter(GameState::Menu), enter)
        .add_systems(Update, input::scroll);
}
fn enter(mut shell: ResMut<Shell>) {
    shell.dirty = true;
}

pub fn button(text: impl Into<String>, action: Action, width: f32) -> impl Bundle {
    (kit::control(text, true, width), UiAction::Disabled, action)
}

fn room_text(
    mut events: MessageReader<TextEntered>,
    mut form: NonSendMut<ActiveForm>,
    mut queue: ResMut<super::ActionQueue>,
    mut data: ResMut<data::Data>,
    mut shell: ResMut<Shell>,
) {
    for event in events.read() {
        if event.kind == TextKind::PlayerSearch {
            data.search(event.value.clone());
            shell.dirty = true;
            textform::close(&mut form);
            continue;
        }
        if event.kind != TextKind::Room {
            continue;
        }
        let raw = event.value.trim().to_ascii_uppercase();
        if raw.len() == 5 && raw.bytes().all(|b| b.is_ascii_alphanumeric()) {
            if let Ok(code) = raw.as_bytes().try_into() {
                queue.0.push(UiAction::JoinRoom(code));
                textform::close(&mut form);
            }
        } else {
            textform::set_error(&form, "Enter a five-character room code");
        }
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)] // Explicit Bevy resources and independent scroll queries.
fn render(
    mut commands: Commands,
    mut shell: ResMut<Shell>,
    data: Res<data::Data>,
    assets: Res<AssetServer>,
    account: Res<Account>,
    ranking: Res<Ranking>,
    lang: Res<Lang>,
    catalog: Res<crate::training::Catalog>,
    (sounds, social, history, rematch, time): (
        Res<Sounds>,
        Res<super::social::Social>,
        Res<super::history::History>,
        Res<crate::sync::RematchInfo>,
        Res<Time>,
    ),
    #[cfg(tbg_desktop)] volume: Res<bevy::audio::GlobalVolume>,
    form: NonSend<ActiveForm>,
    roots: Query<Entity, With<Root>>,
    window: Single<&Window>,
    mut hidden: Local<(bool, bool, bool)>,
    bodies: (
        Query<&ScrollPosition, With<Body>>,
        Query<(&crate::training::catalog::Pane, &ScrollPosition)>,
    ),
    mut layout: Local<(Panel, Vec2)>,
) {
    if !crate::config::bevy_ui() || crate::config::arena_preview() {
        return;
    }
    let modal = matches!(account.screen, AccountScreen::SaveKey { .. });
    #[cfg(not(tbg_desktop))]
    let modal = modal || ranking.open;
    #[cfg(tbg_desktop)]
    let volume_changed = shell.panel == Panel::Settings && volume.is_changed();
    #[cfg(not(tbg_desktop))]
    let volume_changed = false;
    if !shell.dirty
        && !volume_changed
        && !account.is_changed()
        && !lang.is_changed()
        && sounds.muted == hidden.1
        && modal == hidden.0
        && ranking.open == hidden.2
        && layout.1 == Vec2::new(window.width(), window.height())
        && !roots.is_empty()
    {
        return;
    }
    let scroll = if layout.0 == shell.panel {
        bodies
            .0
            .iter()
            .next()
            .map(|p| ScrollPosition(p.0))
            .unwrap_or_default()
    } else {
        ScrollPosition::default()
    };
    *layout = (shell.panel, Vec2::new(window.width(), window.height()));
    shell.dirty = false;
    *hidden = (modal, sounds.muted, ranking.open);
    for e in &roots {
        commands.entity(e).despawn();
    }
    if modal {
        return;
    }
    let compact = window.width() < 900.;
    let home = shell.panel == Panel::Lobby;
    #[cfg(tbg_desktop)]
    let home = home && !ranking.open;
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(px(if cfg!(tbg_desktop) || home {
                    0
                } else if compact {
                    10
                } else {
                    24
                })),
                row_gap: px(16),
                ..default()
            },
            BackgroundColor(if home || shell.panel == Panel::Troops {
                Color::NONE
            } else {
                Color::srgba(0.02, 0.035, 0.045, 0.72)
            }),
            Root,
            DespawnOnExit(GameState::Menu),
        ))
        .with_children(|root| {
            #[cfg(tbg_desktop)]
            {
                header::spawn(
                    root,
                    &data,
                    &account,
                    &assets,
                    *lang,
                    shell.panel,
                    ranking.open,
                );
                if ranking.open {
                    return;
                }
            }
            if home {
                lobby::spawn(root, &shell, &data, &account, &assets, *lang);
                if social.outgoing.is_some() || !social.incoming.is_empty() {
                    root.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            right: px(24),
                            top: percent(18),
                            width: px(420),
                            max_height: percent(66),
                            overflow: Overflow::scroll_y(),
                            flex_direction: FlexDirection::Column,
                            ..default()
                        },
                        BackgroundColor(kit::PANEL),
                    ))
                    .with_children(|p| {
                        super::social::cards(p, &social, *lang, time.elapsed_secs_f64())
                    });
                }
                return;
            }
            #[cfg(not(tbg_desktop))]
            navigation(root, *lang, shell.panel, &data, compact, &assets);
            #[cfg(not(tbg_desktop))]
            if let Some(version) = &data.update {
                root.spawn(button(
                    format!("{} · {version}", lang.key("native.update")),
                    Action::Download,
                    320.,
                ));
            }
            #[cfg(not(tbg_desktop))]
            let body = Node {
                flex_grow: 1.,
                min_height: px(0),
                width: percent(100),
                overflow: Overflow::scroll_y(),
                flex_direction: FlexDirection::Column,
                row_gap: px(16),
                ..default()
            };
            #[cfg(tbg_desktop)]
            let body = {
                let mut node = page_node();
                if shell.panel == Panel::Troops {
                    node.overflow = Overflow::clip();
                }
                node
            };
            root.spawn((body, scroll, Body)).with_children(|body| {
                #[cfg(feature = "ui-acceptance")]
                if crate::config::url_param("capture_panel").is_some_and(|v| v.starts_with("p73-"))
                {
                    body.spawn(kit::muted(lang.key("p73.fixture"), 16.));
                }
                #[cfg(tbg_desktop)]
                if let Some(version) = &data.update {
                    body.spawn(button(
                        format!("{} · {version}", lang.key("native.update")),
                        Action::Download,
                        320.,
                    ));
                }
                match shell.panel {
                    Panel::Lobby => {}
                    Panel::Friends => lobby::friends(body, *lang),
                    #[cfg(not(tbg_desktop))]
                    Panel::Settings => settings::spawn(body, &account, &sounds, &window, *lang),
                    #[cfg(tbg_desktop)]
                    Panel::Settings => settings::spawn(
                        body,
                        &account,
                        &sounds,
                        &window,
                        *lang,
                        volume.volume.to_linear(),
                        &assets,
                    ),
                    Panel::Arenas => arenas::spawn(body, &assets, *lang),
                    Panel::Strongholds => strongholds::spawn(body, &assets, *lang),
                    Panel::Arcanas => arcanas::spawn(body, &shell, &assets, *lang, compact),
                    Panel::Live => {
                        live::spawn(body, &data, *lang);
                        super::social::cards(body, &social, *lang, time.elapsed_secs_f64());
                        super::history::rematch_card(body, &rematch, *lang);
                        super::history::players(body, &data, &account, *lang);
                    }
                    Panel::History => super::history::spawn(body, &history, *lang),
                    Panel::PlayerProfile => super::history::profile(body, &history, *lang),
                    Panel::Troops => {
                        crate::training::catalog::spawn(body, &catalog, &assets, *lang, {
                            let mut positions = [Vec2::ZERO; 2];
                            for (pane, position) in &bodies.1 {
                                positions[pane.0] = position.0;
                            }
                            positions
                        })
                    }
                    Panel::PatchNotes => patch_notes::spawn(body, &data, &assets, *lang),
                    Panel::Kit => settings::gallery(body, *lang),
                }
            });
            #[cfg(not(tbg_desktop))]
            root.spawn(kit::muted(
                format!(
                    "{} · v{} · {}",
                    lang.key("native.crossplay"),
                    env!("CARGO_PKG_VERSION"),
                    lang.key("native.controls")
                ),
                12.,
            ));
        });
    let _ = form;
}

#[cfg(not(tbg_desktop))]
fn navigation(
    root: &mut ChildSpawnerCommands,
    lang: Lang,
    panel: Panel,
    data: &data::Data,
    compact: bool,
    assets: &AssetServer,
) {
    root.spawn((kit::row(), BackgroundColor(kit::PANEL)))
        .with_children(|row| {
            if compact {
                row.spawn(kit::heading("TBG", 22.));
            } else {
                row.spawn(kit::image(assets, "lobby/wordmark.webp", 210., 84.));
            }
            row.spawn(Node {
                flex_grow: 1.,
                ..default()
            });
            for (key, p) in [
                ("lobby.tabs.lobby", Panel::Lobby),
                ("lobby.tabs.arcanas", Panel::Arcanas),
                ("lobby.tabs.stronghold", Panel::Strongholds),
                ("lobby.tabs.arenas", Panel::Arenas),
                ("lobby.tabs.settings", Panel::Settings),
            ] {
                if !compact || [Panel::Lobby, Panel::Settings].contains(&p) {
                    row.spawn(button(lang.key(key), Action::Open(p), 80.))
                        .insert(BorderColor::all(if p == panel {
                            kit::GOLD
                        } else {
                            kit::LINE
                        }));
                }
            }
            let online = data.live.as_ref().map_or_else(
                || lang.key("live.chip_loading"),
                |v| {
                    lang.key("live.chip")
                        .replace("{online}", &v.players.to_string())
                        .replace("{battle}", &v.battle.to_string())
                },
            );
            row.spawn(button(online, Action::Open(Panel::Live), 120.));
        });
}
