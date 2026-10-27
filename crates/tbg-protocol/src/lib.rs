//! `tbg-protocol`: versioned message contracts between client and server.
//!
//! Phase 00 carries the protocol version handshake only. Intention and
//! snapshot schemas arrive with authoritative multiplayer (Phase 04).

/// Current protocol version. Mismatched peers are rejected, never
/// silently interpreted (see `docs/architecture/NETWORKING.md`).
pub const PROTOCOL_VERSION: u16 = 0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_starts_unversioned_at_zero() {
        assert_eq!(PROTOCOL_VERSION, 0);
    }
}
