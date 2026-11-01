//! D-97 A01–A04/A08/A11–A13: bounded issuance, replay, cookie isolation and migration.

use serde_json::{json, Value};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use tbg_server::{auth::Clock, Config};

static NEXT: AtomicU64 = AtomicU64::new(1);
struct Server {
    base: String,
    db: PathBuf,
    clock: Clock,
    task: tokio::task::JoinHandle<std::io::Result<()>>,
}
impl Drop for Server {
    fn drop(&mut self) {
        self.task.abort();
    }
}
async fn server(db: Option<PathBuf>, clock: Option<Clock>) -> Server {
    let db = db.unwrap_or_else(|| {
        std::env::temp_dir().join(format!(
            "tbg-p68-http-{}-{}.sqlite",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ))
    });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let clock = clock.unwrap_or_else(|| Clock::controlled(1000));
    let config = Config {
        db: Some(db.clone()),
        auth_origins: vec![base.clone()],
        auth_clock: clock.clone(),
        max_per_ip: 100,
        register_per_ip_hour: 100,
        login_per_ip_min: 100,
        ..Config::default()
    };
    let task = tokio::spawn(tbg_server::run(config, listener));
    Server {
        base,
        db,
        clock,
        task,
    }
}
struct Reply {
    status: u16,
    value: Value,
    cookie: Option<String>,
    cache: Option<String>,
    cors: bool,
}
async fn post(
    server: &Server,
    channel: &str,
    operation: &str,
    body: Value,
    extra: Vec<(String, String)>,
) -> Reply {
    raw_post(server, channel, operation, body.to_string(), extra).await
}
async fn raw_post(
    server: &Server,
    channel: &str,
    operation: &str,
    body: String,
    extra: Vec<(String, String)>,
) -> Reply {
    let target = format!("{}/api/auth/{channel}/{operation}", server.base);
    let browser = channel == "browser";
    let base = server.base.clone();
    tokio::task::spawn_blocking(move || {
        let agent = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .timeout_global(Some(std::time::Duration::from_secs(5)))
            .build()
            .new_agent();
        let mut request = agent
            .post(target)
            .header("Content-Type", "application/json");
        if browser {
            request = request
                .header("Origin", base)
                .header("X-TBG-Auth", "1")
                .header("Sec-Fetch-Site", "same-origin");
        } else {
            request = request.header("X-TBG-Client", "native");
        }
        for (key, value) in extra {
            request = request.header(key, value);
        }
        let mut response = request.send(body.as_bytes()).unwrap();
        let status = response.status().as_u16();
        let cookie = response
            .headers()
            .get("set-cookie")
            .map(|v| v.to_str().unwrap().to_owned());
        let cache = response
            .headers()
            .get("cache-control")
            .map(|v| v.to_str().unwrap().to_owned());
        let cors = response
            .headers()
            .contains_key("access-control-allow-origin");
        let bytes = response
            .body_mut()
            .with_config()
            .limit(8192)
            .read_to_vec()
            .unwrap();
        let value = serde_json::from_slice(&bytes).unwrap();
        Reply {
            status,
            value,
            cookie,
            cache,
            cors,
        }
    })
    .await
    .unwrap()
}
fn registration(name: &str) -> Value {
    json!({"name":name,"device_id":"01".repeat(16),"persistent":true,"registration_retry_key":"02".repeat(32)})
}
fn refresh(token: &Value, op: u8) -> Value {
    json!({"refresh_token":token,"operation_id":format!("{op:02x}").repeat(16)})
}
fn credential(token: &Value) -> Value {
    json!({"refresh_token":token})
}

