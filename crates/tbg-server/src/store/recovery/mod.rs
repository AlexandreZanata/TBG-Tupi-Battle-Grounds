//! Private email identity, purpose-bound challenges and in-place key rotation.

pub mod challenge;
pub mod email;
pub mod outbox;
pub mod service;
use super::AccountId;
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Error {
    Denied,
    TrustedChangeRequired,
    Unavailable,
    EmailInvalid,
    EmailBlocked,
    EmailLimit,
    RegistrationClosed,
    Conflict,
    RateLimited,
}
impl From<rusqlite::Error> for Error {
    fn from(_: rusqlite::Error) -> Self {
        Self::Unavailable
    }
}
#[cfg(test)]
mod challenges;
#[cfg(test)]
mod fixture;
#[cfg(test)]
mod limits;
#[cfg(test)]
pub(crate) mod mail;
#[cfg(test)]
pub(crate) mod proof;
#[cfg(test)]
pub(crate) use fixture::{Accepted, Recovery};
pub mod rotate;
#[cfg(test)]
pub(crate) mod tests;
