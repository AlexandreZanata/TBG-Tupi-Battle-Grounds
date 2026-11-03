//! System EditText uses the same Rust form submission/validation and localized labels.

use crate::{
    i18n::Lang,
    platform::android::services,
    ui::textform::{self, ActiveForm, TextKind},
};
use bevy::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};
static REQUEST: AtomicU64 = AtomicU64::new(0);
pub fn poll(
    form: NonSend<ActiveForm>,
    lang: Res<Lang>,
    mut shown: Local<(Option<TextKind>, u64, String)>,
) {
    let error = form.error.lock().map(|e| e.clone()).unwrap_or_default();
    let identity = (form.kind, form.generation, error.clone());
    if identity == *shown {
        return;
    }
    *shown = identity;
    if let Some(kind) = form.kind {
        let id = REQUEST.fetch_add(1, Ordering::SeqCst) + 1;
        let request = serde_json::json!({"id":id,"limit":match kind {TextKind::Name=>16,TextKind::Room=>5,_=>128},"heading":match kind {TextKind::Name=>lang.key("lobby.account.create"),TextKind::Key=>lang.key("lobby.account.key"),TextKind::BackupKey=>lang.key("android.account.backup_title"),_=>form.prompt.clone()},"reveal_short":lang.key("android.reveal_short"),"secret":form.masked,"value":form.edit.text,"title":if error.is_empty() {form.prompt.clone()} else {format!("{} · {}",form.prompt,error)},"done":lang.key("android.done"),"cancel":lang.key("android.cancel"),"reveal":lang.key("android.reveal")});
        services::send(jni::jni_str!("showTextForm"), &request.to_string());
    }
}
pub fn commit(form: &mut ActiveForm, id: u64, value: &str) {
    if id != REQUEST.load(Ordering::SeqCst) || form.kind.is_none() {
        return;
    }
    let limit = match form.kind {
        Some(TextKind::Name) => 16,
        Some(TextKind::Room) => 5,
        _ => 128,
    };
    if value.chars().count() > limit {
        return;
    }
    form.edit = Default::default();
    form.edit.insert(value, limit, |_| true);
    textform::submit(form);
}

pub fn cancel(form: &ActiveForm, id: u64) {
    if id == REQUEST.load(Ordering::SeqCst) {
        textform::cancel(form);
    }
}
