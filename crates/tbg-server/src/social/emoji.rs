//! Frozen three-slot defaults and metadata bounds for cosmetic emoji delivery.
use tbg_core::protocol::{EmojiRef, EMOJI_SLOTS};
pub fn defaults() -> [EmojiRef; EMOJI_SLOTS] {
    std::array::from_fn(|slot| EmojiRef::Standard { id: slot as u8 })
}
