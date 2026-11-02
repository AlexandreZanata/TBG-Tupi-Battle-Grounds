//! Practice match: shared simulation with independently drawn human/bot sides.
//! Produces exactly the `ServerMsg`s the real server sends, so the rest of
//! the client cannot tell practice from online play.

use tbg_core::practice_bot::PracticeBot;
use tbg_core::protocol::{ClientMsg, ServerMsg};
use tbg_core::{ArcanaPreset, Event, MatchState, Phase, PlayerIntention, Seat, StrongholdKind};

pub struct LocalMatch {
    you: Seat,
    result_id: tbg_core::protocol::ResultId,
    state: MatchState,
    bot: PracticeBot,
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
        arena: tbg_core::Arena,
    ) -> LocalMatch {
        let difficulty = crate::platform::storage::get(
            crate::platform::storage::Scope::Local,
            "practice.difficulty",
        )
        .and_then(|s| s.parse::<u8>().ok())
        .unwrap_or(1)
        .min(3);
        let you = practice_side(seed);
        let mut presets = [ArcanaPreset::DEFAULT; 2];
        presets[you.index()] = arcanas;
        let selectable = StrongholdKind::SELECTABLE;
        let bot_kind = selectable[(seed % selectable.len() as u64) as usize];
        let mut kinds = [bot_kind; 2];
        kinds[you.index()] = stronghold;
        let mut state = MatchState::with_preview(seed, presets)
            .with_strongholds(kinds)
            .with_arena(arena);
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
        #[cfg(all(any(debug_assertions, feature = "ui-acceptance"), tbg_native_ui))]
        if crate::config::url_param("capture").is_some()
            && crate::config::url_param("capture_panel").as_deref() == Some("council")
        {
            // Advance the real local simulation to its first War Council for UI capture.
            while !state.augments.selecting() && state.phase != Phase::Ended {
                state.step();
            }
        }
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let mut result_id = [0u8; 16];
        result_id[..8].copy_from_slice(
            &NEXT
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                .to_le_bytes(),
        );
        result_id[15] = 1;
        LocalMatch {
            you,
            result_id,
            state,
            bot: PracticeBot::new(seed ^ 0x5eed, difficulty),
            queue: Vec::new(),
            events: Vec::new(),
            outbox: vec![ServerMsg::MatchStart {
                result_id,
                you,
                you_name: "You".into(),
                you_rating: None,
                opponent: if difficulty == 3 {
                    "Impossible · Bot Energy ×2".into()
                } else {
                    format!(
                        "Training Bot · {}",
                        ["Easy", "Medium", "Hard"][usize::from(difficulty)]
                    )
                },
                mode: tbg_core::protocol::MatchMode::Casual,
                opponent_rating: None,
                arcanas: presets,
                // ARENAS-G: the player's Practice choice; same rules as online.
                arena,
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
            if let Err(reason) = self.state.apply_intention(self.you, seq, intent) {
                self.outbox.push(ServerMsg::Rejected { seq, reason });
            }
        }
        let bot = self.you.other();
        let bonus = self.bot.regen_bonus_for(&self.state, bot);
        self.events.extend(self.state.step());
        self.state.players[bot.index()].energy =
            self.state.players[bot.index()].energy.saturating_add(bonus);
        let over = self.state.outcome.is_some();
        if self.state.clock.is_multiple_of(2) || over {
            let view = self.state.view_for(self.you, &self.events);
            self.outbox.push(ServerMsg::View(Box::new(view)));
            let bot_view = self.state.view_for(bot, &self.events);
            if let Some(intent) = self.bot.decide(&bot_view) {
                let _ = self.state.apply_intention(bot, self.state.clock, intent);
            }
            self.events.clear();
        }
        if let Some(outcome) = self.state.outcome {
            self.outbox.push(ServerMsg::MatchEnd {
                result_id: self.result_id,
                last_tick: self.state.tick,
                cast: self
                    .state
                    .celebration_cast()
                    .as_ref()
                    .and_then(tbg_core::celebration::CastPayload::new),
                outcome,
            });
            self.ended = true;
        }
    }
}

fn practice_side(seed: u64) -> Seat {
    // A separate stream keeps the side draw independent of shops and bot choices.
    let mut rng = tbg_core::Pcg32::new(seed, 0x5349_4445);
    let drawn = if rng.below(2) == 0 { Seat::A } else { Seat::B };
    #[cfg(all(any(debug_assertions, feature = "ui-acceptance"), tbg_desktop))]
    if crate::config::url_param("capture").is_some() {
        return match crate::config::url_param("capture_side").as_deref() {
            Some("blue") => Seat::A,
            Some("red") => Seat::B,
            _ => drawn,
        };
    }
    drawn
}

#[cfg(test)]
mod side_tests {
    use super::*;

    #[test]
    fn side01_practice_draws_both_sides_and_keeps_human_commands_and_loadout() {
        let mut seen = [false; 2];
        let preset = ArcanaPreset::complete([2, 2, 2, 2, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0]).unwrap();
        for seed in 0..64 {
            let mut game = LocalMatch::new(
                seed,
                preset,
                StrongholdKind::Command,
                None,
                tbg_core::Arena::Forest,
            );
            let start = game.take_outbox();
            assert!(matches!(start[0], ServerMsg::MatchStart { you, .. } if you == game.you));
            assert_eq!(game.state.player(game.you).arcanas, preset);
            assert_eq!(
                game.state.player(game.you.other()).arcanas,
                ArcanaPreset::DEFAULT
            );
            seen[game.you.index()] = true;
            assert_eq!(
                game.state.player(game.you).stronghold,
                StrongholdKind::Command
            );
            while game.state.phase != Phase::Running {
                game.state.step();
            }
            // GD-04: the preview pool's dearest offer (Astreon) costs 23.
            game.state.players[game.you.index()].energy = 30 * tbg_core::content::ENERGY_SCALE;
            let energy = game.state.player(game.you).energy;
            game.send(ClientMsg::Intent {
                seq: 17,
                intent: PlayerIntention::Purchase { offer: 0 },
            });
            game.tick();
            assert!(game.state.player(game.you).energy < energy);
            assert!(game.state.units.iter().any(|u| u.seat == game.you));
            assert!(game.state.view_for(game.you, &[]).last_seq >= 17);
        }
        assert_eq!(seen, [true, true]);
    }
}
