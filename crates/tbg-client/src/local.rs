//! Practice match: `tbg-core` stepped in the browser, the bot on seat B.
//! Produces exactly the `ServerMsg`s the real server sends, so the rest of
//! the client cannot tell practice from online play.

use tbg_core::bot::Bot;
use tbg_core::protocol::{ClientMsg, ServerMsg};
use tbg_core::{ArcanaPreset, Event, MatchState, Phase, PlayerIntention, Seat, StrongholdKind};

pub struct LocalMatch {
    state: MatchState,
    bot: Bot,
    queue: Vec<(u32, PlayerIntention)>,
    events: Vec<Event>,
    outbox: Vec<ServerMsg>,
    ended: bool,
}

impl LocalMatch {
    /// Practice uses the player's saved Arcanas and Stronghold against a
    /// default bot (D-40) whose kind follows the seed, so a practice match
    /// replays exactly (BR-SK09). `army` is the P30-A debug parameter: top
    /// both sides up to N units before the first view; `None` is the normal
    /// practice match.
    pub fn new(
        seed: u64,
        arcanas: ArcanaPreset,
        stronghold: StrongholdKind,
        army: Option<usize>,
    ) -> LocalMatch {
        let presets = [arcanas, ArcanaPreset::DEFAULT];
        let selectable = StrongholdKind::SELECTABLE;
        let bot_kind = selectable[(seed % selectable.len() as u64) as usize];
        let mut state =
            MatchState::with_preview(seed, presets).with_strongholds([stronghold, bot_kind]);
        if let Some(n) = army {
            while state.phase != Phase::Running {
                state.step();
            }
            let mut next = [0usize; 2];
            for seat in Seat::BOTH {
                while state.army_size(seat) < n {
                    let kind =
                        tbg_core::UNIT_KINDS[next[seat.index()] % tbg_core::UNIT_KINDS.len()];
                    next[seat.index()] += 1;
                    state.players[seat.index()].offers[0] = kind;
                    state.players[seat.index()].energy = 6_000;
                    if state
                        .apply_intention(seat, 0, PlayerIntention::Purchase { offer: 0 })
                        .is_err()
                    {
                        break;
                    }
                }
            }
        }
        LocalMatch {
            state,
            bot: Bot::new(seed ^ 0x5eed),
            queue: Vec::new(),
            events: Vec::new(),
            outbox: vec![ServerMsg::MatchStart {
                you: Seat::A,
                you_name: "You".into(),
                you_rating: None,
                opponent: "Training Bot".into(),
                mode: tbg_core::protocol::MatchMode::Casual,
                opponent_rating: None,
                arcanas: presets,
            }],
            ended: false,
        }
    }

    pub fn send(&mut self, msg: ClientMsg) {
        if let ClientMsg::Intent { seq, intent } = msg {
            self.queue.push((seq, intent));
        }
    }

    pub fn take_outbox(&mut self) -> Vec<ServerMsg> {
        std::mem::take(&mut self.outbox)
    }

    /// One 50 ms tick: player intents, step, views every 2nd tick.
    pub fn tick(&mut self) {
        if self.ended {
            return;
        }
        for (seq, intent) in std::mem::take(&mut self.queue) {
            if let Err(reason) = self.state.apply_intention(Seat::A, seq, intent) {
                self.outbox.push(ServerMsg::Rejected { seq, reason });
            }
        }
        self.events.extend(self.state.step());
        let over = self.state.outcome.is_some();
        if self.state.clock.is_multiple_of(2) || over {
            let view = self.state.view_for(Seat::A, &self.events);
            self.outbox.push(ServerMsg::View(Box::new(view)));
            let bot_view = self.state.view_for(Seat::B, &self.events);
            if let Some(intent) = self.bot.decide(&bot_view) {
                let _ = self
                    .state
                    .apply_intention(Seat::B, self.state.clock, intent);
            }
            self.events.clear();
        }
        if let Some(outcome) = self.state.outcome {
            self.outbox.push(ServerMsg::MatchEnd { outcome });
            self.ended = true;
        }
    }
}
