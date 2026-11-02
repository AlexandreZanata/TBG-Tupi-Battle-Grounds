//! Presentation-only social metadata; never included in simulation or replay state.
use serde::{Deserialize, Serialize};
pub const CHAT_MAX_CHARS: usize = 120;
pub const EMOJI_SLOTS: usize = 3;
pub const STANDARD_EMOJIS: u8 = 16;
pub const CUSTOM_EMOJI_MAX_FRAMES: u8 = 24;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmojiRef {
    Standard {
        id: u8,
    },
    Custom {
        id: [u8; 16],
        frame_px: u8,
        frames: u8,
        columns: u8,
        delays_ms: Vec<u16>,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChatPermission {
    Allowed,
    Disabled,
    Guest,
    Unverified,
    Restricted { until_unix: u64 },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchSocial {
    pub chat: ChatPermission,
    pub emojis: bool,
    pub custom_emojis: bool,
    pub loadouts: [[EmojiRef; EMOJI_SLOTS]; 2],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SocialRefusal {
    RateLimited,
    Duplicate,
    InvalidText,
    Restricted,
    Disabled,
    NotAvailable,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SuspensionReason {
    Cheating,
    AbusiveChat,
    OffensiveName,
    OffensiveEmoji,
    Harassment,
    BanEvasion,
    Impersonation,
    Other,
}
impl SuspensionReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cheating => "cheating",
            Self::AbusiveChat => "abusive_chat",
            Self::OffensiveName => "offensive_name",
            Self::OffensiveEmoji => "offensive_emoji",
            Self::Harassment => "harassment",
            Self::BanEvasion => "ban_evasion",
            Self::Impersonation => "impersonation",
            Self::Other => "other",
        }
    }
    pub fn parse(value: &str) -> Option<Self> {
        [
            Self::Cheating,
            Self::AbusiveChat,
            Self::OffensiveName,
            Self::OffensiveEmoji,
            Self::Harassment,
            Self::BanEvasion,
            Self::Impersonation,
            Self::Other,
        ]
        .into_iter()
        .find(|v| v.as_str() == value)
    }
}
