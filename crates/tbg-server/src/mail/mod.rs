//! Transactional email delivery, isolated from game outcomes and logging.
pub mod dispatch;
pub mod settings;
pub mod templates;
pub mod transport;

#[cfg(test)]
mod tests;
