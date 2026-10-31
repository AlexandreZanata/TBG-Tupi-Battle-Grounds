//! Ruleset identity contract (P61 §3, D-88): every core source is classified,
//! the watched list matches the hash input, and the current hash equals the
//! last append-only registry entry with a strictly increasing revision.

use std::path::{Path, PathBuf};

use tbg_core::ruleset_version::{ruleset_source_hash, RULESET_VERSION, WATCHED};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn sources(dir: &Path, base: &Path, out: &mut Vec<String>) {
    for entry in std::fs::read_dir(dir).expect("read src").flatten() {
        let path = entry.path();
        if path.is_dir() {
            sources(&path, base, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            let rel = path.strip_prefix(base).expect("under src");
            out.push(rel.to_string_lossy().replace('\\', "/"));
        }
    }
}

fn classification() -> (Vec<String>, Vec<String>) {
    let text = std::fs::read_to_string(root().join("ruleset-sources.txt")).expect("list");
    let (mut watch, mut exclude) = (Vec::new(), Vec::new());
    for line in text
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
    {
        let mut parts = line.splitn(3, ' ');
        match (parts.next(), parts.next(), parts.next()) {
            (Some("watch"), Some(p), None) => watch.push(p.to_string()),
            (Some("exclude"), Some(p), Some(reason)) if !reason.trim().is_empty() => {
                exclude.push(p.to_string())
            }
            _ => panic!("malformed classification line: {line}"),
        }
    }
    (watch, exclude)
}

#[test]
fn every_core_source_is_classified() {
    let (watch, exclude) = classification();
    let mut found = Vec::new();
    sources(&root().join("src"), &root().join("src"), &mut found);
    for path in &found {
        assert!(
            watch.contains(path) || exclude.contains(path),
            "{path}: classify it in ruleset-sources.txt (watch or exclude with a reason)"
        );
    }
    for path in &watch {
        assert!(found.contains(path), "watched path {path} does not exist");
    }
    let hashed: Vec<String> = WATCHED.iter().map(|(p, _)| p.to_string()).collect();
    assert_eq!(hashed, watch, "WATCHED must equal the watch list, in order");
}

#[test]
fn current_hash_is_the_last_registered_revision() {
    let text = std::fs::read_to_string(root().join("tests/fixtures/ruleset-versions.txt"))
        .expect("registry");
    let entries: Vec<(u32, String)> = text
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(|l| {
            let (rev, hash) = l.split_once(' ').expect("revision hash");
            (rev.parse().expect("revision"), hash.to_string())
        })
        .collect();
    for pair in entries.windows(2) {
        assert!(pair[0].0 < pair[1].0, "revisions strictly increase");
        assert_ne!(pair[0].1, pair[1].1, "a revision never reuses a hash");
    }
    let (rev, hash) = entries.last().expect("at least one revision");
    assert_eq!(*rev, RULESET_VERSION);
    assert_eq!(
        *hash,
        format!("{:016x}", ruleset_source_hash()),
        "a watched gameplay source changed: append the next revision"
    );
}
