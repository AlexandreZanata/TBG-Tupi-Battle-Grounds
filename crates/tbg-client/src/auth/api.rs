//! Account-scoped bounded HTTP on the existing auth coordinator; UI never handles credentials.

use super::{epoch, snapshot, Status};
use serde_json::Value;
use std::sync::{Arc, Mutex};
use tbg_core::protocol::AccessToken;

#[derive(Clone, Debug)]
pub struct Reply {
    pub status: u16,
    pub value: Value,
    pub bytes: Option<Vec<u8>>,
}
type Slot = Arc<Mutex<Option<Result<Reply, ()>>>>;
pub struct Request {
    path: String,
    method: &'static str,
    body: Option<Value>,
    revision: Option<i64>,
    raw: Option<Vec<u8>>,
    mime: Option<String>,
    sheet: bool,
    private: bool,
    epoch: u64,
    access: Option<AccessToken>,
    slot: Slot,
    retried: bool,
    wait_until: Option<f64>,
}
impl Request {
    pub fn new(
        path: String,
        method: &'static str,
        body: Option<Value>,
        revision: Option<i64>,
        private: bool,
    ) -> Self {
        let mut request = Self::prepare(path, method, body, revision, private);
        request.dispatch();
        request
    }
    fn prepare(
        path: String,
        method: &'static str,
        body: Option<Value>,
        revision: Option<i64>,
        private: bool,
    ) -> Self {
        let view = snapshot();
        Self {
            path,
            method,
            body,
            revision,
            private,
            raw: None,
            mime: None,
            sheet: false,
            epoch: epoch(),
            access: view.access,
            slot: Arc::default(),
            retried: false,
            wait_until: None,
        }
    }
    pub fn recovery(name: &str, email: &str, locale: &str) -> Self {
        let channel = if cfg!(target_arch = "wasm32") {
            "browser"
        } else {
            "native"
        };
        Self::new(
            format!("/api/auth/{channel}/recovery-request"),
            "POST",
            Some(serde_json::json!({"name":name,"email":email,"locale":locale})),
            None,
            false,
        )
    }
    pub fn upload(label: &str, bytes: Vec<u8>, mime: &str) -> Self {
        let label: String = label
            .bytes()
            .map(|b| {
                if b.is_ascii_alphanumeric() {
                    (b as char).to_string()
                } else {
                    format!("%{b:02X}")
                }
            })
            .collect();
        let mut r = Self::prepare(
            format!("/api/me/emoji?label={label}"),
            "POST",
            None,
            None,
            true,
        );
        r.raw = Some(bytes);
        r.mime = Some(mime.into());
        r.dispatch();
        r
    }
    pub fn sheet(path: String, private: bool) -> Self {
        let mut r = Self::prepare(path, "GET", None, None, private);
        r.sheet = true;
        r.dispatch();
        r
    }
    fn dispatch(&mut self) {
        self.slot = Arc::default();
        let allowed = self.path.len() <= 2048 && allowed(&self.path, self.method);
        let origin = super::transport::origin(&crate::config::server_url()).ok();
        if !allowed || origin.is_none() || self.private && self.access.is_none() {
            if let Ok(mut slot) = self.slot.lock() {
                *slot = Some(Ok(Reply {
                    status: 401,
                    value: Value::Null,
                    bytes: None,
                }));
            }
            return;
        }
        let body = serde_json::json!({"url":format!("{}{}",origin.unwrap_or_default(),self.path),"method":self.method,
            "body":self.body,"bytes":self.raw,"mime":self.mime,"sheet":self.sheet,"revision":self.revision,"slot":super::slot().map(String::from),
            "access":if self.private {self.access.map(|a|a.0.iter().map(|b|format!("{b:02x}")).collect::<String>())}else{None}});
        start(body, self.slot.clone());
    }
    pub fn take(&mut self, now: f64) -> Option<Result<Reply, ()>> {
        if self.epoch != epoch() {
            return Some(Err(()));
        }
        if let Some(until) = self.wait_until {
            let view = snapshot();
            if view.access != self.access
                && matches!(
                    view.status,
                    Status::Authenticated | Status::PersistenceUnavailable
                )
            {
                self.access = view.access;
                self.wait_until = None;
                self.dispatch();
            } else if now >= until || view.status == Status::ReauthRequired {
                return Some(Ok(Reply {
                    status: 401,
                    value: Value::Null,
                    bytes: None,
                }));
            } else {
                return None;
            }
        }
        let result = self.slot.lock().ok()?.take()?;
        if result.as_ref().is_ok_and(|r| r.status == 401) && self.private && !self.retried {
            self.retried = true;
            super::authentication_required(tbg_core::protocol::AuthRefusal::Expired);
            if self.method == "GET" {
                self.wait_until = Some(now + 8.);
                return None;
            }
        }
        Some(result)
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn start(value: Value, slot: Slot) {
    std::thread::spawn(move || {
        let result = (|| {
            let agent = ureq::Agent::config_builder()
                .max_redirects(0)
                .http_status_as_error(false)
                .timeout_global(Some(std::time::Duration::from_secs(2)))
                .build()
                .new_agent();
            let mut builder = ureq::http::Request::builder()
                .method(value["method"].as_str().ok_or(())?)
                .uri(value["url"].as_str().ok_or(())?)
                .header(
                    "Content-Type",
                    value["mime"].as_str().unwrap_or("application/json"),
                )
                .header("X-TBG-Client", "native")
                .header("Cache-Control", "no-store");
            if let Some(slot) = value["slot"].as_str() {
                builder = builder.header("X-TBG-Slot", slot);
            }
            if let Some(rev) = value["revision"].as_i64() {
                builder = builder.header("If-Match", format!("\"{rev}\""));
            }
            if let Some(access) = value["access"].as_str() {
                let mut header = ureq::http::HeaderValue::from_str(&format!("Bearer {access}"))
                    .map_err(|_| ())?;
                header.set_sensitive(true);
                builder = builder.header("Authorization", header);
            }
            let bytes = if let Some(raw) = value["bytes"].as_array() {
                if raw.len() > 1048576 {
                    return Err(());
                }
                raw.iter()
                    .map(|n| n.as_u64().filter(|v| *v <= 255).map(|v| v as u8).ok_or(()))
                    .collect::<Result<Vec<_>, _>>()?
            } else if value["body"].is_null() {
                Vec::new()
            } else {
                serde_json::to_vec(&value["body"]).map_err(|_| ())?
            };
            let mut response = agent
                .run(builder.body(bytes).map_err(|_| ())?)
                .map_err(|_| ())?;
            let status = response.status().as_u16();
            if (300..400).contains(&status) {
                return Err(());
            }
            let bytes = response
                .body_mut()
                .with_config()
                .limit(262144)
                .read_to_vec()
                .map_err(|_| ())?;
            Ok(Reply {
                status,
                value: if value["sheet"] == true {
                    Value::Null
                } else {
                    serde_json::from_slice(&bytes).map_err(|_| ())?
                },
                bytes: (value["sheet"] == true).then_some(bytes),
            })
        })();
        if let Ok(mut output) = slot.lock() {
            *output = Some(result);
        }
    });
}
#[cfg(target_arch = "wasm32")]
fn start(value: Value, slot: Slot) {
    use wasm_bindgen::prelude::*;
    #[wasm_bindgen(module = "/assets/auth/history.js")]
    extern "C" {
        #[wasm_bindgen(catch,js_name=historyRequest)]
        fn request(raw: &str, callback: &JsValue) -> Result<(), JsValue>;
    }
    let output = slot.clone();
    let callback = Closure::once_into_js(move |raw: String| {
        let result = serde_json::from_str::<Value>(&raw)
            .ok()
            .and_then(|v| {
                Some(Reply {
                    status: u16::try_from(v["status"].as_u64()?).ok()?,
                    value: v["value"].clone(),
                    bytes: v["bytes"].as_array().and_then(|a| {
                        a.iter()
                            .map(|v| v.as_u64().filter(|v| *v <= 255).map(|v| v as u8))
                            .collect()
                    }),
                })
            })
            .ok_or(());
        if let Ok(mut output) = output.lock() {
            *output = Some(result);
        }
    });
    if request(&value.to_string(), &callback).is_err() {
        if let Ok(mut output) = slot.lock() {
            *output = Some(Err(()));
        }
    }
}

/// Exact mutation surface; widening player-history reads never authorizes an arbitrary write.
pub fn allowed(path: &str, method: &str) -> bool {
    let path = path.split('?').next().unwrap_or("");
    match method {
        "GET" => {
            path.starts_with("/api/me/")
                || path.starts_with("/api/players/")
                || path.starts_with("/api/emoji/")
        }
        "PATCH" => path == "/api/me/preferences/history",
        "POST" => matches!(
            path,
            "/api/me/email"
                | "/api/me/email/verify"
                | "/api/me/email/resend"
                | "/api/me/email/change/confirm"
                | "/api/me/email/change/cancel"
                | "/api/me/reports"
                | "/api/me/emoji"
                | "/api/auth/native/recovery-request"
                | "/api/auth/browser/recovery-request"
        ),
        "PUT" => path == "/api/me/emoji/slots",
        "DELETE" => path
            .strip_prefix("/api/me/emoji/")
            .is_some_and(|id| !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit())),
        _ => false,
    }
}

#[cfg(test)]
mod p75_tests {
    use super::*;
    #[test]
    fn p75_mutations_remain_on_the_exact_allowlist() {
        assert!(allowed("/api/me/emoji/slots", "PUT"));
        assert!(allowed("/api/auth/native/recovery-request", "POST"));
        for (path, method) in [
            ("/api/players/Victim", "POST"),
            ("/admin-api/v1/settings/chat_enabled", "PUT"),
            ("/api/me/matches/1", "DELETE"),
            ("/api/me/emoji/1/sheet.png", "DELETE"),
        ] {
            assert!(!allowed(path, method));
        }
    }
}
