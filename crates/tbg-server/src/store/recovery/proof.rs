//! Offline confirmation and temporary-secret boundaries; no mounted HTTP endpoint.

use sha2::{Digest, Sha256};

use super::Error;

pub(crate) fn digest(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

pub(crate) fn equal(a: &[u8], b: &[u8]) -> bool {
    let a = digest(a);
    let b = digest(b);
    a.iter().zip(b).fold(0u8, |diff, (a, b)| diff | (a ^ b)) == 0
}

pub(super) fn random_hex() -> Result<String, Error> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).map_err(|_| Error::Unavailable)?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

/// Models the trusted browser context, not a client-supplied "csrf valid" flag.
pub(crate) struct ConfirmationBrowser {
    nonce: String,
}

pub(crate) struct Post<'a> {
    pub(crate) method: &'a str,
    pub(crate) origin: &'a str,
    pub(crate) nonce: &'a str,
}

impl ConfirmationBrowser {
    pub(crate) fn new() -> Result<Self, Error> {
        Ok(Self {
            nonce: random_hex()?,
        })
    }

    pub(crate) fn form(&self) -> Post<'_> {
        Post {
            method: "POST",
            origin: "https://tbg.invalid",
            nonce: &self.nonce,
        }
    }

    pub(super) fn validate(&self, post: &Post<'_>) -> Result<(), Error> {
        if post.method != "POST"
            || post.origin != "https://tbg.invalid"
            || post.nonce.len() != 64
            || !equal(post.nonce.as_bytes(), self.nonce.as_bytes())
        {
            return Err(Error::Denied);
        }
        Ok(())
    }
}
