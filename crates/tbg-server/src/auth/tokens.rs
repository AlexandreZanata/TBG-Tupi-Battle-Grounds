//! Fallible entropy, redacted encodings and injectable expiry clocks (D-97).

use sha2::Digest;
use std::sync::{
    atomic::{AtomicI64, Ordering},
    Arc,
};
use std::time::{SystemTime, UNIX_EPOCH};

use super::Error;

pub(super) fn random<const N: usize>() -> Result<[u8; N], Error> {
    let mut bytes = [0; N];
    getrandom::fill(&mut bytes).map_err(|_| Error::Unavailable)?;
    Ok(bytes)
}

pub(super) fn hash(bytes: &[u8]) -> [u8; 32] {
    sha2::Sha256::digest(bytes).into()
}

pub(super) fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(DIGITS[usize::from(b >> 4)] as char);
        out.push(DIGITS[usize::from(b & 15)] as char);
    }
    out
}

pub(super) fn decode<const N: usize>(value: &str) -> Result<[u8; N], Error> {
    if value.len() != N * 2 {
        return Err(Error::Unauthorized);
    }
    let mut bytes = [0; N];
    for (i, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        let nibble = |b| match b {
            b'0'..=b'9' => Ok(b - b'0'),
            b'a'..=b'f' => Ok(b - b'a' + 10),
            _ => Err(Error::Unauthorized),
        };
        bytes[i] = (nibble(pair[0])? << 4) | nibble(pair[1])?;
    }
    Ok(bytes)
}

/// Test injection is programmatic only; no client request/environment can set time.
#[derive(Clone, Debug, Default)]
pub struct Clock(Option<Arc<AtomicI64>>);

impl Clock {
    pub fn controlled(unix_seconds: i64) -> Self {
        Self(Some(Arc::new(AtomicI64::new(unix_seconds))))
    }
    pub fn set(&self, unix_seconds: i64) {
        if let Some(value) = &self.0 {
            value.store(unix_seconds, Ordering::SeqCst);
        }
    }
    pub(crate) fn now(&self) -> Result<i64, Error> {
        let value = match &self.0 {
            Some(value) => value.load(Ordering::SeqCst),
            None => i64::try_from(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|_| Error::Unavailable)?
                    .as_secs(),
            )
            .map_err(|_| Error::Unavailable)?,
        };
        if value < 0 {
            return Err(Error::Unavailable);
        }
        Ok(value)
    }
}
