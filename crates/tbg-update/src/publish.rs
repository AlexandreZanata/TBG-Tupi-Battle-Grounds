//! Release tooling (P66 §4, Step 7): generate role keys, create a signed
//! TUF root, and publish one release (package, notes, descriptor, channel
//! pointer) as new signed metadata with consistent snapshots. Targets are
//! copied before the metadata that names them is written, so a client
//! never sees metadata for a missing file. Production keys and their
//! custody are an owner decision (O02); this tool works with any key
//! files it is given and never stores keys in the repository output.

use std::collections::HashMap;
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};

use aws_lc_rs::rand::SystemRandom;
use aws_lc_rs::signature::Ed25519KeyPair;
use tough::editor::signed::{PathExists, SignedRole};
use tough::editor::RepositoryEditor;
use tough::key_source::{KeySource, LocalKeySource};
use tough::schema::{KeyHolder, RoleKeys, RoleType, Root};
use tough::sign::{parse_keypair, Sign};
use tough::{RepositoryLoader, TargetName};
use url::Url;

use crate::descriptor::{ChannelPointer, ReleaseDescriptor};
use crate::fsx;
use crate::notes::Notes;

pub const ROLES: [&str; 4] = ["root", "targets", "snapshot", "timestamp"];

fn key_path(dir: &Path, role: &str) -> PathBuf {
    dir.join(format!("{role}.pk8"))
}

