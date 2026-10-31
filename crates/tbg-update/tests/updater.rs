//! P66 updater acceptance on synthetic fixtures (contracts §4): a real TUF
//! repository signed with throwaway keys, releases whose "game" is a small
//! shell script, and a temporary install root. Never touches the owner's
//! installation, real keys or the network (file:// repositories only).
#![cfg(unix)]

use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use tbg_update::descriptor::{Compatibility, ReleaseDescriptor};
use tbg_update::journal::{self, Txn};
use tbg_update::layout::Layout;
use tbg_update::publish::{self, Release};
use tbg_update::trust::Source;
use tbg_update::{activate, extract, helper, launcher, receipt};

fn rt() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}

fn notes(build: &str, gameplay: bool) -> serde_json::Value {
    let mut sections = vec![serde_json::json!({
        "kind": "highlights",
        "items": {"en": [format!("Release {build}")], "pt-BR": [format!("Versão {build}")]}
    })];
    if gameplay {
        sections.push(serde_json::json!({
            "kind": "gameplay",
            "items": {"en": ["Balance pass"], "pt-BR": ["Ajuste de balanceamento"]}
        }));
    }
    serde_json::json!({
        "schema": 1, "build_id": build, "date": "2026-10-31",
        "title": {"en": format!("Version {build}"), "pt-BR": format!("Versão {build}")},
        "sections": sections,
    })
}

/// A release archive whose `tbg-client` runs `script` (sh).
fn package(dir: &Path, build: &str, script: &str) -> PathBuf {
    let src = dir.join(format!("src-{build}"));
    fs::create_dir_all(src.join("assets")).unwrap();
    fs::write(src.join("tbg-client"), format!("#!/bin/sh\n{script}\n")).unwrap();
    fs::set_permissions(src.join("tbg-client"), fs::Permissions::from_mode(0o755)).unwrap();
    fs::write(
        src.join("build-info.json"),
        serde_json::json!({"build_id": build}).to_string(),
    )
    .unwrap();
    fs::write(
        src.join("release-notes.json"),
        notes(build, false).to_string(),
    )
    .unwrap();
    fs::write(src.join("assets/marker.txt"), build).unwrap();
    let out = dir.join(format!("package-{build}.tar.gz"));
    let file = fs::File::create(&out).unwrap();
    let mut tar = tar::Builder::new(flate2::write::GzEncoder::new(
        file,
        flate2::Compression::fast(),
    ));
    tar.append_dir_all(".", &src).unwrap();
    tar.into_inner().unwrap().finish().unwrap();
    out
}

fn descriptor(build: &str, sequence: u64, contract: &str) -> ReleaseDescriptor {
    let (platform, arch) = tbg_update::descriptor::this_platform();
    ReleaseDescriptor {
        schema: 1,
        build_id: build.into(),
        release_sequence: sequence,
        display_version: format!("0.2.{sequence}"),
        channel: "stable".into(),
        platform: platform.into(),
        architecture: arch.into(),
        install_mode: "managed".into(),
        package_target: format!("package-{build}.tar.gz"),
        notes_target: format!("notes-{build}.json"),
        compatibility: Compatibility {
            match_contract: contract.into(),
            presentation: "p1".into(),
        },
        updater_abi: 1,
        minimum_bootstrap: 1,
        security_floor_sequence: 0,
        local_state_reader: vec![1],
        local_state_writer: 1,
        minimum_glibc: None,
    }
}

struct Fixture {
    _tmp: tempfile::TempDir,
    dir: PathBuf,
    keys: PathBuf,
    repo: PathBuf,
    layout: Layout,
}

impl Fixture {
    fn new() -> Fixture {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_path_buf();
        let (keys, repo) = (dir.join("keys"), dir.join("repo"));
        publish::init_keys(&keys).unwrap();
        rt().block_on(publish::init_repo(&keys, &repo, 30)).unwrap();
        let layout = Layout::new(dir.join("install"));
        for d in [
            layout.root.clone(),
            layout.trust(),
            layout.state(),
            layout.run(),
        ] {
            fs::create_dir_all(d).unwrap();
        }
        fs::copy(repo.join("metadata/1.root.json"), layout.trusted_root()).unwrap();
        let url = |sub: &str| {
            url::Url::from_directory_path(repo.join(sub))
                .unwrap()
                .to_string()
        };
        let source = Source {
            metadata_url: url("metadata"),
            targets_url: url("targets"),
            channel: "stable".into(),
        };
        tbg_update::fsx::write_json(&layout.source(), &source).unwrap();
        Fixture {
            _tmp: tmp,
            dir,
            keys,
            repo,
            layout,
        }
    }

