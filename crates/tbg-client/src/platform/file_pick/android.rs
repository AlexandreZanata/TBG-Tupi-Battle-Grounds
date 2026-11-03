//! Correlate one explicit Android document selection without file paths or persistence.
use super::*;
use crate::platform::android::services;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(0);
static PENDING: Mutex<Option<(u64, Output)>> = Mutex::new(None);

pub(super) fn choose(output: Output) {
    let id = NEXT.fetch_add(1, Ordering::Relaxed) + 1;
    if let Ok(mut pending) = PENDING.lock() {
        *pending = Some((id, output.clone()));
    } else {
        complete(output, Err(()));
        return;
    }
    if !services::send(jni::jni_str!("chooseImage"), &id.to_string()) {
        finish(id, "", false);
    }
}

pub(crate) fn finish(id: u64, name: &str, success: bool) {
    let output = PENDING.lock().ok().and_then(|mut pending| {
        if pending.as_ref().is_some_and(|(current, _)| *current == id) {
            pending.take().map(|(_, output)| output)
        } else {
            None
        }
    });
    let Some(output) = output else { return };
    let value = if success && !name.is_empty() && name.len() <= 256 {
        services::bytes(jni::jni_str!("takePickedImage"), 1048576)
            .filter(|bytes| !bytes.is_empty())
            .map(|bytes| (bytes, name.to_owned()))
            .ok_or(())
    } else {
        Err(())
    };
    complete(output, value);
}

fn complete(output: Output, value: Result<(Vec<u8>, String), ()>) {
    OCCUPIED.store(false, Ordering::Release);
    if let Ok(mut slot) = output.lock() {
        *slot = Some(value);
    }
}
