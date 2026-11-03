//! Seat-only social presentation and bounded frozen evidence, never simulation state.
use super::*;
use serde_json::{json, Value};
use tbg_core::protocol::{EmojiRef, EMOJI_SLOTS};
pub(super) struct Social {
    pub config: Option<Arc<Config>>,
    pub verified: [bool; 2],
    pub limits: [crate::social::limits::Limits; 2],
    pub started: Instant,
    pub transcript: Vec<Value>,
    pub bytes: usize,
    pub loadouts: [[EmojiRef; EMOJI_SLOTS]; 2],
}
impl Default for Social {
    fn default() -> Self {
        Self {
            config: None,
            verified: [false; 2],
            limits: Default::default(),
            started: Instant::now(),
            transcript: Vec::new(),
            bytes: 2,
            loadouts: std::array::from_fn(|_| {
                std::array::from_fn(|slot| EmojiRef::Standard { id: slot as u8 })
            }),
        }
    }
}
impl Match {
    pub(super) fn social_record(&self, store: &Store, ended: i64, duration: u64) {
        let accounts = self.players.each_ref().map(|p| p.account.map(|(id, _)| id));
        if accounts.iter().all(Option::is_none) {
            return;
        }
        store.social_record(crate::store::social::Record{result:self.result_id,ended,mode:match self.mode{MatchMode::Ranked=>"ranked",MatchMode::Challenge=>"challenge",_=>"casual"}.into(),accounts,names:self.players.each_ref().map(|p|p.name.clone()),outcome:json!({"winner":self.state.outcome.and_then(|o|o.winner).map(|s|s.index()),"reason":self.end_detail.unwrap_or(EndDetail::Stronghold).as_str(),"duration_s":duration,"arena":self.state.arena.index()}),loadouts:serde_json::to_value(&self.social.loadouts).unwrap_or(Value::Null),transcript:json!(self.social.transcript)});
    }
}

impl Match {
    pub(super) fn social_rights(&self, seat: Seat) -> tbg_core::protocol::MatchSocial {
        use tbg_core::protocol::ChatPermission as P;
        let account = self.players[seat.index()].account.map(|(id, _)| id);
        let config = self.social.config.as_ref();
        let now = config.and_then(|c| c.auth_clock.now().ok()).unwrap_or(0);
        let cache = config.map(|c| &c.moderation);
        let state = account
            .map(|id| cache.map(|c| c.state(id)).unwrap_or_default())
            .unwrap_or_default();
        let chat = if !cache.is_some_and(|c| c.enabled("chat_enabled")) {
            P::Disabled
        } else if account.is_none() {
            P::Guest
        } else if !self.social.verified[seat.index()] {
            P::Unverified
        } else if let Some(until) = state.chat_until.filter(|u| *u > now) {
            P::Restricted {
                until_unix: until as u64,
            }
        } else {
            P::Allowed
        };
        let emojis = cache.is_some_and(|c| c.enabled("emoji_enabled"))
            && !state.emoji_until.is_some_and(|u| u > now);
        let custom = cache.is_some_and(|c| c.enabled("custom_emoji_enabled"));
        let mut loadouts = self.social.loadouts.clone();
        for slots in &mut loadouts {
            for (slot, emoji) in slots.iter_mut().enumerate() {
                if matches!(emoji,EmojiRef::Custom{id,..}if !custom||!cache.is_some_and(|c|c.approved(id)))
                {
                    *emoji = EmojiRef::Standard { id: slot as u8 };
                }
            }
        }
        tbg_core::protocol::MatchSocial {
            chat,
            emojis,
            custom_emojis: custom,
            loadouts,
        }
    }
    pub(super) fn send_social(&mut self, seat: Seat) {
        self.send(
            seat,
            tbg_core::protocol::ServerMsg::MatchSocial(Box::new(self.social_rights(seat))),
        );
    }
    pub(super) fn social_policy(&mut self) {
        for seat in Seat::BOTH {
            self.send_social(seat);
        }
    }
    pub(super) fn social_chat(&mut self, seat: Seat, conn_id: u64, text: String) {
        use tbg_core::protocol::{ChatPermission as P, ServerMsg, SocialRefusal as R};
        if !self.owns_connection(seat, conn_id) {
            return;
        }
        let result = (|| {
            match self.social_rights(seat).chat {
                P::Allowed => {}
                P::Disabled => return Err(R::Disabled),
                _ => return Err(R::Restricted),
            }
            let text = crate::social::chat::sanitize(&text)?;
            let ms = self
                .social
                .started
                .elapsed()
                .as_millis()
                .min(u128::from(u64::MAX)) as u64;
            self.social.limits[seat.index()].chat(&text, ms)?;
            let terms = self
                .social
                .config
                .as_ref()
                .map(|c| c.moderation.terms())
                .unwrap_or_default();
            Ok(crate::social::chat::mask(&text, &terms))
        })();
        match result {
            Ok(text) => {
                Metrics::add(&self.metrics.chat_sent, 1);
                self.transcript(json!({"from":seat.index(),"text":text}));
                for recipient in Seat::BOTH {
                    self.send(
                        recipient,
                        ServerMsg::Chat {
                            from: seat,
                            text: text.clone(),
                        },
                    );
                }
            }
            Err(reason) => {
                Metrics::add(&self.metrics.chat_refused, 1);
                self.send(seat, ServerMsg::SocialRefused { reason });
            }
        }
    }
    pub(super) fn social_emoji(&mut self, seat: Seat, conn_id: u64, slot: u8) {
        use tbg_core::protocol::{ServerMsg, SocialRefusal as R};
        if !self.owns_connection(seat, conn_id) {
            return;
        }
        let result = if slot >= 3 {
            Err(R::NotAvailable)
        } else if !self.social_rights(seat).emojis {
            Err(R::Restricted)
        } else {
            let ms = self
                .social
                .started
                .elapsed()
                .as_millis()
                .min(u128::from(u64::MAX)) as u64;
            self.social.limits[seat.index()].emoji(ms)
        };
        match result {
            Ok(()) => {
                Metrics::add(&self.metrics.emoji_sent, 1);
                self.transcript(json!({"from":seat.index(),"slot":slot}));
                for recipient in Seat::BOTH {
                    self.send(recipient, ServerMsg::Emoji { from: seat, slot });
                }
            }
            Err(reason) => self.send(seat, ServerMsg::SocialRefused { reason }),
        }
    }
    fn transcript(&mut self, entry: Value) {
        let size = entry.to_string().len() + 1;
        if self.social.transcript.len() < 200 && self.social.bytes + size <= 32768 {
            self.social.bytes += size;
            self.social.transcript.push(entry);
        }
    }
}
