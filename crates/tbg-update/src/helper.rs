//! The helper the game talks to (P66 §3.1): `check`, `stage`, `notes`,
//! `ack` and `status`, answering in JSON lines on stdout. The game only
//! renders these events; every network, disk and trust decision stays in
//! this process. `stage` stops at the next chunk when stdin says `cancel`
//! or closes.

use std::io::{BufRead, Write};
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::Arc;

use serde_json::{json, Value};

use crate::descriptor::ReleaseDescriptor;
use crate::journal::{self, Txn};
use crate::layout::{valid_build_id, Layout};
use crate::limits::{DESCRIPTOR_BYTES, NOTES_BYTES, PACKAGE_BYTES};
use crate::{extract, fsx, receipt, trust};

/// The descriptor a staged or installed release keeps next to its files.
pub const DESCRIPTOR_FILE: &str = "release.json";

pub fn emit(event: Value) {
    let mut out = std::io::stdout().lock();
    let _ = writeln!(out, "{event}");
    let _ = out.flush();
}

fn fail(code: &str, detail: impl std::fmt::Display) -> i32 {
    emit(json!({"event": "error", "code": code, "detail": detail.to_string()}));
    1
}

/// The installed release's descriptor (`None` for an adopted legacy build).
pub fn installed(layout: &Layout, build: &str) -> Option<ReleaseDescriptor> {
    fsx::read_json(
        &layout.release(build).join(DESCRIPTOR_FILE),
        DESCRIPTOR_BYTES,
    )
    .ok()
}

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("helper runtime")
}

/// `check`: is a newer eligible release available?
pub fn check(layout: &Layout) -> i32 {
    runtime().block_on(async {
        match tokio::time::timeout(
            std::time::Duration::from_secs(crate::limits::DISCOVERY_SECS),
            check_async(layout),
        )
        .await
        {
            Ok(code) => code,
            Err(_) => fail("unavailable", "signed discovery deadline exceeded"),
        }
    })
}

async fn check_async(layout: &Layout) -> i32 {
    let (repo, source) = match trust::open(layout).await {
        Ok(v) => v,
        Err(e) => return fail("unavailable", e),
    };
    let (pointer, descriptor) = match trust::latest(&repo, &source).await {
        Ok(v) => v,
        Err(e) => return fail("verification", e),
    };
    let floor = match crate::policy::remember(layout, descriptor.security_floor_sequence) {
        Ok(value) => value,
        Err(error) => return fail("disk", error),
    };
    let current = layout.current_build().unwrap_or_default();
    let sequence = installed(layout, &current).map_or(0, |d| d.release_sequence);
    let state = if pointer.release_sequence > sequence && pointer.build_id != current {
        "available"
    } else {
        "current"
    };
    let notes = trust::read_small(&repo, &descriptor.notes_target, NOTES_BYTES)
        .await
        .ok()
        .and_then(|b| serde_json::from_slice::<Value>(&b).ok());
    emit(json!({
        "event": "status",
        "state": state,
        "current": current,
        "build": descriptor.build_id,
        "version": descriptor.display_version,
        "size": trust::target_length(&repo, &descriptor.package_target),
        "required": sequence < floor,
        "contract": descriptor.compatibility.match_contract,
        "notes": notes,
    }));
    0
}

fn stdin_cancel() -> Arc<AtomicBool> {
    let cancel = Arc::new(AtomicBool::new(false));
    let flag = cancel.clone();
    std::thread::spawn(move || {
        for line in std::io::stdin().lock().lines() {
            if line.map_or(true, |l| l.trim() == "cancel") {
                break;
            }
        }
        flag.store(true, Relaxed);
    });
    cancel
}

/// Free bytes on the filesystem holding `path` (Unix); `None` if unknown.
fn free_bytes(path: &std::path::Path) -> Option<u64> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        let c = std::ffi::CString::new(path.as_os_str().as_bytes()).ok()?;
        let mut s: libc::statvfs = unsafe { std::mem::zeroed() };
        // SAFETY: `c` is a valid NUL-terminated path and `s` a writable struct.
        let ok = unsafe { libc::statvfs(c.as_ptr(), &mut s) } == 0;
        ok.then(|| s.f_bavail as u64 * s.f_frsize as u64)
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        None
    }
}

/// `stage <build>`: download, verify, extract and record a release; a
/// `cancel` line (or the game closing stdin) stops it.
pub fn stage(layout: &Layout, build: &str) -> i32 {
    stage_with(layout, build, &stdin_cancel())
}

/// `stage` with an explicit cancel flag (tests, other front ends).
pub fn stage_with(layout: &Layout, build: &str, cancel: &AtomicBool) -> i32 {
    if !valid_build_id(build) {
        return fail("unsupported", "invalid build id");
    }
    runtime().block_on(stage_async(layout, build, cancel))
}

