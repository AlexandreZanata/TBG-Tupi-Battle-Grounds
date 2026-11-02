//! Operator-only AdminKey codec, independent from the player AccountKey.
use sha2::{Digest, Sha256};
const ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
pub fn generate() -> Result<(String, [u8; 32]), &'static str> {
    let mut bytes = [0; 32];
    getrandom::fill(&mut bytes).map_err(|_| "entropy")?;
    let secret: String = bytes
        .iter()
        .map(|b| ALPHABET[usize::from(b & 31)] as char)
        .collect();
    let groups = secret
        .as_bytes()
        .chunks(4)
        .map(|s| String::from_utf8_lossy(s))
        .collect::<Vec<_>>()
        .join("-");
    Ok((
        format!("TBGADM-{groups}"),
        Sha256::digest(secret.as_bytes()).into(),
    ))
}
pub fn parse(value: &str) -> Option<[u8; 32]> {
    if value.len() > 80 || !value.is_ascii() {
        return None;
    }
    let upper = value.to_ascii_uppercase();
    let raw = upper.strip_prefix("TBGADM-")?;
    let mut secret = String::new();
    for mut b in raw.bytes() {
        if b == b'-' || b == b' ' {
            continue;
        }
        b = match b {
            b'O' => b'0',
            b'I' | b'L' => b'1',
            _ => b,
        };
        if !ALPHABET.contains(&b) {
            return None;
        }
        secret.push(b as char);
    }
    (secret.len() == 32).then(|| Sha256::digest(secret.as_bytes()).into())
}
pub fn random<const N: usize>() -> Result<String, &'static str> {
    let mut b = [0; N];
    getrandom::fill(&mut b).map_err(|_| "entropy")?;
    Ok(b.iter().map(|v| format!("{v:02x}")).collect())
}
pub fn hash(value: &str) -> [u8; 32] {
    Sha256::digest(value.as_bytes()).into()
}
