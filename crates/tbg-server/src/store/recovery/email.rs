//! Narrow synthetic email normalization, pending the owner's O02 delivery policy.

use super::Error;

pub fn normalize(email: &str) -> Result<String, Error> {
    let email = email.trim_matches(' ');
    if !email.is_ascii() || email.len() > 254 {
        return Err(Error::Denied);
    }
    let (local, domain) = email.split_once('@').ok_or(Error::Denied)?;
    if !valid_local(local) || !valid_domain(domain) {
        return Err(Error::Denied);
    }
    Ok(format!("{local}@{}", domain.to_ascii_lowercase()))
}

fn valid_local(local: &str) -> bool {
    !local.is_empty()
        && local.len() <= 64
        && !local.starts_with('.')
        && !local.ends_with('.')
        && !local.contains("..")
        && local
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b".!#$%&'*+-/=?^_`{|}~".contains(&b))
}

fn valid_domain(domain: &str) -> bool {
    domain.contains('.') && domain.split('.').all(valid_label)
}

fn valid_label(label: &str) -> bool {
    !label.is_empty()
        && label.len() <= 63
        && !label.to_ascii_lowercase().starts_with("xn--")
        && !label.starts_with('-')
        && !label.ends_with('-')
        && label
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
}
