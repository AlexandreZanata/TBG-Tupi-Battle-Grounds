//! Seat-only social presentation and bounded frozen evidence, never simulation state.
use super::*;
use serde_json::{json, Value};
use tbg_core::protocol::{EmojiRef, EMOJI_SLOTS};
pub(super) struct Social {
    pub transcript: Vec<Value>,
    pub bytes: usize,
    pub loadouts: [[EmojiRef; EMOJI_SLOTS]; 2],
}
impl Default for Social {
    fn default() -> Self {
        Self {
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
