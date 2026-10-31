//! Shared lobby navigation and screen lifecycle; gameplay uses existing UiActions.

mod arcanas;
mod arenas;
mod data;
mod input;
mod live;
mod lobby;
mod settings;
mod strongholds;

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
    Kit,
}
#[derive(Component, Clone, Copy, Debug)]
pub enum Action {
    Open(Panel),
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

pub fn plugin(app: &mut App) {
    app.init_resource::<Shell>()
        .init_resource::<data::Data>()
        .add_systems(
            Update,
            (data::poll, input::mouse_and_keys, room_text, render)
                .chain()
                .run_if(in_state(GameState::Menu)),
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
) {
    for event in events.read().filter(|e| e.kind == TextKind::Room) {
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

#[allow(clippy::too_many_arguments)]
fn render(
    mut commands: Commands,
    mut shell: ResMut<Shell>,
    data: Res<data::Data>,
    assets: Res<AssetServer>,
    account: Res<Account>,
    ranking: Res<Ranking>,
    lang: Res<Lang>,
    sounds: Res<Sounds>,
    form: NonSend<ActiveForm>,
    roots: Query<Entity, With<Root>>,
    window: Single<&Window>,
    mut hidden: Local<(bool, bool)>,
    bodies: Query<&ScrollPosition, With<Body>>,
    mut layout: Local<(Panel, Vec2)>,
) {
    if !crate::config::bevy_ui() || crate::config::arena_preview() {
        return;
    }
    let modal = ranking.open || matches!(account.screen, AccountScreen::SaveKey { .. });
    if !shell.dirty
        && !account.is_changed()
        && !lang.is_changed()
        && sounds.muted == hidden.1
        && modal == hidden.0
        && layout.1 == Vec2::new(window.width(), window.height())
        && !roots.is_empty()
    {
        return;
    }
    let scroll = if layout.0 == shell.panel {
        bodies
            .iter()
            .next()
            .map(|p| ScrollPosition(p.0))
            .unwrap_or_default()
    } else {
        ScrollPosition::default()
    };
    *layout = (shell.panel, Vec2::new(window.width(), window.height()));
    shell.dirty = false;
    *hidden = (modal, sounds.muted);
    for e in &roots {
        commands.entity(e).despawn();
    }
    if modal {
        return;
    }
    let compact = window.width() < 900.;
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(px(if compact { 10 } else { 24 })),
                row_gap: px(16),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.035, 0.045, 0.72)),
            Root,
            DespawnOnExit(GameState::Menu),
        ))
        .with_children(|root| {
            navigation(root, *lang, shell.panel, &data, compact, &assets);
            if let Some(version) = &data.update {
                root.spawn(button(
                    format!("{} · {version}", lang.key("native.update")),
                    Action::Download,
                    320.,
                ));
            }
            root.spawn((
                Node {
                    flex_grow: 1.,
                    min_height: px(0),
                    width: percent(100),
                    overflow: Overflow::scroll_y(),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(16),
                    ..default()
                },
                scroll,
                Body,
            ))
            .with_children(|body| match shell.panel {
                Panel::Lobby => {
                    lobby::spawn(body, &shell, &data, &account, &assets, *lang, compact)
                }
                Panel::Friends => lobby::friends(body, *lang),
                Panel::Settings => settings::spawn(body, &account, &sounds, &window, *lang),
                Panel::Arenas => arenas::spawn(body, &assets, *lang),
                Panel::Strongholds => strongholds::spawn(body, &assets, *lang),
                Panel::Arcanas => arcanas::spawn(body, &shell, &assets, *lang, compact),
                Panel::Live => live::spawn(body, &data, *lang),
                Panel::Kit => settings::gallery(body, *lang),
            });
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
                row.spawn(kit::image(assets, "branding/tbg-emblem.webp", 70., 70.));
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
