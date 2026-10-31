//! P66 initial bounded settings (GAME-DESIGN §19, Proposed): exceeding one
//! fails with a user-visible retry or unsupported message, never a bypass.

/// Signed metadata files (root/timestamp/snapshot/targets).
pub const METADATA_BYTES: u64 = 1024 * 1024;
/// A release descriptor or channel pointer target.
pub const DESCRIPTOR_BYTES: u64 = 64 * 1024;
/// Localized notes for one release.
pub const NOTES_BYTES: u64 = 512 * 1024;
/// A compressed release package.
pub const PACKAGE_BYTES: u64 = 2 * 1024 * 1024 * 1024;
/// All files of one extracted release.
pub const EXTRACTED_BYTES: u64 = 8 * 1024 * 1024 * 1024;
/// Entries in one release archive.
pub const ARCHIVE_ENTRIES: usize = 100_000;
/// Automatic retries after the first download attempt.
pub const DOWNLOAD_RETRIES: u32 = 2;
/// Startup discovery deadline (seconds).
pub const DISCOVERY_SECS: u64 = 5;
/// Background recheck interval while idle (seconds).
pub const RECHECK_SECS: u64 = 6 * 60 * 60;
/// A candidate build must report healthy within this many seconds.
pub const READINESS_SECS: u64 = 60;
/// Automatic notes history: highlights of at most this many releases.
pub const NOTES_HISTORY: usize = 10;
/// Per locale: notes items, characters per item and per title.
pub const NOTES_ITEMS: usize = 40;
pub const NOTES_ITEM_CHARS: usize = 280;
pub const NOTES_TITLE_CHARS: usize = 80;
/// Releases kept besides the current one (the verified fallback).
pub const KEEP_PREVIOUS: usize = 1;