#[tokio::test]
async fn a01_a02_rotation_is_absolute_idempotent_bounded_and_hash_only() {
    let server = server(None, None).await;
    let initial = post(
        &server,
        "native",
        "register",
        registration("AuthLifetime"),
        vec![],
    )
    .await;
    assert_eq!(initial.status, 200);
    assert_eq!(initial.cache.as_deref(), Some("no-store"));
    assert!(!initial.cors);
    assert_eq!(initial.value["access_expires_at"], 1900);
    assert_eq!(initial.value["session_expires_at"], 2_593_000);
    let repeated = post(
        &server,
        "native",
        "register",
        registration("AuthLifetime"),
        vec![],
    )
    .await;
    assert_eq!(repeated.status, 200);
    assert_eq!(
        repeated.value, initial.value,
        "lost registration response returns the exact cached grant"
    );
    let body = refresh(&initial.value["refresh_token"], 3);
    let next = post(&server, "native", "refresh", body.clone(), vec![]).await;
    assert_eq!(next.status, 200);
    assert_eq!(
        next.value["session_expires_at"],
        initial.value["session_expires_at"]
    );
    assert_ne!(next.value["refresh_token"], initial.value["refresh_token"]);
    let repeated = post(&server, "native", "refresh", body, vec![]).await;
    assert_eq!(repeated.status, 200);
    assert_eq!(repeated.value, next.value);
    assert_eq!(
        post(
            &server,
            "native",
            "refresh",
            refresh(&initial.value["refresh_token"], 4),
            vec![]
        )
        .await
        .status,
        409
    );
    assert_eq!(
        post(
            &server,
            "native",
            "confirm",
            credential(&next.value["refresh_token"]),
            vec![]
        )
        .await
        .status,
        200
    );
    let db = rusqlite::Connection::open(&server.db).unwrap();
    let stored: Vec<Vec<u8>> = db
        .prepare("SELECT token_hash FROM auth_refresh_tokens")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert_eq!(stored.len(), 2);
    assert!(stored.iter().all(|hash| hash.len() == 32));
    for token in [
        &initial.value["refresh_token"],
        &next.value["refresh_token"],
        &initial.value["access_token"],
    ] {
        let raw = token.as_str().unwrap();
        let bytes = std::fs::read(&server.db).unwrap();
        assert!(!bytes
            .windows(raw.len())
            .any(|window| window == raw.as_bytes()));
    }
    server.clock.set(1031);
    assert_eq!(
        post(
            &server,
            "native",
            "refresh",
            refresh(&initial.value["refresh_token"], 3),
            vec![]
        )
        .await
        .status,
        401
    );
    assert_eq!(
        post(
            &server,
            "native",
            "confirm",
            credential(&next.value["refresh_token"]),
            vec![]
        )
        .await
        .status,
        401,
        "late reuse revokes the whole family"
    );
}

#[tokio::test]
async fn a03_restart_loses_retry_secrets_without_reconstructing_them_from_sql() {
    let first = server(None, None).await;
    let initial = post(
        &first,
        "native",
        "register",
        registration("RestartAuth"),
        vec![],
    )
    .await;
    let body = refresh(&initial.value["refresh_token"], 5);
    let next = post(&first, "native", "refresh", body.clone(), vec![]).await;
    assert_eq!(next.status, 200);
    let path = first.db.clone();
    let clock = first.clock.clone();
    drop(first);
    let restarted = server(Some(path), Some(clock)).await;
    let lost = post(&restarted, "native", "refresh", body, vec![]).await;
    assert_eq!(lost.status, 409);
    assert_eq!(lost.value["error"], "recovery_required");
    assert_eq!(
        post(
            &restarted,
            "native",
            "refresh",
            refresh(&next.value["refresh_token"], 6),
            vec![]
        )
        .await
        .status,
        200
    );
}

#[tokio::test]
async fn a04_a13_cookie_attributes_and_browser_native_isolation_are_enforced() {
    let server = server(None, None).await;
    let mut body = registration("BrowserAuth");
    body["persistent"] = json!(false);
    let browser = post(&server, "browser", "register", body, vec![]).await;
    assert_eq!(browser.status, 200);
    assert!(browser.value.get("refresh_token").is_none());
    let cookie = browser.cookie.unwrap();
    for attr in [
        "__Host-tbg_refresh=",
        "; Path=/",
        "; Secure",
        "; HttpOnly",
        "; SameSite=Strict",
    ] {
        assert!(cookie.contains(attr));
    }
    assert!(!cookie.contains("Domain="));
    assert!(!cookie.contains("Max-Age="));
    let token = cookie.split(';').next().unwrap().split_once('=').unwrap().1;
    let cookie_header = vec![(
        "Cookie".into(),
        cookie.split(';').next().unwrap().to_owned(),
    )];
    assert_eq!(
        post(
            &server,
            "browser",
            "confirm",
            json!({}),
            cookie_header.clone()
        )
        .await
        .status,
        200
    );
    assert_eq!(
        post(
            &server,
            "native",
            "confirm",
            json!({"refresh_token":token}),
            vec![]
        )
        .await
        .status,
        401
    );
    for header in [
        ("Origin", server.base.as_str()),
        ("Cookie", "anything=1"),
        ("Sec-Fetch-Site", "none"),
    ] {
        assert_eq!(
            post(
                &server,
                "native",
                "login",
                json!({}),
                vec![(header.0.into(), header.1.into())]
            )
            .await
            .status,
            403
        );
    }
    assert_eq!(
        post(
            &server,
            "browser",
            "confirm",
            json!({}),
            vec![("Origin".into(), "https://attacker.example".into())]
        )
        .await
        .status,
        403
    );
    assert_eq!(
        post(
            &server,
            "browser",
            "confirm",
            json!({}),
            vec![("Sec-Fetch-Site".into(), "cross-site".into())]
        )
        .await
        .status,
        403
    );
    let updated = post(
        &server,
        "browser",
        "refresh",
        json!({"operation_id":"07".repeat(16)}),
        cookie_header,
    )
    .await;
    assert_eq!(updated.status, 200);
    assert!(updated.value.get("refresh_token").is_none());
    assert!(
        !updated.cookie.unwrap().contains("Max-Age="),
        "session-cookie choice survives rotation"
    );
    assert_eq!(raw_post(&server,"native","login","{\"account_key\":\"x\",\"account_key\":\"y\",\"persistent\":true,\"device_id\":\"bad\"}".into(),vec![]).await.status,400);
    assert_eq!(
        raw_post(&server, "native", "login", " ".repeat(4097), vec![])
            .await
            .status,
        400
    );
}

