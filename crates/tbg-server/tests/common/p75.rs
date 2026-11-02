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
        static CAPTURES: std::sync::OnceLock<
            std::sync::Mutex<std::collections::BTreeMap<PathBuf, MailCapture>>,
        > = std::sync::OnceLock::new();
        let capture = {
            let mut captures = CAPTURES.get_or_init(Default::default).lock().unwrap();
            assert!(
                captures.len() < 256,
                "synthetic capture inventory is bounded"
            );
            captures.entry(db.clone()).or_default().clone()
        };
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
                if mail.purpose == purpose && current_code(&self.db, &mail) {
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
            .header("x-tbg-client", "native");
        if !headers
            .iter()
            .any(|(key, _)| key.eq_ignore_ascii_case("content-type"))
        {
            req = req.header("content-type", "application/json");
        }
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
            body: String::from_utf8_lossy(
                &reply
                    .body_mut()
                    .with_config()
                    .limit(128 * 1024)
                    .read_to_vec()
                    .unwrap(),
            )
            .into_owned(),
        }
    })
    .await
    .unwrap()
}

pub struct BinaryReply {
    pub status: u16,
    pub bytes: Vec<u8>,
}
pub async fn bytes_request(
    url: &str,
    method: &str,
    path: &str,
    bytes: Vec<u8>,
    mime: &str,
    access: Option<&str>,
) -> BinaryReply {
    let url = format!("{url}{path}");
    let method = method.to_owned();
    let mime = mime.to_owned();
    let access = access.map(str::to_owned);
    tokio::task::spawn_blocking(move || {
        let agent = ureq::Agent::config_builder()
            .max_redirects(0)
            .http_status_as_error(false)
            .timeout_global(Some(Duration::from_secs(5)))
            .build()
            .new_agent();
        let mut req = ureq::http::Request::builder()
            .uri(url)
            .method(method.as_str())
            .header("content-type", mime)
            .header("x-tbg-client", "native");
        if let Some(access) = access {
            req = req.header("authorization", format!("Bearer {access}"));
        }
        let mut r = agent.run(req.body(bytes).unwrap()).unwrap();
        BinaryReply {
            status: r.status().as_u16(),
            bytes: r
                .body_mut()
                .with_config()
                .limit(1048576)
                .read_to_vec()
                .unwrap(),
        }
    })
    .await
    .unwrap()
}

/// Fixed-size log capture for S10; proves capture is active with a harmless marker.
#[derive(Clone, Default)]
struct LogBuffer(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);
impl std::io::Write for LogBuffer {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if let Ok(mut out) = self.0.lock() {
            let room = 1048576usize.saturating_sub(out.len());
            out.extend_from_slice(&bytes[..bytes.len().min(room)]);
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for LogBuffer {
    type Writer = Self;
    fn make_writer(&'a self) -> Self {
        self.clone()
    }
}
pub fn logs() -> String {
    static LOGS: std::sync::OnceLock<LogBuffer> = std::sync::OnceLock::new();
    let buffer = LOGS.get_or_init(|| {
        let buffer = LogBuffer::default();
        let _ = tracing::subscriber::set_global_default(
            tracing_subscriber::fmt()
                .without_time()
                .with_ansi(false)
                .with_max_level(tracing::Level::TRACE)
                .with_writer(buffer.clone())
                .finish(),
        );
        buffer
    });
    tracing::info!("P75_LOG_CAPTURE_ACTIVE");
    String::from_utf8_lossy(&buffer.0.lock().unwrap()).into_owned()
}

/// Snapshot all existing/future direct account-FK tables and all non-exempt account columns.
pub fn bound_rows(
    path: &std::path::Path,
    excluded: &[&str],
    account_columns: &[&str],
) -> std::collections::BTreeMap<String, Vec<String>> {
    let db = rusqlite::Connection::open(path).unwrap();
    let mut stmt=db.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name").unwrap();
    let tables = stmt
        .query_map([], |r| r.get::<_, String>(0))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let mut result = std::collections::BTreeMap::new();
    for table in tables {
        if excluded
            .iter()
            .any(|prefix| table == *prefix || prefix.ends_with('_') && table.starts_with(prefix))
        {
            continue;
        }
        let bound: bool = db
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM pragma_foreign_key_list(?1) WHERE [table]='accounts')",
                [&table],
                |r| r.get(0),
            )
            .unwrap();
        if table != "accounts" && !bound {
            continue;
        }
        let quote = |s: &str| format!("\"{}\"", s.replace('"', "\"\""));
        let mut info = db
            .prepare("SELECT name FROM pragma_table_info(?1) ORDER BY cid")
            .unwrap();
        let columns = info
            .query_map([&table], |r| r.get::<_, String>(0))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let columns: Vec<_> = columns
            .iter()
            .filter(|name| table != "accounts" || !account_columns.contains(&name.as_str()))
            .collect();
        let projection = columns
            .iter()
            .map(|name| quote(name))
            .collect::<Vec<_>>()
            .join(",");
        let order = (1..=columns.len())
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let mut rows = db
            .prepare(&format!(
                "SELECT {projection} FROM {} ORDER BY {order}",
                quote(&table)
            ))
            .unwrap();
        result.insert(
            table,
            rows.query_map([], |row| {
                Ok((0..columns.len())
                    .map(|i| format!("{:?}", row.get_ref(i).unwrap()))
                    .collect::<Vec<_>>()
                    .join("|"))
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap(),
        );
    }
    result
}

/// The dispatcher may have captured an invalidated predecessor before the next request committed.
fn current_code(path: &std::path::Path, mail: &CapturedMail) -> bool {
    let Some(code) = &mail.code else {
        return true;
    };
    let Some(canonical) = tbg_server::identity::codes::parse(code) else {
        return false;
    };
    let (purpose, operator) = match mail.purpose.as_str() {
        "account_verify" => ("verify", false),
        "account_recovery" => ("rotate", false),
        "account_change_old" => ("change_old", false),
        "account_change_new" => ("change_new", false),
        "operator_claim" => ("claim", true),
        "operator_login" => ("login", true),
        _ => return true,
    };
    let Ok(db) = rusqlite::Connection::open(path) else {
        return false;
    };
    let sql = if operator {
        "SELECT c.operator_id,c.digest FROM operator_challenges c JOIN operators o ON o.id=c.operator_id WHERE o.email=?1 AND c.purpose=?2 AND c.consumed=0"
    } else {
        "SELECT account_id,digest FROM recovery_challenges WHERE email=?1 AND purpose=?2 AND consumed=0"
    };
    let row = db.query_row(sql, rusqlite::params![mail.to, purpose], |r| {
        Ok((r.get::<_, i64>(0)?, r.get::<_, Vec<u8>>(1)?))
    });
    row.is_ok_and(|(id, digest)| {
        digest.as_slice()
            == tbg_server::identity::codes::digest(
                id,
                &if operator {
                    format!("operator:{purpose}")
                } else {
                    purpose.into()
                },
                &canonical,
            )
    })
}
