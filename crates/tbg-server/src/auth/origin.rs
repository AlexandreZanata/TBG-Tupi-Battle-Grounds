//! Exact browser Origin/custom-header checks and native cookie isolation (BR-AS04/13).

use super::Error;
use crate::store::auth_sessions::Channel;
use axum::http::{HeaderMap, Method};

/// Same-origin browser GETs omit Origin; their browser-controlled Fetch Metadata
/// and exact allowed Referer origin provide the read-only origin proof instead.
pub(super) fn validate_account(
    channel: Channel,
    method: &Method,
    headers: &HeaderMap,
    origins: &[String],
) -> Result<(), Error> {
    if channel != Channel::Browser || *method != Method::GET || headers.contains_key("origin") {
        return validate(channel, headers, origins);
    }
    if headers.get_all("referer").iter().count() != 1
        || headers
            .get("sec-fetch-site")
            .is_none_or(|v| v != "same-origin")
        || headers
            .get("sec-fetch-mode")
            .is_none_or(|v| v != "cors" && v != "same-origin")
        || headers.get("sec-fetch-dest").is_none_or(|v| v != "empty")
    {
        return Err(Error::Forbidden);
    }
    let referer = headers
        .get("referer")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| url::Url::parse(v).ok())
        .filter(|v| v.username().is_empty() && v.password().is_none())
        .ok_or(Error::Forbidden)?;
    let origin = referer.origin().ascii_serialization();
    let mut envelope = headers.clone();
    envelope.insert("origin", origin.parse().map_err(|_| Error::Forbidden)?);
    validate(channel, &envelope, origins)
}

pub(super) fn canonical(value: &str) -> Option<String> {
    let url = url::Url::parse(value).ok()?;
    let local = match url.host() {
        Some(url::Host::Ipv4(ip)) => ip.is_loopback(),
        Some(url::Host::Ipv6(ip)) => ip.is_loopback(),
        Some(url::Host::Domain("localhost")) => true,
        _ => false,
    };
    if url.scheme() != "https" && !(url.scheme() == "http" && local) {
        return None;
    }
    if !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.path() != "/"
    {
        return None;
    }
    let origin = url.origin().ascii_serialization();
    (origin == value).then_some(origin)
}

pub(super) fn allowed(headers: &HeaderMap, origins: &[String]) -> bool {
    if headers.get_all("origin").iter().count() != 1 {
        return false;
    }
    let Some(origin) = headers.get("origin").and_then(|v| v.to_str().ok()) else {
        return false;
    };
    origins.iter().any(|v| v == origin) && canonical(origin).is_some()
}

pub(super) fn validate(
    channel: Channel,
    headers: &HeaderMap,
    origins: &[String],
) -> Result<(), Error> {
    if [
        "origin",
        "content-type",
        "x-tbg-auth",
        "x-tbg-client",
        "sec-fetch-site",
        "sec-fetch-mode",
        "sec-fetch-dest",
    ]
    .iter()
    .any(|name| headers.get_all(*name).iter().count() > 1)
    {
        return Err(Error::Forbidden);
    }
    let json = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| {
            v.eq_ignore_ascii_case("application/json")
                || v.eq_ignore_ascii_case("application/json; charset=utf-8")
        });
    if !json {
        return Err(Error::BadRequest);
    }
    match channel {
        Channel::Browser => {
            if !allowed(headers, origins) || headers.get("x-tbg-auth").is_none_or(|v| v != "1") {
                return Err(Error::Forbidden);
            }
            if headers
                .get("sec-fetch-site")
                .is_some_and(|v| v != "same-origin")
                || headers
                    .get("sec-fetch-mode")
                    .is_some_and(|v| v != "cors" && v != "same-origin")
                || headers.get("sec-fetch-dest").is_some_and(|v| v != "empty")
            {
                return Err(Error::Forbidden);
            }
        }
        Channel::Native => {
            if headers.contains_key("origin")
                || headers.contains_key("cookie")
                || headers
                    .keys()
                    .any(|name| name.as_str().starts_with("sec-fetch-"))
                || headers.get("x-tbg-client").is_none_or(|v| v != "native")
            {
                return Err(Error::Forbidden);
            }
        }
    }
    Ok(())
}

pub(crate) fn websocket(headers: &HeaderMap, origins: &[String]) -> bool {
    if headers.contains_key("origin") {
        allowed(headers, origins)
    } else {
        !headers.contains_key("cookie")
            && !headers
                .keys()
                .any(|name| name.as_str().starts_with("sec-fetch-"))
    }
}

pub(super) fn cookie(headers: &HeaderMap) -> Result<Option<String>, Error> {
    let mut found = None;
    for header in headers.get_all("cookie") {
        let value = header.to_str().map_err(|_| Error::Unauthorized)?;
        for item in value.split(';') {
            if let Some((name, value)) = item.trim().split_once('=') {
                if name == "__Host-tbg_refresh" {
                    if found.is_some() {
                        return Err(Error::Unauthorized);
                    }
                    super::tokens::decode::<32>(value)?;
                    found = Some(value.to_owned());
                }
            }
        }
    }
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn browser_read() -> HeaderMap {
        let mut headers = HeaderMap::new();
        for (key, value) in [
            ("content-type", "application/json"),
            ("x-tbg-auth", "1"),
            ("sec-fetch-site", "same-origin"),
            ("sec-fetch-mode", "cors"),
            ("sec-fetch-dest", "empty"),
            ("referer", "https://app.example/game/?lang=en"),
        ] {
            headers.insert(key, value.parse().unwrap());
        }
        headers
    }

    #[test]
    fn browser_account_get_requires_exact_origin_proof_even_without_origin_header() {
        let origins = ["https://app.example".into()];
        let read = browser_read();
        let check = |h: &HeaderMap| validate_account(Channel::Browser, &Method::GET, h, &origins);
        assert_eq!(check(&read), Ok(()));
        for (key, value) in [
            ("sec-fetch-site", "same-site"),
            ("sec-fetch-mode", "navigate"),
            ("sec-fetch-dest", "document"),
            ("referer", "https://other.example/"),
            ("referer", "https://app.example.evil/"),
            ("referer", "https://app.example@other.example/"),
            ("referer", "http://app.example/"),
            ("origin", "https://other.example"),
            ("origin", "null"),
            ("x-tbg-auth", "0"),
        ] {
            let mut bad = read.clone();
            bad.insert(key, value.parse().unwrap());
            assert!(check(&bad).is_err(), "{key}: {value}");
        }
        for key in [
            "referer",
            "sec-fetch-site",
            "sec-fetch-mode",
            "sec-fetch-dest",
            "x-tbg-auth",
        ] {
            let mut missing = read.clone();
            missing.remove(key);
            assert!(check(&missing).is_err(), "missing {key}");
            let mut duplicate = read.clone();
            duplicate.append(key, read[key].clone());
            assert!(check(&duplicate).is_err(), "duplicate {key}");
        }
        for method in [
            Method::PUT,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
            Method::HEAD,
        ] {
            assert!(validate_account(Channel::Browser, &method, &read, &origins).is_err());
        }
        assert!(validate_account(Channel::Native, &Method::GET, &read, &origins).is_err());
    }
}
