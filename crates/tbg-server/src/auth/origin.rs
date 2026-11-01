//! Exact browser Origin/custom-header checks and native cookie isolation (BR-AS04/13).

use super::Error;
use crate::store::auth_sessions::Channel;
use axum::http::HeaderMap;

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