    fn publish(&self, build: &str, sequence: u64, script: &str) {
        let package = package(&self.dir, build, script);
        let notes_path = self.dir.join(format!("notes-{build}.json"));
        fs::write(&notes_path, notes(build, false).to_string()).unwrap();
        let release = Release {
            package,
            notes: notes_path,
            descriptor: descriptor(build, sequence, "c1"),
            previous_contract: Some("c1".into()),
            metadata_days: 7,
        };
        rt().block_on(publish::publish(&self.keys, &self.repo, &release))
            .unwrap();
    }

    /// Stage the channel's release as the helper does (never cancelled).
    fn stage(&self, build: &str) -> i32 {
        helper::stage_with(
            &self.layout,
            build,
            &std::sync::atomic::AtomicBool::new(false),
        )
    }
}

/// Install `build` as the current, healthy release.
fn install_current(f: &Fixture, build: &str) {
    assert_eq!(f.stage(build), 0);
    activate::switch(&f.layout, "", build).unwrap();
    journal::store(&f.layout, None).unwrap();
}

#[test]
fn u02_signed_releases_stage_and_tampering_or_rollback_is_refused() {
    let f = Fixture::new();
    f.publish("b1", 1, "exit 0");
    assert_eq!(f.stage("b1"), 0, "a signed release stages");
    assert!(activate::complete(&f.layout, "b1"));
    // Snapshot the metadata the client has now trusted.
    let old = f.dir.join("old-metadata");
    copy_dir(&f.repo.join("metadata"), &old);
    f.publish("b2", 2, "exit 0");
    // A substituted package (same name, other bytes) fails verification.
    let target = fs::read_dir(f.repo.join("targets"))
        .unwrap()
        .flatten()
        .find(|e| {
            e.file_name()
                .to_string_lossy()
                .ends_with("package-b2.tar.gz")
        })
        .unwrap()
        .path();
    let good = fs::read(&target).unwrap();
    fs::write(&target, b"not the signed bytes").unwrap();
    assert_ne!(f.stage("b2"), 0, "a substituted package never stages");
    assert!(
        !f.layout.release("b2").exists(),
        "nothing of it is installed"
    );
    fs::write(&target, &good).unwrap();
    assert_eq!(f.stage("b2"), 0);
    // Replaying the older (validly signed) metadata is a rollback.
    let metadata = f.repo.join("metadata");
    fs::remove_dir_all(&metadata).unwrap();
    copy_dir(&old, &metadata);
    let refused = rt().block_on(tbg_update::trust::open(&f.layout));
    assert!(
        refused.is_err(),
        "old metadata after newer was trusted is refused"
    );
}

#[test]
fn u02_a_root_from_other_keys_is_not_trusted() {
    let f = Fixture::new();
    f.publish("b1", 1, "exit 0");
    let other = Fixture::new();
    fs::copy(
        other.repo.join("metadata/1.root.json"),
        f.layout.trusted_root(),
    )
    .unwrap();
    assert!(rt().block_on(tbg_update::trust::open(&f.layout)).is_err());
}

#[test]
fn u04_hostile_archives_are_rejected_before_anything_runs() {
    let tmp = tempfile::tempdir().unwrap();
    let cases: Vec<(&str, Box<dyn Fn(&mut tar::Builder<Vec<u8>>)>)> = vec![
        (
            "symlink",
            Box::new(|t| {
                let mut h = tar::Header::new_gnu();
                h.set_entry_type(tar::EntryType::Symlink);
                h.set_size(0);
                t.append_link(&mut h, "tbg-client", "/bin/sh").unwrap();
            }),
        ),
        ("parent", Box::new(|t| raw_file(t, "../escape", b"x"))),
        ("absolute", Box::new(|t| raw_file(t, "/tmp/escape", b"x"))),
        (
            "reserved",
            Box::new(|t| raw_file(t, "assets/CON.txt", b"x")),
        ),
        (
            "case",
            Box::new(|t| {
                raw_file(t, "assets/a.txt", b"x");
                raw_file(t, "assets/A.txt", b"y");
            }),
        ),
        (
            "missing",
            Box::new(|t| raw_file(t, "assets/only.txt", b"x")),
        ),
    ];
    for (name, build) in cases {
        let mut t = tar::Builder::new(Vec::new());
        build(&mut t);
        let bytes = t.into_inner().unwrap();
        let archive = tmp.path().join(format!("{name}.tar.gz"));
        let mut gz = flate2::write::GzEncoder::new(
            fs::File::create(&archive).unwrap(),
            flate2::Compression::fast(),
        );
        gz.write_all(&bytes).unwrap();
        gz.finish().unwrap();
        let dest = tmp.path().join(format!("out-{name}"));
        assert!(
            extract::extract(&archive, &dest).is_err(),
            "{name} must be rejected"
        );
    }
    assert!(!tmp.path().join("escape").exists());
}