/// Generate one Ed25519 key per role (PKCS#8, owner-only files).
pub fn init_keys(dir: &Path) -> Result<(), String> {
    fsx::private_dir(dir).map_err(|e| e.to_string())?;
    let rng = SystemRandom::new();
    for role in ROLES {
        let path = key_path(dir, role);
        if path.exists() {
            return Err(format!(
                "{} exists; keys are never overwritten",
                path.display()
            ));
        }
        let doc = Ed25519KeyPair::generate_pkcs8(&rng).map_err(|e| e.to_string())?;
        fsx::write_atomic(&path, doc.as_ref()).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn source(dir: &Path, role: &str) -> Box<dyn KeySource> {
    Box::new(LocalKeySource {
        path: key_path(dir, role),
    })
}

fn expires(days: i64) -> Result<jiff::Timestamp, String> {
    jiff::Timestamp::now()
        .checked_add(jiff::SignedDuration::from_hours(days * 24))
        .map_err(|e| e.to_string())
}

/// Create `repo/metadata/1.root.json` signed by the root key, naming one
/// key per role (threshold 1: staging; production thresholds are O02).
pub async fn init_repo(keys: &Path, repo: &Path, root_days: i64) -> Result<(), String> {
    let mut root = Root {
        spec_version: "1.0.0".into(),
        consistent_snapshot: true,
        version: NonZeroU64::MIN,
        expires: expires(root_days)?,
        keys: HashMap::new(),
        roles: HashMap::new(),
        _extra: HashMap::new(),
    };
    for (role, kind) in ROLES.iter().zip([
        RoleType::Root,
        RoleType::Targets,
        RoleType::Snapshot,
        RoleType::Timestamp,
    ]) {
        let der = fsx::read_bounded(&key_path(keys, role), 64 * 1024).map_err(|e| e.to_string())?;
        let key = parse_keypair(&der).map_err(|e| e.to_string())?.tuf_key();
        let id = key.key_id().map_err(|e| e.to_string())?;
        root.keys.insert(id.clone(), key);
        root.roles.insert(
            kind,
            RoleKeys {
                keyids: vec![id],
                threshold: NonZeroU64::MIN,
                _extra: HashMap::new(),
            },
        );
    }
    let holder = KeyHolder::Root(root.clone());
    let signed = SignedRole::new(root, &holder, &[source(keys, "root")], &SystemRandom::new())
        .await
        .map_err(|e| e.to_string())?;
    let metadata = repo.join("metadata");
    std::fs::create_dir_all(&metadata).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(repo.join("targets")).map_err(|e| e.to_string())?;
    fsx::write_atomic(&metadata.join("1.root.json"), signed.buffer()).map_err(|e| e.to_string())?;
    Ok(())
}

/// One release to publish.
pub struct Release {
    pub package: PathBuf,
    pub notes: PathBuf,
    pub descriptor: ReleaseDescriptor,
    /// The previous stable release's match contract, if any (notes lint).
    pub previous_contract: Option<String>,
    pub metadata_days: i64,
}

/// The newest `N.root.json` in `metadata/`.
fn newest_root(metadata: &Path) -> Result<PathBuf, String> {
    let mut best: Option<(u64, PathBuf)> = None;
    for entry in std::fs::read_dir(metadata)
        .map_err(|e| e.to_string())?
        .flatten()
    {
        let name = entry.file_name().to_string_lossy().to_string();
        let Some(n) = name
            .strip_suffix(".root.json")
            .and_then(|v| v.parse::<u64>().ok())
        else {
            continue;
        };
        if best.as_ref().is_none_or(|(b, _)| n > *b) {
            best = Some((n, entry.path()));
        }
    }
    best.map(|(_, p)| p)
        .ok_or_else(|| "no root metadata; run init-repo".into())
}

/// Validate, then add the release's targets and sign new metadata.
pub async fn publish(keys: &Path, repo: &Path, r: &Release) -> Result<(), String> {
    let d = &r.descriptor;
    let (platform, arch) = (d.platform.as_str(), d.architecture.as_str());
    d.check(&d.channel, platform, arch)
        .map_err(|e| format!("descriptor refused: {e:?}"))?;
    let notes: Notes =
        fsx::read_json(&r.notes, crate::limits::NOTES_BYTES).map_err(|e| format!("notes: {e}"))?;
    let rules_changed = r
        .previous_contract
        .as_ref()
        .is_some_and(|c| *c != d.compatibility.match_contract);
    notes.validate(&d.build_id, rules_changed)?;
    let metadata = repo.join("metadata");
    let root_path = newest_root(&metadata)?;
    let base = |dir: &str| {
        Url::from_directory_path(repo.join(dir)).map_err(|_| "repository path".to_string())
    };
    let editor = match RepositoryLoader::new(
        &std::fs::read(&root_path).map_err(|e| e.to_string())?,
        base("metadata")?,
        base("targets")?,
    )
    .load()
    .await
    {
        Ok(existing) => RepositoryEditor::from_repo(&root_path, existing).await,
        Err(_) => RepositoryEditor::new(&root_path).await,
    };
    let mut editor = editor.map_err(|e| e.to_string())?;
    let staging = tempfile::tempdir().map_err(|e| e.to_string())?;
    let pointer = ChannelPointer {
        schema: crate::descriptor::SCHEMA,
        channel: d.channel.clone(),
        build_id: d.build_id.clone(),
        release_sequence: d.release_sequence,
        descriptor: format!("release-{}.json", d.build_id),
    };
    let files = [
        (d.package_target.clone(), r.package.clone()),
        (d.notes_target.clone(), r.notes.clone()),
        (
            pointer.descriptor.clone(),
            write_doc(staging.path(), &pointer.descriptor, d)?,
        ),
        (
            crate::descriptor::pointer_target(&d.channel, platform, arch),
            write_doc(staging.path(), "pointer.json", &pointer)?,
        ),
    ];
    for (name, path) in &files {
        let (_, target) = RepositoryEditor::build_target(path)
            .await
            .map_err(|e| e.to_string())?;
        editor
            .add_target(name.as_str(), target)
            .map_err(|e| e.to_string())?;
    }
    let next = |v: u64| NonZeroU64::new(v + 1).unwrap_or(NonZeroU64::MIN);
    let version = next(current_version(&metadata));
    let when = expires(r.metadata_days)?;
    editor
        .targets_version(version)
        .and_then(|e| e.targets_expires(when))
        .map_err(|e| e.to_string())?;
    editor
        .snapshot_version(version)
        .snapshot_expires(when)
        .timestamp_version(version)
        .timestamp_expires(when);
    let role_keys: Vec<Box<dyn KeySource>> = ROLES.iter().map(|r| source(keys, r)).collect();
    let signed = editor.sign(&role_keys).await.map_err(|e| e.to_string())?;
    let targets_dir = repo.join("targets");
    for (name, path) in &files {
        let target = TargetName::new(name.as_str()).map_err(|e| e.to_string())?;
        signed
            .copy_target(path, &targets_dir, PathExists::Skip, Some(&target))
            .await
            .map_err(|e| e.to_string())?;
    }
    signed.write(&metadata).await.map_err(|e| e.to_string())
}

fn write_doc<T: serde::Serialize>(dir: &Path, name: &str, value: &T) -> Result<PathBuf, String> {
    let path = dir.join(name);
    fsx::write_json(&path, value).map_err(|e| e.to_string())?;
    Ok(path)
}

/// The highest published timestamp version (0 before the first release).
fn current_version(metadata: &Path) -> u64 {
    let Ok(raw) = std::fs::read(metadata.join("timestamp.json")) else {
        return 0;
    };
    serde_json::from_slice::<serde_json::Value>(&raw)
        .ok()
        .and_then(|v| v.get("signed")?.get("version")?.as_u64())
        .unwrap_or(0)
}
