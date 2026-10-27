//! Client/server message contracts.
//!
//! Message enums (`ClientMsg`, `ServerMsg`) arrive with the online match
//! phase (docs/ROADMAP.md, Phase 5). Peers with a different version are
//! rejected during the handshake, never silently interpreted.

/// Current protocol version.
pub const PROTOCOL_VERSION: u16 = 0;