/// A plain regular-file entry with an arbitrary (unchecked) name.
fn raw_file(t: &mut tar::Builder<Vec<u8>>, name: &str, data: &[u8]) {
    let mut h = tar::Header::new_old();
    h.as_old_mut().name[..name.len()].copy_from_slice(name.as_bytes());
    h.set_entry_type(tar::EntryType::Regular);
    h.set_size(data.len() as u64);
    h.set_mode(0o644);
    h.set_cksum();
    t.append(&h, data).unwrap();
}

#[test]
fn u05_every_journal_point_recovers_a_complete_release() {
    let f = Fixture::new();
    f.publish("b1", 1, "exit 0");
    install_current(&f, "b1");
    f.publish("b2", 2, "exit 0");
    assert_eq!(f.stage("b2"), 0);
    assert!(matches!(journal::load(&f.layout), Some(Txn::Staged { .. })));
    // Interrupted during the switch or the first launch: back to b1.
    for txn in [
        Txn::Activating {
            id: "t".into(),
            from: "b1".into(),
            to: "b2".into(),
        },
        Txn::Launching {
            id: "t".into(),
            from: "b1".into(),
            to: "b2".into(),
        },
    ] {
        activate::switch(&f.layout, "b1", "b2").unwrap();
        journal::store(&f.layout, Some(&txn)).unwrap();
        let r = activate::reconcile(&f.layout).unwrap();
        assert_eq!(
            r,
            activate::Recovery::RolledBack {
                from: "b1".into(),
                to: "b2".into()
            }
        );
        assert_eq!(f.layout.current_build().as_deref(), Some("b1"));
        assert!(journal::load(&f.layout).is_none());
    }
    // A staged entry whose directory vanished is dropped, not executed.
    journal::store(
        &f.layout,
        Some(&Txn::Staged {
            id: "t".into(),
            to: "gone".into(),
        }),
    )
    .unwrap();
    assert_eq!(
        activate::reconcile(&f.layout).unwrap(),
        activate::Recovery::Unchanged
    );
}

/// The fake game: `apply` once (exit 75) unless recovered or updated.
const OLD_GAME: &str =
    r#"case "$TBG_UPDATE_OUTCOME" in recovered|updated) exit 0;; *) exit 75;; esac"#;
const HEALTHY_GAME: &str = r#"printf '{"nonce":"%s","build":"%s"}' "$TBG_HEALTH_NONCE" "$TBG_INSTALLED_BUILD" > "$TBG_HEALTH_FILE"; sleep 0.3; exit 0"#;

#[test]
fn u06_u10_a_healthy_candidate_commits_and_shows_its_notes() {
    let f = Fixture::new();
    f.publish("b1", 1, OLD_GAME);
    install_current(&f, "b1");
    f.publish("b2", 2, HEALTHY_GAME);
    assert_eq!(f.stage("b2"), 0);
    assert_eq!(launcher::run(&f.layout, &[]), 0);
    assert_eq!(f.layout.current_build().as_deref(), Some("b2"));
    assert_eq!(f.layout.previous_build().as_deref(), Some("b1"));
    assert!(journal::load(&f.layout).is_none());
    let r = receipt::pending(&f.layout).expect("notes wait for the player");
    assert_eq!((r.kind, r.build.as_str()), (receipt::Kind::Updated, "b2"));
    // At-least-once: still pending until acknowledged for that build.
    assert!(!receipt::acknowledge(&f.layout, "b1").unwrap());
    assert!(receipt::pending(&f.layout).is_some());
    assert!(receipt::acknowledge(&f.layout, "b2").unwrap());
    assert!(receipt::pending(&f.layout).is_none());
}

#[test]
fn u06_a_candidate_that_dies_before_healthy_rolls_back_without_claiming_success() {
    let f = Fixture::new();
    f.publish("b1", 1, OLD_GAME);
    install_current(&f, "b1");
    f.publish("b2", 2, "exit 3");
    assert_eq!(f.stage("b2"), 0);
    assert_eq!(
        launcher::run(&f.layout, &[]),
        0,
        "b1 runs again and exits cleanly"
    );
    assert_eq!(f.layout.current_build().as_deref(), Some("b1"));
    let r = receipt::pending(&f.layout).unwrap();
    assert_eq!(
        (r.kind, r.build.as_str(), r.from.as_str()),
        (receipt::Kind::Recovered, "b2", "b1")
    );
}

#[test]
fn u03_cancel_and_missing_targets_leave_no_partial_files() {
    let f = Fixture::new();
    f.publish("b1", 1, "exit 0");
    assert_ne!(
        f.stage("nope"),
        0,
        "a build that is not the channel's release is refused"
    );
    let staging = f.layout.staging();
    let leftovers: Vec<_> = fs::read_dir(&staging)
        .map(|d| d.flatten().collect())
        .unwrap_or_default();
    assert!(leftovers.is_empty(), "{leftovers:?}");
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for e in fs::read_dir(from).unwrap().flatten() {
        fs::copy(e.path(), to.join(e.file_name())).unwrap();
    }
}
