//! Typed Crockford codes with principal/purpose binding and constant-shape digest comparison.

use sha2::{Digest, Sha256};
const ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

pub fn generate() -> Result<String, &'static str> {
    let mut bytes = [0; 10];
    getrandom::fill(&mut bytes).map_err(|_| "entropy")?;
    let raw: String = bytes
        .iter()
        .map(|b| ALPHABET[usize::from(b & 31)] as char)
        .collect();
    Ok(format!("{}-{}", &raw[..5], &raw[5..]))
}

pub fn parse(value: &str) -> Option<String> {
    if value.len() > 32 {
        return None;
    }
    let mut out = String::with_capacity(10);
    for mut b in value.bytes() {
        if b == b'-' || b == b' ' {
            continue;
        }
        b = b.to_ascii_uppercase();
        b = match b {
            b'O' => b'0',
            b'I' | b'L' => b'1',
            _ => b,
        };
        if !ALPHABET.contains(&b) {
            return None;
        }
        out.push(b as char);
    }
    (out.len() == 10).then_some(out)
}

pub fn digest(principal: i64, purpose: &str, code: &str) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(principal.to_string().as_bytes());
    hash.update([0]);
    hash.update(purpose.as_bytes());
    hash.update([0]);
    hash.update(code.as_bytes());
    hash.finalize().into()
}

pub fn equal(a: &[u8], b: &[u8]) -> bool {
    let mut difference = a.len() ^ b.len();
    for i in 0..32 {
        difference |= usize::from(a.get(i).copied().unwrap_or(0) ^ b.get(i).copied().unwrap_or(0));
    }
    difference == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ic15_codes_are_typed_and_purpose_bound() {
        let code = generate().unwrap();
        let raw = parse(&code).unwrap();
        assert_eq!(raw.len(), 10);
        assert_eq!(parse("ooooo-lllll").unwrap(), "0000011111");
        assert_ne!(digest(1, "verify", &raw), digest(1, "rotate", &raw));
        assert_ne!(digest(1, "verify", &raw), digest(2, "verify", &raw));
        assert!(!equal(&[0; 31], &[0; 32]));
    }
}
