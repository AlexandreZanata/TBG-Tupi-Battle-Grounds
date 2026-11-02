//! Account-scoped read-only history state; private data and stale responses are cleared on identity changes.

use super::{
    kit,
    shell::{self, Panel},
    social::{Action, Name},
};
use crate::{
    auth::api::{Reply, Request},
    i18n::Lang,
};
use bevy::prelude::*;
use serde_json::{json, Value};
mod views;
pub(super) use views::date;
pub use views::{players, profile, rematch_card, spawn};

#[derive(Clone, Copy)]
enum Purpose {
    List,
    Detail,
    Profile,
    Save,
    Preference,
}
#[derive(Resource)]
pub struct History {
    pub subject: String,
    pub owner: bool,
    pub rows: Vec<Value>,
    pub detail: Option<Value>,
    pub profile_data: Option<Value>,
    pub error: Option<&'static str>,
    pub preference: Option<(bool, i64)>,
    pub save_state: &'static str,
    pub mode: Option<&'static str>,
    pub outcome: Option<&'static str>,
    pub has_more: bool,
    pub cursor: Option<String>,
    pages: Vec<Option<String>>,
    next_cursor: Option<String>,
    request: Option<(Purpose, Request)>,
    pref_request: Option<(Purpose, Request)>,
    epoch: u64,
    pub backoff: f64,
    pub now: f64,
    pub refresh_result: Option<[u8; 16]>,
}
impl Default for History {
    fn default() -> Self {
        Self {
            subject: String::new(),
            owner: false,
            rows: Vec::new(),
            detail: None,
            profile_data: None,
            error: None,
            preference: None,
            save_state: "",
            mode: None,
            outcome: None,
            has_more: false,
            cursor: None,
            pages: Vec::new(),
            next_cursor: None,
            request: None,
            pref_request: None,
            epoch: crate::auth::epoch(),
            backoff: 0.,
            now: 0.,
            refresh_result: None,
        }
    }
}
impl History {
    pub fn current(&self) -> bool {
        self.epoch == crate::auth::epoch()
    }
    pub fn pending(&self) -> bool {
        self.request.is_some()
    }
    pub fn preference_pending(&self) -> bool {
        self.pref_request.is_some()
    }
    pub fn open(&mut self, name: String, owner: bool) {
        *self = Self {
            subject: name,
            owner,
            ..default()
        };
        self.load();
        if owner {
            self.load_preference();
        }
    }
    pub fn mine(&mut self) {
        if let Some(profile) = crate::auth::snapshot().profile {
            self.open(profile.name, true);
        } else {
            *self = Self {
                error: Some("p73.sign_in"),
                ..default()
            };
        }
    }
    pub fn profile(&mut self, name: String) {
        let owner = crate::auth::snapshot()
            .profile
            .is_some_and(|p| p.name.eq_ignore_ascii_case(&name));
        *self = Self {
            subject: name,
            owner,
            ..default()
        };
        self.request = Some((
            Purpose::Profile,
            Request::new(
                format!("/api/players/{}", self.subject),
                "GET",
                None,
                None,
                false,
            ),
        ));
    }
    fn base(&self) -> String {
        if self.owner {
            "/api/me/matches".into()
        } else {
            format!("/api/players/{}/matches", self.subject)
        }
    }
    fn load(&mut self) {
        if self.subject.is_empty() || self.now < self.backoff {
            return;
        }
        self.error = None;
        self.detail = None;
        let mut path = format!("{}?limit=20", self.base());
        if let Some(cursor) = &self.cursor {
            path.push_str(&format!("&cursor={cursor}"));
        }
        if let Some(mode) = self.mode {
            path.push_str(&format!("&mode={mode}"));
        }
        if let Some(outcome) = self.outcome {
            path.push_str(&format!("&outcome={outcome}"));
        }
        self.request = Some((
            Purpose::List,
            Request::new(path, "GET", None, None, self.owner),
        ));
    }
    fn load_preference(&mut self) {
        self.pref_request = Some((
            Purpose::Preference,
            Request::new(
                "/api/me/preferences/history".into(),
                "GET",
                None,
                None,
                true,
            ),
        ));
    }
    pub fn next(&mut self) {
        if self.pending() {
            return;
        }
        if self.has_more {
            self.pages.push(self.cursor.clone());
            self.cursor = self.next_cursor.clone();
            self.load();
        }
    }
    pub fn previous(&mut self) {
        if self.pending() {
            return;
        }
        if let Some(cursor) = self.pages.pop() {
            self.cursor = cursor;
            self.load();
        }
    }
    pub fn previous_available(&self) -> bool {
        !self.pages.is_empty()
    }
    pub fn return_list(&mut self) {
        self.detail = None;
        self.load();
    }
    pub fn detail(&mut self, id: [u8; 16]) {
        if self.pending() {
            return;
        }
        self.error = None;
        let id = hex(&id);
        self.request = Some((
            Purpose::Detail,
            Request::new(
                format!("{}/{id}", self.base()),
                "GET",
                None,
                None,
                self.owner,
            ),
        ));
    }
    pub fn refresh(&mut self) {
        if !self.pending() {
            self.load();
            if self.owner {
                self.load_preference();
            }
        }
    }
    pub fn mode(&mut self) {
        if self.pending() {
            return;
        }
        self.mode = match self.mode {
            None => Some("casual"),
            Some("casual") => Some("ranked"),
            Some("ranked") => Some("challenge"),
            _ => None,
        };
        self.cursor = None;
        self.pages.clear();
        self.load();
    }
    pub fn outcome(&mut self) {
        if self.pending() {
            return;
        }
        self.outcome = match self.outcome {
            None => Some("win"),
            Some("win") => Some("loss"),
            Some("loss") => Some("draw"),
            _ => None,
        };
        self.cursor = None;
        self.pages.clear();
        self.load();
    }
    pub fn visibility(&mut self, public: bool) {
        if !self.owner || self.preference_pending() {
            return;
        }
        let Some((_, revision)) = self.preference else {
            return;
        };
        self.save_state = "p73.saving";
        self.pref_request = Some((
            Purpose::Save,
            Request::new(
                "/api/me/preferences/history".into(),
                "PATCH",
                Some(json!({"visibility":if public{"public"}else{"private"}})),
                Some(revision),
                true,
            ),
        ));
    }
    pub fn preview(&mut self) {
        let subject = self.subject.clone();
        self.open(subject, false);
    }
    fn accept(&mut self, purpose: Purpose, reply: Result<Reply, ()>) {
        let Ok(reply) = reply else {
            self.error = Some("p73.error");
            if matches!(purpose, Purpose::Save) {
                self.save_state = "p73.save_uncertain";
                self.load_preference();
            }
            return;
        };
        if reply.status == 400 && matches!(purpose, Purpose::List) && self.cursor.is_some() {
            self.cursor = None;
            self.pages.clear();
            self.load();
            return;
        }
        if reply.status != 200 {
            self.error = Some(match reply.status {
                401 => "p73.session_expired",
                403 => "p73.private_history",
                404 => "p73.not_found",
                409 => "p73.privacy_conflict",
                429 => "p73.rate_limited",
                _ => "p73.error",
            });
            if reply.status == 429 {
                self.backoff = self.now + 15.;
            }
            if matches!(reply.status, 401 | 403 | 404) {
                self.rows.clear();
                self.detail = None;
            }
            if matches!(purpose, Purpose::Save) {
                self.save_state = if reply.status == 409 {
                    "p73.privacy_conflict"
                } else {
                    "p73.save_uncertain"
                };
                self.load_preference();
            }
            return;
        }
        let value = reply.value;
        match purpose {
            Purpose::Preference | Purpose::Save => {
                if let (Some(visibility), Some(revision)) =
                    (value["visibility"].as_str(), value["revision"].as_i64())
                {
                    if matches!(visibility, "private" | "public") {
                        self.preference = Some((visibility == "public", revision));
                    }
                }
                if matches!(purpose, Purpose::Save) {
                    self.save_state = "p73.saved";
                    self.cursor = None;
                    self.pages.clear();
                    self.load();
                }
            }
            Purpose::Profile => self.profile_data = Some(value),
            Purpose::List => {
                let Some(rows) = value["rows"].as_array() else {
                    self.error = Some("p73.error");
                    return;
                };
                if rows.len() > 50 {
                    self.error = Some("p73.error");
                    return;
                }
                self.rows = rows.clone();
                self.has_more = value["has_more"].as_bool().unwrap_or(false);
                self.next_cursor = value["cursor"]
                    .as_str()
                    .filter(|s| s.len() <= 512)
                    .map(str::to_owned);
            }
            Purpose::Detail => self.detail = Some(value),
        }
    }
}