#[tokio::test]
async fn a11_family_cap_and_absolute_expiry_preserve_manual_recovery() {
    let server = server(None, None).await;
    let first = post(
        &server,
        "native",
        "register",
        registration("FamilyCap"),
        vec![],
    )
    .await;
    assert_eq!(first.status, 200);
    let key = &first.value["account_key"];
    for n in 1..10 {
        let body =
            json!({"account_key":key,"device_id":format!("{n:02x}").repeat(16),"persistent":true});
        assert_eq!(
            post(&server, "native", "login", body, vec![]).await.status,
            200
        );
    }
    assert_eq!(
        post(
            &server,
            "native",
            "login",
            json!({"account_key":key,"device_id":"ff".repeat(16),"persistent":true}),
            vec![]
        )
        .await
        .status,
        409
    );
    server.clock.set(2_593_000);
    assert_eq!(
        post(
            &server,
            "native",
            "refresh",
            refresh(&first.value["refresh_token"], 8),
            vec![]
        )
        .await
        .status,
        401
    );
    assert_eq!(
        post(
            &server,
            "native",
            "login",
            json!({"account_key":key,"device_id":"ff".repeat(16),"persistent":true}),
            vec![]
        )
        .await
        .status,
        200,
        "manual AccountKey remains valid after family expiry"
    );
}

#[tokio::test]
async fn a03_concurrent_retries_issue_one_successor_and_conflict_does_not_revoke() {
    let server = server(None, None).await;
    let initial = post(
        &server,
        "native",
        "register",
        registration("ParallelRetry"),
        vec![],
    )
    .await;
    let body = refresh(&initial.value["refresh_token"], 9);
    let (a, b) = tokio::join!(
        post(&server, "native", "refresh", body.clone(), vec![]),
        post(&server, "native", "refresh", body, vec![])
    );
    assert_eq!((a.status, b.status), (200, 200));
    assert_eq!(a.value["access_token"], b.value["access_token"]);
    assert_eq!(a.value["refresh_token"], b.value["refresh_token"]);
    assert_eq!(
        post(
            &server,
            "native",
            "refresh",
            refresh(&initial.value["refresh_token"], 10),
            vec![]
        )
        .await
        .status,
        409
    );
    assert_eq!(
        post(
            &server,
            "native",
            "confirm",
            credential(&a.value["refresh_token"]),
            vec![]
        )
        .await
        .status,
        200
    );
}

#[tokio::test]
async fn a10_manual_recovery_never_erases_an_unrelated_browser_cookie() {
    let server = server(None, None).await;
    let native = post(
        &server,
        "native",
        "register",
        registration("RecoveredUser"),
        vec![],
    )
    .await;
    let mut other_registration = registration("OtherBrowser");
    other_registration["registration_retry_key"] = json!("03".repeat(32));
    let browser = post(&server, "browser", "register", other_registration, vec![]).await;
    let cookie = browser
        .cookie
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_owned();
    let recovered = post(
        &server,
        "browser",
        "logout-all",
        json!({"account_key":native.value["account_key"]}),
        vec![("Cookie".into(), cookie.clone())],
    )
    .await;
    assert_eq!(recovered.status, 200);
    assert!(recovered.cookie.is_none());
    assert_eq!(
        post(
            &server,
            "browser",
            "confirm",
            json!({}),
            vec![("Cookie".into(), cookie)]
        )
        .await
        .status,
        200
    );
}
