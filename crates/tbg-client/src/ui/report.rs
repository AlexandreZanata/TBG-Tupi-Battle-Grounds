//! Report composition; evidence is always selected by the server.
use serde_json::{json, Value};
pub const REASONS: [&str; 8] = [
    "cheating",
    "abusive_chat",
    "offensive_name",
    "offensive_emoji",
    "harassment",
    "griefing",
    "impersonation",
    "other",
];
pub struct Flow {
    pub context: Value,
    pub reason: u8,
    pub comment: String,
}
impl Default for Flow {
    fn default() -> Self {
        Self {
            context: json!({}),
            reason: 0,
            comment: String::new(),
        }
    }
}
impl Flow {
    pub fn body(&self) -> Value {
        let mut body = self.context.clone();
        body["reason"] = json!(REASONS[usize::from(self.reason.min(7))]);
        if !self.comment.is_empty() {
            body["comment"] = json!(self.comment);
        }
        body
    }
}
