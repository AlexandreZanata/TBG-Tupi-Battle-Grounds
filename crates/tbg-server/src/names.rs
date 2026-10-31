//! Server-generated display names and random bytes (no accounts, no
//! user-chosen names: nothing to moderate, decision D-09).

pub fn random_bytes<const N: usize>() -> [u8; N] {
    let mut buf = [0u8; N];
    // The OS RNG only fails on broken platforms; a zeroed buffer would
    // still be a unique-enough fallback for display names, never for
    // tokens, so treat failure as fatal.
    getrandom::fill(&mut buf).expect("OS random number generator unavailable");
    buf
}

pub fn display_name() -> String {
    let [a, b] = random_bytes::<2>();
    format!(
        "Commander {}",
        (u16::from(a) << 8 | u16::from(b)) % 9_000 + 1_000
    )
}

/// A failed entropy source cannot create a predictable resume handle or panic a handshake.
pub fn session_identity() -> Option<(tbg_core::protocol::Token, String)> {
    let mut bytes = [0; 18];
    getrandom::fill(&mut bytes).ok()?;
    let mut token = [0; 16];
    token.copy_from_slice(&bytes[..16]);
    let suffix = u16::from_le_bytes([bytes[16], bytes[17]]) % 9_000 + 1_000;
    Some((token, format!("Commander {suffix}")))
}
