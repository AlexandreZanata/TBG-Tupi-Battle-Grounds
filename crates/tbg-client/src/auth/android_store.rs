//! Android uses the reviewed auth coordinator with process-only metadata and no vault.

use super::types::Failure;
use serde_json::{json, Value};
use std::sync::{LazyLock, Mutex};
static META: LazyLock<Mutex<Value>> = LazyLock::new(|| Mutex::new(Value::Null));
pub struct Store {
    pub meta: Value,
}
pub fn nonce(bytes: usize) -> Result<String, Failure> {
    let mut raw = vec![0; bytes];
    getrandom::fill(&mut raw).map_err(|_| Failure::Unavailable)?;
    Ok(raw.iter().map(|v| format!("{v:02x}")).collect())
}
impl Store {
    pub fn open(origin: &str) -> Result<Self, Failure> {
        let mut meta = META.lock().map_err(|_| Failure::Unavailable)?;
        if meta.get("origin").and_then(Value::as_str) != Some(origin) {
            *meta = json!({"schema":1,"origin":origin,"device":nonce(16)?,"remember":false,"logout_pending":false});
        }
        Ok(Self { meta: meta.clone() })
    }
    pub fn device(&self) -> Result<&str, Failure> {
        self.meta["device"].as_str().ok_or(Failure::Unavailable)
    }
    pub fn account(&self) -> Option<&str> {
        self.meta["account"].as_str()
    }
    pub fn save(&self) -> Result<(), Failure> {
        *META.lock().map_err(|_| Failure::Unavailable)? = self.meta.clone();
        Ok(())
    }
    pub fn namespace(&self, origin: &str, account: &str) -> Result<String, Failure> {
        Ok(format!("{origin}:{account}:{}", self.device()?))
    }
    pub fn resume_namespace(&self, origin: &str) -> Result<String, Failure> {
        self.namespace(origin, "~resume")
    }
}