async fn stage_async(layout: &Layout, build: &str, cancel: &AtomicBool) -> i32 {
    let (repo, source) = match tokio::time::timeout(
        std::time::Duration::from_secs(crate::limits::DISCOVERY_SECS),
        trust::open(layout),
    )
    .await
    {
        Err(_) => return fail("unavailable", "metadata discovery deadline exceeded"),
        Ok(result) => match result {
            Ok(v) => v,
            Err(e) => return fail("unavailable", e),
        },
    };
    let descriptor = match tokio::time::timeout(
        std::time::Duration::from_secs(crate::limits::DISCOVERY_SECS),
        trust::latest(&repo, &source),
    )
    .await
    {
        Err(_) => return fail("unavailable", "descriptor discovery deadline exceeded"),
        Ok(result) => match result {
            Ok((_, d)) if d.build_id == build => d,
            Ok(_) => {
                return fail(
                    "unsupported",
                    "that build is no longer the channel's release",
                )
            }
            Err(e) => return fail("verification", e),
        },
    };
    let size = trust::target_length(&repo, &descriptor.package_target).unwrap_or(PACKAGE_BYTES);
    // Package + extracted copy (bounded below by 3x) next to the kept builds.
    if free_bytes(&layout.root).is_some_and(|free| free < size.saturating_mul(4)) {
        return fail(
            "disk",
            format!(
                "needs about {} MiB free",
                size.saturating_mul(4) / (1 << 20)
            ),
        );
    }
    let staging = layout.staging();
    if let Err(e) = fsx::private_dir(&staging) {
        return fail("disk", e);
    }
    let archive = staging.join(format!("{build}.tar.gz.part"));
    let mut last = 0u64;
    let mut progress = |done: u64, total: u64| {
        if done == total || done - last >= 1 << 20 {
            last = done;
            emit(json!({"event": "progress", "done": done, "total": total}));
        }
    };
    let mut attempt = 0;
    loop {
        match trust::download(
            &repo,
            &descriptor.package_target,
            &archive,
            PACKAGE_BYTES,
            cancel,
            &mut progress,
        )
        .await
        {
            Ok(_) => break,
            Err(_) if cancel.load(Relaxed) => {
                emit(json!({"event": "cancelled"}));
                return 2;
            }
            // A digest/length mismatch is not transient: never retried.
            Err(e) if e.contains("failed verification") => return fail("verification", e),
            Err(e) if attempt >= crate::limits::DOWNLOAD_RETRIES => return fail("download", e),
            Err(_) => {
                attempt += 1;
                tokio::time::sleep(std::time::Duration::from_secs(2u64.pow(attempt))).await;
            }
        }
    }
    emit(json!({"event": "verified"}));
    install_staged(layout, build, &archive, &descriptor)
}

/// Extract a verified archive into `releases/<build>` and journal it.
pub fn install_staged(
    layout: &Layout,
    build: &str,
    archive: &std::path::Path,
    d: &ReleaseDescriptor,
) -> i32 {
    let floor = match crate::policy::remember(layout, d.security_floor_sequence) {
        Ok(value) => value,
        Err(error) => return fail("disk", error),
    };
    if d.release_sequence < floor {
        return fail(
            "unsupported",
            "release is below the verified security floor",
        );
    }
    let unpack = layout.staging().join(format!("{build}.d"));
    let _ = std::fs::remove_dir_all(&unpack);
    if let Err(e) = extract::extract(archive, &unpack) {
        let _ = std::fs::remove_dir_all(&unpack);
        let _ = std::fs::remove_file(archive);
        return fail("verification", e);
    }
    let info: Option<Value> = fsx::read_json(&unpack.join("build-info.json"), 64 * 1024).ok();
    if info
        .as_ref()
        .and_then(|v| v.get("build_id")?.as_str().map(String::from))
        .as_deref()
        != Some(build)
    {
        let _ = std::fs::remove_dir_all(&unpack);
        return fail("verification", "build-info.json names another build");
    }
    if let Err(e) = fsx::write_json(&unpack.join(DESCRIPTOR_FILE), d) {
        return fail("disk", e);
    }
    let target = layout.release(build);
    let moved = std::fs::create_dir_all(layout.releases())
        .and_then(|_| {
            if target.exists() {
                std::fs::remove_dir_all(&unpack)
            } else {
                std::fs::rename(&unpack, &target)
            }
        })
        .and_then(|_| fsx::sync_dir(&layout.releases()));
    let _ = std::fs::remove_file(archive);
    if let Err(e) = moved {
        return fail("disk", e);
    }
    let txn = Txn::Staged {
        id: journal::new_id(),
        to: build.to_string(),
    };
    if let Err(e) = journal::store(layout, Some(&txn)) {
        return fail("disk", e);
    }
    emit(json!({"event": "staged", "build": build}));
    0
}

/// `notes`: the pending receipt and the installed notes it refers to.
pub fn notes(layout: &Layout) -> i32 {
    let Some(r) = receipt::pending(layout) else {
        emit(json!({"event": "notes", "pending": false}));
        return 0;
    };
    let shown = match r.kind {
        receipt::Kind::Updated => r.build.clone(),
        receipt::Kind::Recovered => r.from.clone(),
    };
    let notes: Option<Value> = fsx::read_json(
        &layout.release(&r.build).join("release-notes.json"),
        NOTES_BYTES,
    )
    .ok();
    emit(json!({
        "event": "notes",
        "pending": true,
        "kind": r.kind,
        "build": r.build,
        "running": shown,
        "notes": notes,
    }));
    0
}

/// `ack <build>`: the player read the notes.
pub fn ack(layout: &Layout, build: &str) -> i32 {
    match receipt::acknowledge(layout, build) {
        Ok(done) => {
            emit(json!({"event": "acknowledged", "build": build, "changed": done}));
            0
        }
        Err(e) => fail("disk", e),
    }
}

/// `status`: selection and journal, for diagnostics and the game.
pub fn status(layout: &Layout) -> i32 {
    emit(json!({
        "event": "installed",
        "current": layout.current_build(),
        "previous": layout.previous_build(),
        "staged": match journal::load(layout) {
            Some(Txn::Staged { to, .. }) => Some(to),
            _ => None,
        },
    }));
    0
}
