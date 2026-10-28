//! Accounts (BR-P01, BR-P02): unique `PlayerName` checks and `AccountKey`
//! parsing/formatting. No I/O, no randomness here; the server supplies the
//! 10 random bytes. The secret is never stored — only its SHA-256 (server).

use crate::protocol::AccountError;

pub const MAX_NAME_LEN: usize = 16;
pub const MAX_KEY_LEN: usize = 40;

/// Names that are never allowed (compared lowercased).
pub const RESERVED_NAMES: &[&str] = &[
    "admin",
    "administrator",
    "mod",
    "moderator",
    "system",
    "server",
    "tbg",
    "tupi",
    "support",
    "staff",
    "official",
    "owner",
    "commander",
    "guest",
    "null",
    "root",
    "bot",
    "player",
];

// ponytail: 30 common English and Brazilian-Portuguese profanity/slur
// roots, matched as substrings after leetspeak normalization. Short roots
// over-block (any name containing them); the owner must review this list
// (P11 report) before launch.
pub const BLOCKED_SUBSTRINGS: &[&str] = &[
    "fuck",
    "shit",
    "dick",
    "pussy",
    "cunt",
    "whore",
    "slut",
    "bitch",
    "bastard",
    "asshole",
    "nazi",
    "hitler",
    "porn",
    "pedo",
    "rape",
    "retard",
    "porra",
    "caralho",
    "merda",
    "puta",
    "foda",
    "bosta",
    "corno",
    "viado",
    "arrombado",
    "babaca",
    "idiota",
    "imbecil",
    "otario",
    "vagabunda",
];

/// BR-P01: 3–16 ASCII letters/digits/`_`, starts with a letter, not
/// reserved, no blocked substring (leetspeak-normalized). Uniqueness is
/// enforced by the store (`NameTaken` on conflict).
pub fn check_name(name: &str) -> Result<(), AccountError> {
    if name.len() < 3 || name.len() > MAX_NAME_LEN || !name.is_ascii() {
        return Err(AccountError::NameInvalid);
    }
    let bytes = name.as_bytes();
    if !bytes[0].is_ascii_alphabetic()
        || !bytes
            .iter()
            .all(|b| b.is_ascii_alphanumeric() || *b == b'_')
    {
        return Err(AccountError::NameInvalid);
    }
    let lower = name.to_ascii_lowercase();
    if RESERVED_NAMES.contains(&lower.as_str()) {
        return Err(AccountError::NameInvalid);
    }
    if BLOCKED_SUBSTRINGS
        .iter()
        .any(|s| normalize(&lower).contains(s))
    {
        return Err(AccountError::NameInvalid);
    }
    Ok(())
}

/// Map leetspeak to letters before the blocked-substring check.
fn normalize(lower: &str) -> String {
    lower
        .chars()
        .map(|c| match c {
            '0' => 'o',
            '1' => 'i',
            '3' => 'e',
            '4' => 'a',
            '5' => 's',
            '7' => 't',
            _ => c,
        })
        .collect()
}

/// Crockford base32 without `O`, `I`, `L` (no look-alike secrets).
const ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// 10 random bytes (80 bits) → 16 secret characters.
pub fn encode_secret(random: &[u8; 10]) -> [u8; 16] {
    let mut out = [0u8; 16];
    let mut acc: u16 = 0;
    let mut bits = 0u8;
    let mut j = 0;
    for &b in random {
        acc = (acc << 8) | u16::from(b);
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            out[j] = ALPHABET[((acc >> bits) & 31) as usize];
            j += 1;
        }
    }
    debug_assert!(bits == 0 && j == 16);
    out
}

/// `<Name>-XXXX-XXXX-XXXX-XXXX` from the name and a 16-char secret.
pub fn format_key(name: &str, secret: &[u8; 16]) -> String {
    let s = secret
        .chunks(4)
        .map(|c| String::from_utf8_lossy(c).into_owned())
        .collect::<Vec<_>>()
        .join("-");
    format!("{name}-{s}")
}

/// Split at the first `-`; the secret is the rest without `-`, uppercased,
/// with `O→0`, `I→1`, `L→1`; it must be exactly 16 alphabet characters.
/// Returns the typed name and the 16 secret ASCII bytes (what gets hashed).
pub fn parse_key(key: &str) -> Option<(String, [u8; 16])> {
    if key.len() > MAX_KEY_LEN {
        return None;
    }
    let (_, rest) = key.split_once('-')?;
    let mut secret = [0u8; 16];
    let mut j = 0;
    for c in rest.chars().filter(|c| *c != '-') {
        if j >= 16 {
            return None;
        }
        let upper = c.to_ascii_uppercase() as u8;
        let fixed = match upper {
            b'O' => b'0',
            b'I' | b'L' => b'1',
            _ => upper,
        };
        if !ALPHABET.contains(&fixed) {
            return None;
        }
        secret[j] = fixed;
        j += 1;
    }
    if j != 16 {
        return None;
    }
    let name = key.split_once('-').map(|(n, _)| n.to_string())?;
    if name.is_empty() {
        return None;
    }
    Some((name, secret))
}
