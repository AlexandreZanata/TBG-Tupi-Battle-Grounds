//! Offline practice: the authoritative core runs in-process with nine bots.

use bevy::prelude::*;
use tbg_moba_core::content::TICK_HZ;
use tbg_moba_core::state::Controller;
use tbg_moba_core::{host, view, MobaIntention, MobaMatch, Seat, Team};

use super::{Feed, Outbox, Status};
use crate::config::Launch;

const DT: f32 = 1.0 / TICK_HZ as f32;
/// Ticks a slow frame may catch up before the simulation slows instead.
const MAX_CATCH_UP: u32 = 4;

#[derive(Resource)]
pub struct LocalHost {
    game: MobaMatch,
    seat: Seat,
    inputs: Vec<(Seat, MobaIntention)>,
    pending: f32,
}

pub fn start(world: &mut World, launch: &Launch) {
    let seat = Seat::of(Team::Blue, launch.role);
    let mut game = MobaMatch::new(launch.seed);
    // A review warp lets the bot play the seat too, so the scene is lived-in.
    let mut none = Vec::new();
    for _ in 0..launch.warp_secs * TICK_HZ {
        host::tick(&mut game, &mut none);
    }
    game.set_controller(seat, Controller::Human);
    let now = world.resource::<Time>().elapsed_secs_f64();
    let mut feed = world.resource_mut::<Feed>();
    feed.seat = Some(seat);
    feed.status = Status::Live;
    feed.me = Some(view::private(&game, seat));
    feed.push(view::world(&game, seat.team()), now);
    world.insert_resource(LocalHost {
        game,
        seat,
        inputs: Vec::new(),
        pending: 0.0,
    });
}

/// Starts a fresh offline match with the same options.
pub fn restart(world: &mut World) {
    let mut launch = world.resource::<Launch>().clone();
    launch.seed = launch.seed.wrapping_add(1);
    launch.warp_secs = 0;
    world.insert_resource(launch.clone());
    *world.resource_mut::<Feed>() = Feed::default();
    start(world, &launch);
}

pub fn run(
    host: Option<ResMut<LocalHost>>,
    mut feed: ResMut<Feed>,
    mut outbox: ResMut<Outbox>,
    time: Res<Time>,
) {
    let Some(mut host) = host else { return };
    let host = &mut *host;
    let seat = host.seat;
    host.inputs.extend(outbox.0.drain(..).map(|i| (seat, i)));
    host.pending = (host.pending + time.delta_secs()).min(DT * MAX_CATCH_UP as f32);
    while host.pending >= DT {
        host.pending -= DT;
        let rejects = host::tick(&mut host.game, &mut host.inputs);
        feed.rejects.extend(rejects.into_iter().map(|(_, r)| r));
        feed.me = Some(view::private(&host.game, seat));
        feed.push(
            view::world(&host.game, seat.team()),
            time.elapsed_secs_f64(),
        );
    }
}
