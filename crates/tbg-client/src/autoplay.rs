//! `?autoplay`: the `tbg-core` bot plays for this client through the normal
//! `MatchLink` path (dev and validation aid; P4 uses it for two-browser
//! matches without clicking).

use bevy::prelude::*;
use tbg_core::bot::Bot;
use tbg_core::protocol::ClientMsg;

use crate::config;
use crate::link::MatchLink;
use crate::sync::LatestView;

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
        .add_systems(Update, play);
    }
}

fn play(mut auto: ResMut<Autoplay>, view: Res<LatestView>, mut link: NonSendMut<MatchLink>) {
    let Some(v) = &view.0 else { return };
    if v.tick == auto.last_tick {
        return;
    }
    auto.last_tick = v.tick;
    if let Some(intent) = auto.bot.decide(v) {
        auto.seq += 1;
        let seq = auto.seq;
        link.send(ClientMsg::Intent { seq, intent });
    }
}
