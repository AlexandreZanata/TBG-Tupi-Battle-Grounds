//! Bounded in-memory delivery capture; no mail transport or persisted bearer links.

use super::Error;

// Proposed fixture capacity, not a production delivery/retention policy.
const CAPACITY: usize = 64;

pub(crate) enum Message {
    Verify(String),
    Recover(String),
    KeyRotated,
}

// Intentionally no Debug/Serialize: secrets must not escape through diagnostics.
pub(crate) struct CapturedMail {
    pub(crate) to: String,
    pub(crate) message: Message,
}

#[derive(Default)]
pub(crate) struct Capture {
    pub(crate) messages: Vec<CapturedMail>,
    pub(crate) unavailable: bool,
}

impl Capture {
    pub(super) fn reserve(&self) -> Result<(), Error> {
        if self.unavailable || self.messages.len() >= CAPACITY {
            return Err(Error::Unavailable);
        }
        Ok(())
    }

    pub(super) fn deliver(&mut self, to: String, message: Message) {
        // The single-threaded fixture reserves before committing any mutation.
        self.messages.push(CapturedMail { to, message });
    }
}
