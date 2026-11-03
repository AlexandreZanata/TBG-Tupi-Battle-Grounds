//! Trusted-proxy IP extraction, exact-origin mutations and bounded console envelopes.
use crate::store::operators::Error;
use axum::http::{HeaderMap, Method};
use std::net::IpAddr;
pub fn cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    let mut found = None;
    for header in headers.get_all("cookie") {
        for entry in header.to_str().ok()?.split(';') {
            let (key, value) = entry.trim().split_once('=')?;
            if key == name {
                if found.is_some() {
                    return None;
                }
                found = Some(value.into());
            }
        }
    }
    found
}
pub fn mutation(headers: &HeaderMap, method: &Method, origin: &str) -> Result<(), Error> {
    if method == Method::GET {
        return Ok(());
    }
    if headers.get_all("origin").iter().count() != 1
        || headers.get("origin").and_then(|v| v.to_str().ok()) != Some(origin)
        || headers.get_all("content-type").iter().count() != 1
        || headers
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split(';').next())
            != Some("application/json")
        || headers.get_all("x-tbg-csrf").iter().count() != 1
    {
        return Err(Error::Forbidden);
    }
    Ok(())
}
pub fn allowed_ip(ip: IpAddr, entries: &[String]) -> bool {
    entries.is_empty() || entries.iter().any(|s| cidr(ip, s))
}
fn cidr(ip: IpAddr, entry: &str) -> bool {
    let (raw, prefix) = entry.split_once('/').unwrap_or((entry, ""));
    let Ok(network) = raw.parse::<IpAddr>() else {
        return false;
    };
    match (ip, network) {
        (IpAddr::V4(ip), IpAddr::V4(net)) => {
            let bits = if prefix.is_empty() {
                32
            } else {
                prefix.parse().unwrap_or(255)
            };
            if bits > 32 {
                return false;
            }
            let mask = u32::MAX.checked_shl(32 - bits).unwrap_or(0);
            u32::from(ip) & mask == u32::from(net) & mask
        }
        (IpAddr::V6(ip), IpAddr::V6(net)) => {
            let bits = if prefix.is_empty() {
                128
            } else {
                prefix.parse().unwrap_or(255)
            };
            if bits > 128 {
                return false;
            }
            let mask = u128::MAX.checked_shl(128 - bits).unwrap_or(0);
            u128::from(ip) & mask == u128::from(net) & mask
        }
        _ => false,
    }
}

pub fn valid_cidr(entry: &str) -> bool {
    let (addr, bits) = entry.split_once('/').unwrap_or((entry, ""));
    let Ok(addr) = addr.parse::<IpAddr>() else {
        return false;
    };
    bits.is_empty()
        || bits
            .parse::<u8>()
            .is_ok_and(|v| v <= if addr.is_ipv4() { 32 } else { 128 })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn p75_s14_forwarded_ip_is_only_trusted_from_loopback() {
        let mut h = HeaderMap::new();
        h.insert("x-forwarded-for", "127.0.0.1".parse().unwrap());
        h.insert("host", "admin.tupibg.com".parse().unwrap());
        let peer = "203.0.113.4:42".parse().unwrap();
        let ip = crate::ws::client_ip(peer, &h);
        assert_eq!(ip, "203.0.113.4".parse::<IpAddr>().unwrap());
        assert!(!allowed_ip(ip, &["127.0.0.0/8".into()]));
        assert!(allowed_ip(
            "192.0.2.7".parse().unwrap(),
            &["192.0.2.0/24".into()]
        ));
        assert!(!valid_cidr("0.0.0.0/33"));
    }
}
