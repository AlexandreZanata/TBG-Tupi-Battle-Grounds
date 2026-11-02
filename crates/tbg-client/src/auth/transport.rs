//! Auth-specific transport forbids redirects, public HTTP and unbounded response reads.

use super::types::Failure;

#[cfg(not(target_arch = "wasm32"))]
pub fn origin(server: &str) -> Result<String, Failure> {
    let mut url = url::Url::parse(server).map_err(|_| Failure::InvalidOrigin)?;
    if !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(Failure::InvalidOrigin);
    }
    let scheme = match url.scheme() {
        "wss" => "https",
        "ws" => "http",
        _ => return Err(Failure::InvalidOrigin),
    };
    url.set_scheme(scheme).map_err(|_| Failure::InvalidOrigin)?;
    let local = match url.host() {
        Some(url::Host::Ipv4(ip)) => ip.is_loopback(),
        Some(url::Host::Ipv6(ip)) => ip.is_loopback(),
        Some(url::Host::Domain("localhost")) => true,
        _ => false,
    };
    if scheme != "https" && !local {
        return Err(Failure::InvalidOrigin);
    }
    Ok(url.origin().ascii_serialization())
}

#[cfg(target_arch = "wasm32")]
pub fn origin(server: &str) -> Result<String, Failure> {
    let origin = crate::platform::http::origin(server).ok_or(Failure::InvalidOrigin)?;
    let current = web_sys::window()
        .ok_or(Failure::InvalidOrigin)?
        .location()
        .origin()
        .map_err(|_| Failure::InvalidOrigin)?;
    if origin != current {
        return Err(Failure::InvalidOrigin);
    }
    Ok(origin)
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn post(
    origin: &str,
    operation: &str,
    body: serde_json::Value,
    access: Option<tbg_core::protocol::AccessToken>,
) -> Result<serde_json::Value, Failure> {
    if ![
        "login",
        "register",
        "recover",
        "recovery-request",
        "refresh",
        "confirm",
        "logout",
        "logout-all",
    ]
    .contains(&operation)
    {
        return Err(Failure::Incompatible);
    }
    let bytes = serde_json::to_vec(&body).map_err(|_| Failure::Incompatible)?;
    if bytes.len() > 4096 {
        return Err(Failure::Incompatible);
    }
    let agent = ureq::Agent::config_builder()
        .max_redirects(0)
        .max_redirects_will_error(false)
        .http_status_as_error(false)
        .timeout_global(Some(std::time::Duration::from_secs(5)))
        .build()
        .new_agent();
    let mut request = agent
        .post(format!("{origin}/api/auth/native/{operation}"))
        .header("Content-Type", "application/json")
        .header("X-TBG-Client", "native")
        .header("Cache-Control", "no-store");
    // P66: the proxy routes auth to the generation holding this client's socket.
    if let Some(slot) = super::slot() {
        request = request.header("X-TBG-Slot", slot.to_string());
    }
    if let Some(access) = access {
        let encoded: String = access.0.iter().map(|v| format!("{v:02x}")).collect();
        let mut header = ureq::http::HeaderValue::from_str(&format!("Bearer {encoded}"))
            .map_err(|_| Failure::Incompatible)?;
        header.set_sensitive(true);
        request = request.header("Authorization", header);
    }
    let mut response = request
        .send(bytes.as_slice())
        .map_err(|_| Failure::Unavailable)?;
    let status = response.status().as_u16();
    if (300..400).contains(&status) || status == 404 {
        return Err(Failure::Incompatible);
    }
    let bytes = response
        .body_mut()
        .with_config()
        .limit(4096)
        .read_to_vec()
        .map_err(|_| Failure::Unavailable)?;
    let value = serde_json::from_slice(&bytes).map_err(|_| Failure::Incompatible)?;
    if status != 200 && !(status == 202 && operation == "recovery-request") {
        return Err(super::types::public_error(&value));
    }
    Ok(value)
}
