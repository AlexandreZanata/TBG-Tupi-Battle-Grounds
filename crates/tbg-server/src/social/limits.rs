//! Per-seat monotonic social budgets with bounded retained history.
use std::collections::VecDeque;
use tbg_core::protocol::SocialRefusal;
#[derive(Default)]
pub struct Limits {
    chat: VecDeque<(u64, String)>,
    emoji: VecDeque<u64>,
}
impl Limits {
    pub fn chat(&mut self, text: &str, ms: u64) -> Result<(), SocialRefusal> {
        while self
            .chat
            .front()
            .is_some_and(|(at, _)| ms.saturating_sub(*at) >= 60000)
        {
            self.chat.pop_front();
        }
        if self.chat.len() >= 20
            || self
                .chat
                .iter()
                .filter(|(at, _)| ms.saturating_sub(*at) < 5000)
                .count()
                >= 3
        {
            return Err(SocialRefusal::RateLimited);
        }
        if self
            .chat
            .iter()
            .any(|(at, t)| ms.saturating_sub(*at) < 10000 && t == text)
        {
            return Err(SocialRefusal::Duplicate);
        }
        self.chat.push_back((ms, text.into()));
        Ok(())
    }
    pub fn emoji(&mut self, ms: u64) -> Result<(), SocialRefusal> {
        while self
            .emoji
            .front()
            .is_some_and(|at| ms.saturating_sub(*at) >= 30000)
        {
            self.emoji.pop_front();
        }
        if self.emoji.len() >= 8
            || self
                .emoji
                .back()
                .is_some_and(|at| ms.saturating_sub(*at) < 2000)
        {
            return Err(SocialRefusal::RateLimited);
        }
        self.emoji.push_back(ms);
        Ok(())
    }
}
