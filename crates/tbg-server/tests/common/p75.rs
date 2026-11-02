//! Isolated P75 HTTP, clock, mail and database fixtures; never real SMTP.
#![allow(dead_code)]
use serde_json::{json, Value};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};
use tbg_server::{
    auth::Clock,
    mail::transport::{CapturedMail, MailCapture},
    Config,
};
static NEXT: AtomicU64 = AtomicU64::new(0);
pub const NOW: i64 = 1_000_000;
pub struct Server {
    pub shared: tbg_server::Shared,
    pub url: String,
    pub db: PathBuf,
    pub clock: Clock,
    pub capture: MailCapture,
    pub task: tokio::task::JoinHandle<std::io::Result<()>>,
}
impl Server {
    pub async fn new() -> Self {
        Self::configured(|_| {}).await
    }
    pub async fn configured(change: impl FnOnce(&mut Config)) -> Self {
        let db = std::env::temp_dir().join(format!(
            "tbg-p75-{}-{}.sqlite",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        Self::on(db, Clock::controlled(NOW), change).await
    }
    pub async fn on(db: PathBuf, clock: Clock, change: impl FnOnce(&mut Config)) -> Self {
        let capture = MailCapture::default();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let mut config = Config {
            db: Some(db.clone()),
            auth_clock: clock.clone(),
            mail_capture: Some(capture.clone()),
            register_per_ip_hour: 100,
            max_per_ip: 100,
            login_per_ip_min: 1000,
            api_per_ip_min: 1000,
            tick_ms: 10,
            ..Config::default()
        };
        change(&mut config);
        let shared = tbg_server::Shared::default();
        let task = tokio::spawn(tbg_server::run_with(config, listener, shared.clone()));
        Self {
            shared,
            url,
            db,
            clock,
            capture,
            task,
        }
    }
    pub async fn request(
        &self,
        method: &str,
        path: &str,
        value: Value,
        access: Option<&str>,
    ) -> Reply {
        let headers = access
            .map(|v| vec![("authorization".into(), format!("Bearer {v}"))])
            .unwrap_or_default();
        request(&self.url, method, path, value, headers).await
    }
    pub async fn register(&self, name: &str, email: &str) -> Value {
        let reply = self
            .request(
                "POST",
                "/api/auth/native/register",
                registration(name, email),
                None,
            )
            .await;
        assert_eq!(reply.status, 200, "{}", reply.body);
        reply.json()
    }
    pub async fn mail(&self, purpose: &str) -> CapturedMail {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
        loop {
            while let Some(mail) = self.capture.take() {
                if mail.purpose == purpose {
                    return mail;
                }
            }
            assert!(
                tokio::time::Instant::now() < deadline,
                "capture mail timeout: {purpose}"
            );
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }
    pub fn sql(&self, batch: &str) {
        rusqlite::Connection::open(&self.db)
            .unwrap()
            .execute_batch(batch)
            .unwrap();
    }
    pub fn number(&self, query: &str) -> i64 {
        rusqlite::Connection::open(&self.db)
            .unwrap()
            .query_row(query, [], |r| r.get(0))
            .unwrap()
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        self.task.abort();
    }
}
pub fn registration(name: &str, email: &str) -> Value {
    json!({"name":name,"email":email,"locale":"en","device_id":random(16),"persistent":false,"registration_retry_key":random(32)})
}
pub fn random(size: usize) -> String {
    tbg_server::botclient::native_auth::random_hex(size).unwrap()
}
pub struct Reply {
    pub status: u16,
    pub body: String,
    pub headers: ureq::http::HeaderMap,
}
impl Reply {
    pub fn json(&self) -> Value {
        serde_json::from_str(&self.body).unwrap_or(Value::Null)
    }
}
pub async fn request(
    url: &str,
    method: &str,
    path: &str,
    value: Value,
    headers: Vec<(String, String)>,
) -> Reply {
    let url = format!("{url}{path}");
    let method = method.to_owned();
    tokio::task::spawn_blocking(move || {
        let agent = ureq::Agent::config_builder()
            .max_redirects(0)
            .http_status_as_error(false)
            .timeout_global(Some(Duration::from_secs(7)))
            .build()
            .new_agent();
        let mut req = ureq::http::Request::builder()
            .uri(url)
            .method(method.as_str())
            .header("content-type", "application/json")
            .header("x-tbg-client", "native");
        for (key, value) in headers {
            req = req.header(key, value);
        }
        let bytes = if method == "GET" {
            Vec::new()
        } else {
            serde_json::to_vec(&value).unwrap()
        };
        let mut reply = agent.run(req.body(bytes).unwrap()).unwrap();
        Reply {
            status: reply.status().as_u16(),
            headers: reply.headers().clone(),
            body: reply
                .body_mut()
                .with_config()
                .limit(128 * 1024)
                .read_to_string()
                .unwrap(),
        }
    })
    .await
    .unwrap()
}