pub fn poll(mut history: ResMut<History>, mut shell: ResMut<shell::Shell>, time: Res<Time>) {
    if history.epoch != crate::auth::epoch() {
        *history = History::default();
        shell.dirty = true;
    }
    history.now = time.elapsed_secs_f64();
    let now = history.now;
    let result = history
        .request
        .as_mut()
        .and_then(|(kind, r)| r.take(now).map(|value| (*kind, value)));
    if let Some((kind, result)) = result {
        history.request = None;
        history.accept(kind, result);
        shell.dirty = true;
    }
    let result = history
        .pref_request
        .as_mut()
        .and_then(|(kind, r)| r.take(now).map(|value| (*kind, value)));
    if let Some((kind, result)) = result {
        history.pref_request = None;
        history.accept(kind, result);
        shell.dirty = true;
    }
    if shell.panel == Panel::History && history.owner && history.refresh_result.take().is_some() {
        history.refresh();
        shell.dirty = true;
    }
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
pub fn entry(s: &str) -> Option<[u8; 16]> {
    if s.len() != 32 {
        return None;
    }
    let mut bytes = [0; 16];
    for (i, p) in s.as_bytes().as_chunks::<2>().0.iter().enumerate() {
        bytes[i] = u8::from_str_radix(std::str::from_utf8(p).ok()?, 16).ok()?;
    }
    Some(bytes)
}
fn control(p: &mut ChildSpawnerCommands, key: &str, action: Action, lang: Lang, width: f32) {
    p.spawn(shell::button(
        lang.key(key),
        shell::Action::Social(action),
        width,
    ));
}
fn mode(v: &Value) -> String {
    v.as_str().unwrap_or("unavailable").to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn p73_privacy_is_never_saved_before_acknowledgment_and_conflicts_do_not_retry_public() {
        let mut h = History {
            subject: "FixtureA".into(),
            owner: true,
            preference: Some((false, 3)),
            ..default()
        };
        h.visibility(true);
        assert_eq!(h.preference, Some((false, 3)));
        assert_eq!(h.save_state, "p73.saving");
        h.pref_request = None;
        h.accept(
            Purpose::Save,
            Ok(Reply {
                bytes: None,
                status: 409,
                value: Value::Null,
            }),
        );
        assert_eq!(h.preference, Some((false, 3)));
        assert_eq!(h.save_state, "p73.privacy_conflict");
        assert!(matches!(h.pref_request, Some((Purpose::Preference, _))));
    }
    #[test]
    fn p73_private_or_expired_history_clears_previously_rendered_rows() {
        for status in [401, 403, 404] {
            let mut h = History {
                rows: vec![json!({"entry_id":"fixture"})],
                detail: Some(json!({"private":"fixture"})),
                ..default()
            };
            h.accept(
                Purpose::List,
                Ok(Reply {
                    bytes: None,
                    status,
                    value: Value::Null,
                }),
            );
            assert!(h.rows.is_empty() && h.detail.is_none());
        }
    }
    #[test]
    fn p73_entry_ids_and_names_bind_ui_actions_to_rendered_identity() {
        let id = [73; 16];
        assert_eq!(entry(&hex(&id)), Some(id));
        assert!(entry("../../etc/passwd").is_none());
        assert!(Name::new("<script>").is_none());
        let name = Name::new("FixtureA").unwrap();
        assert_eq!(name.text(), "FixtureA");
    }
}

#[cfg(feature = "ui-acceptance")]
impl History {
    /// Explicit synthetic layout data; no authentication, store or game result is implied.
    pub fn visual_fixture(&mut self, public: bool, detail: bool) {
        let summary: Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/p73-personal-summary.json"
        ))
        .expect("checked local presentation fixture");
        let row = json!({"entry_id":hex(&[73;16]),"ended_unix":summary["ended_unix"],"outcome":"loss","reason":"surrender","mode":"Ranked","origin":"public_queue","arena":"forest","wall_ms":240000,"combat_ticks":4000,"tick_hz":20,"opponent":"FixtureRival","quality":"complete","summary_version":1,"rating_status":"committed","rating_delta":-12,"summary":summary});
        *self = Self {
            subject: "FixtureOwner".into(),
            owner: true,
            rows: vec![row.clone()],
            detail: detail.then_some(row),
            preference: Some((public, 7)),
            save_state: "p73.saved",
            ..default()
        };
    }
}
