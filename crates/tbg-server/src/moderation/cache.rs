//! Bounded cross-slot read projection for admission and public visibility.
use crate::store::moderation::State;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, RwLock},
};
#[derive(Clone, Default)]
pub struct Cache(pub(crate) Arc<RwLock<Snapshot>>);
impl std::fmt::Debug for Cache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ModerationCache")
    }
}
#[derive(Default)]
pub(crate) struct Snapshot {
    pub states: BTreeMap<i64, State>,
    pub names: BTreeMap<String, i64>,
    pub settings: BTreeMap<String, bool>,
    pub terms: BTreeSet<String>,
    pub cursor: i64,
    pub approved: std::collections::BTreeSet<[u8; 16]>,
}
impl Cache {
    pub fn cursor(&self) -> i64 {
        self.0.read().map(|s| s.cursor).unwrap_or(0)
    }
    pub fn approved(&self, id: &[u8; 16]) -> bool {
        self.0.read().is_ok_and(|s| s.approved.contains(id))
    }
    pub fn banned(&self, id: i64, now: i64) -> bool {
        self.0
            .read()
            .map(|s| s.states.get(&id).is_some_and(|v| v.banned(now)))
            .unwrap_or(true)
    }
    pub fn hidden(&self, name: &str, now: i64) -> bool {
        self.0
            .read()
            .map(|s| {
                s.names
                    .get(&name.to_ascii_lowercase())
                    .and_then(|id| s.states.get(id))
                    .is_some_and(|v| v.banned(now))
            })
            .unwrap_or(true)
    }
    pub fn state(&self, id: i64) -> State {
        self.0
            .read()
            .ok()
            .and_then(|s| s.states.get(&id).copied())
            .unwrap_or_default()
    }
    pub fn enabled(&self, key: &str) -> bool {
        self.0
            .read()
            .ok()
            .and_then(|s| s.settings.get(key).copied())
            .unwrap_or(matches!(
                key,
                "emoji_enabled" | "reports_enabled" | "registration_enabled"
            ))
    }
    pub fn terms(&self) -> Vec<String> {
        self.0
            .read()
            .map(|s| s.terms.iter().cloned().collect())
            .unwrap_or_default()
    }
}
