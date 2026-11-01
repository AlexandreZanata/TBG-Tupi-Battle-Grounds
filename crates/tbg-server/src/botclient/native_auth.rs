//! Native HTTP authentication for transient bots and synthetic acceptance clients.

use serde_json::{json, Value};
use std::time::Duration;
use tbg_core::protocol::{AccessToken, Profile};

/// Deliberately no Debug/serialization: credentials remain process-local.
pub struct Grant {
    pub access: AccessToken,
    pub refresh: String,
    pub profile: Profile,
    pub account_key: Option<String>,
    pub expires_at: i64,
    pub server_time: i64,
}

pub async fn post(
    server: &str,
    operation: &str,
    body: Value,
    bearer: Option<String>,
) -> Result<(u16, Value), &'static str> {
    if ![
        "login",
        "register",
        "refresh",
        "confirm",
        "logout",
        "logout-all",
    ]
    .contains(&operation)
    {
        return Err("invalid auth operation");
    }
    let mut origin = url::Url::parse(server).map_err(|_| "invalid auth origin")?;
    if !origin.username().is_empty()
        || origin.password().is_some()
        || origin.query().is_some()
        || origin.fragment().is_some()
    {
        return Err("invalid auth origin");
    }
    let scheme = match origin.scheme() {
        "ws" | "http" => "http",
        "wss" | "https" => "https",
        _ => return Err("invalid auth origin"),
    };
    origin
        .set_scheme(scheme)
        .map_err(|_| "invalid auth origin")?;
    let local = match origin.host() {
        Some(url::Host::Ipv4(ip)) => ip.is_loopback(),
        Some(url::Host::Ipv6(ip)) => ip.is_loopback(),
        Some(url::Host::Domain("localhost")) => true,
        _ => false,
    };
    if scheme != "https" && !local {
        return Err("auth requires HTTPS outside loopback");
    }
    let target = format!(
        "{}/api/auth/native/{operation}",
        origin.origin().ascii_serialization()
    );
    let body = serde_json::to_vec(&body).map_err(|_| "invalid auth request")?;
    if body.len() > 4096 {
        return Err("auth request too large");
    }
    tokio::task::spawn_blocking(move || {
        let agent = ureq::Agent::config_builder()
            .max_redirects(0)
            .http_status_as_error(false)
            .timeout_global(Some(Duration::from_secs(5)))
            .build()
            .new_agent();
        let mut request = agent
            .post(target)
            .header("Content-Type", "application/json")
            .header("X-TBG-Client", "native");
        if let Some(access) = bearer {
            let mut header = ureq::http::HeaderValue::from_str(&format!("Bearer {access}"))
                .map_err(|_| "invalid Access credential")?;
            header.set_sensitive(true);
            request = request.header("Authorization", header);
        }
        let mut response = request
            .send(body.as_slice())
            .map_err(|_| "auth transport unavailable")?;
        let status = response.status().as_u16();
        if (300..400).contains(&status) {
            return Err("auth redirects are forbidden");
        }
        let bytes = response
            .body_mut()
            .with_config()
            .limit(4096)
            .read_to_vec()
            .map_err(|_| "invalid auth response")?;
        let value = serde_json::from_slice(&bytes).map_err(|_| "incompatible auth backend")?;
        Ok((status, value))
    })
    .await
    .map_err(|_| "auth worker unavailable")?
}

pub fn random_hex(bytes: usize) -> Result<String, &'static str> {
    if ![16, 32].contains(&bytes) {
        return Err("invalid auth entropy size");
    }
    let mut raw = vec![0; bytes];
    getrandom::fill(&mut raw).map_err(|_| "auth entropy unavailable")?;
    Ok(raw.iter().map(|v| format!("{v:02x}")).collect())
}

pub fn parse(value: &Value) -> Result<Grant, &'static str> {
    let access = value
        .get("access_token")
        .and_then(Value::as_str)
        .and_then(AccessToken::from_hex)
        .ok_or("invalid Access response")?;
    let refresh = value
        .get("refresh_token")
        .and_then(Value::as_str)
        .filter(|v| AccessToken::from_hex(v).is_some())
        .ok_or("invalid Refresh response")?
        .to_owned();
    let profile: Profile =
        serde_json::from_value(value.get("profile").ok_or("missing profile")?.clone())
            .map_err(|_| "invalid profile")?;
    if !tbg_core::account::valid_name_shape(&profile.name) {
        return Err("invalid profile");
    }
    let expires_at = value
        .get("access_expires_at")
        .and_then(Value::as_i64)
        .ok_or("invalid auth expiry")?;
    let server_time = value
        .get("server_time")
        .and_then(Value::as_i64)
        .ok_or("invalid server time")?;
    if server_time < 0 || expires_at <= server_time || expires_at.saturating_sub(server_time) > 900
    {
        return Err("invalid auth lifetime");
    }
    let account_key = value
        .get("account_key")
        .and_then(Value::as_str)
        .map(str::to_owned);
    if account_key
        .as_ref()
        .is_some_and(|key| tbg_core::account::parse_key(key).is_none())
    {
        return Err("invalid AccountKey");
    }
    Ok(Grant {
        access,
        refresh,
        profile,
        account_key,
        expires_at,
        server_time,
    })
}

pub async fn register(server: &str, name: &str) -> Result<Grant, &'static str> {
    let body = json!({"name":name,"device_id":random_hex(16)?,"registration_retry_key":random_hex(32)?,"persistent":false});
    let (status, value) = post(server, "register", body, None).await?;
    if status == 409 && value.get("error").and_then(Value::as_str) == Some("name_taken") {
        return Err("name_taken");
    }
    if status != 200 {
        return Err("registration refused");
    }
    parse(&value)
}

pub async fn refresh(server: &str, grant: &Grant) -> Result<Grant, &'static str> {
    let (status, value) = post(
        server,
        "refresh",
        json!({"refresh_token":grant.refresh,"operation_id":random_hex(16)?}),
        None,
    )
    .await?;
    if status != 200 {
        return Err("refresh refused");
    }
    parse(&value)
}
