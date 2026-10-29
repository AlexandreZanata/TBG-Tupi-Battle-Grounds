//! `?autoplay[=quick|practice]`: the `tbg-core` bot plays for this client
//! through the normal `MatchLink` path. With a mode it also starts that
//! match from the menu. Dev and validation aid (two-browser matches).

use bevy::prelude::*;
use tbg_core::bot::Bot;
use tbg_core::protocol::ClientMsg;

use crate::config;
use crate::link::MatchLink;
use crate::sync::LatestView;
use crate::ui::{ActionQueue, UiAction};
use crate::GameState;

#[derive(Resource)]
struct Autoplay {
    bot: Bot,
    last_tick: u32,
    seq: u32,
}

pub fn plugin(app: &mut App) {
    if config::url_param("autoplay").is_some() {
        app.insert_resource(Autoplay {
            bot: Bot::new(0xa070),
            last_tick: u32::MAX,
            seq: 1_000_000,
        })
        .add_systems(Update, play)
        .add_systems(OnEnter(GameState::Menu), start);
    }
}

/// Start the requested mode whenever the menu shows.
fn start(mut queue: ResMut<ActionQueue>) {
    match config::url_param("autoplay").as_deref() {
        Some("quick") => queue.0.push(UiAction::Quick),
        Some("practice") => queue.0.push(UiAction::Practice),
        _ => {}
    }
}

fn play(
    time: Res<Time>,
    mut auto: ResMut<Autoplay>,
    view: Res<LatestView>,
    mut link: NonSendMut<MatchLink>,
) {
    let Some(v) = &view.0 else { return };
    if v.clock == auto.last_tick {
        return;
    }
    auto.last_tick = v.clock;
    if let Some(intent) = auto.bot.decide(v) {
        auto.seq += 1;
        let seq = auto.seq;
        link.send(ClientMsg::Intent { seq, intent }, time.elapsed_secs_f64());
    }
}
