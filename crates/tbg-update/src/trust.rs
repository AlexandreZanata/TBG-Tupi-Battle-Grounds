//! The TUF client (P66 §4, D-91, U02): the bundled root, the persistent
//! datastore (anti-rollback/freeze, root rotation) and bounded target
//! reads all go through tough. Nothing here trusts `/api/version`, an
//! adjacent checksum or a field inside a downloaded document.

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};

use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use tough::{Limits, Repository, RepositoryLoader, TargetName};
use url::Url;

use crate::descriptor::{ChannelPointer, ReleaseDescriptor};
use crate::fsx;
use crate::layout::Layout;
use crate::limits::{DESCRIPTOR_BYTES, METADATA_BYTES};
use crate::transport::HttpsTransport;

/// Where the repository lives and which channel this install follows.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub metadata_url: String,
    pub targets_url: String,
    pub channel: String,
}

pub fn read_source(layout: &Layout) -> Result<Source, String> {
    fsx::read_json(&layout.source(), 16 * 1024)
        .map_err(|e| format!("update source unreadable: {e}"))
}

/// Load and verify the repository metadata.
pub async fn open(layout: &Layout) -> Result<(Repository, Source), String> {
    let source = read_source(layout)?;
    let root = fsx::read_bounded(&layout.trusted_root(), METADATA_BYTES)
        .map_err(|e| format!("trusted root unreadable: {e}"))?;
    let datastore = layout.datastore();
    fsx::private_dir(&datastore).map_err(|e| e.to_string())?;
    let metadata = Url::parse(&source.metadata_url).map_err(|e| e.to_string())?;
    let targets = Url::parse(&source.targets_url).map_err(|e| e.to_string())?;
    let limits = Limits {
        max_root_size: METADATA_BYTES,
        max_targets_size: METADATA_BYTES,
        max_timestamp_size: DESCRIPTOR_BYTES,
        max_snapshot_size: METADATA_BYTES,
        max_root_updates: 64,
    };
    let repo = RepositoryLoader::new(&root, metadata, targets)
        .transport(HttpsTransport::default())
        .datastore(datastore)
        .limits(limits)
        .load()
        .await
        .map_err(|e| format!("repository verification failed: {e}"))?;
    Ok((repo, source))
}

/// The signed length of a target, if listed.
pub fn target_length(repo: &Repository, name: &str) -> Option<u64> {
    let name = TargetName::new(name).ok()?;
    repo.all_targets()
        .find(|(n, _)| **n == name)
        .map(|(_, t)| t.length)
}

/// Read a small target fully (verified by tough before it is returned).
pub async fn read_small(repo: &Repository, name: &str, limit: u64) -> Result<Vec<u8>, String> {
    match target_length(repo, name) {
        Some(len) if len <= limit => {}
        Some(_) => return Err(format!("{name} exceeds its size limit")),
        None => return Err(format!("{name} is not a signed target")),
    }
    let target = TargetName::new(name).map_err(|e| e.to_string())?;
    let mut stream = repo
        .read_target(&target)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("{name} is not a signed target"))?;
    let mut out = Vec::new();
    while let Some(chunk) = tokio::time::timeout(
        std::time::Duration::from_secs(crate::limits::DOWNLOAD_IDLE_SECS),
        stream.next(),
    )
    .await
    .map_err(|_| "download inactivity deadline exceeded".to_string())?
    {
        out.extend_from_slice(&chunk.map_err(|e| format!("{name} failed verification: {e}"))?);
    }
    Ok(out)
}

/// The channel's newest release for this install.
pub async fn latest(
    repo: &Repository,
    source: &Source,
) -> Result<(ChannelPointer, ReleaseDescriptor), String> {
    let (platform, arch) = crate::descriptor::this_platform();
    let pointer_name = crate::descriptor::pointer_target(&source.channel, platform, arch);
    let raw = read_small(repo, &pointer_name, DESCRIPTOR_BYTES).await?;
    let pointer: ChannelPointer =
        serde_json::from_slice(&raw).map_err(|e| format!("channel pointer: {e}"))?;
    pointer
        .check(&source.channel)
        .map_err(|e| format!("channel pointer refused: {e:?}"))?;
    let raw = read_small(repo, &pointer.descriptor, DESCRIPTOR_BYTES).await?;
    let descriptor: ReleaseDescriptor =
        serde_json::from_slice(&raw).map_err(|e| format!("descriptor: {e}"))?;
    descriptor
        .check(&source.channel, platform, arch)
        .map_err(|e| format!("release refused: {e:?}"))?;
    if descriptor.build_id != pointer.build_id
        || descriptor.release_sequence != pointer.release_sequence
    {
        return Err("channel pointer and descriptor disagree".into());
    }
    Ok((pointer, descriptor))
}

/// Stream a large target to `dest`, reporting progress; any verification
/// error, cancellation or size overrun deletes the partial file.
pub async fn download(
    repo: &Repository,
    name: &str,
    dest: &Path,
    limit: u64,
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(u64, u64),
) -> Result<u64, String> {
    let result = download_inner(repo, name, dest, limit, cancel, progress).await;
    if result.is_err() {
        let _ = std::fs::remove_file(dest);
    }
    result
}

async fn download_inner(
    repo: &Repository,
    name: &str,
    dest: &Path,
    limit: u64,
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(u64, u64),
) -> Result<u64, String> {
    let total =
        target_length(repo, name).ok_or_else(|| format!("{name} is not a signed target"))?;
    if total > limit {
        return Err(format!("{name} exceeds its size limit"));
    }
    let target = TargetName::new(name).map_err(|e| e.to_string())?;
    let mut stream = repo
        .read_target(&target)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("{name} is not a signed target"))?;
    let mut file = tokio::fs::File::create(dest)
        .await
        .map_err(|e| e.to_string())?;
    let mut done = 0u64;
    while let Some(chunk) = tokio::time::timeout(
        std::time::Duration::from_secs(crate::limits::DOWNLOAD_IDLE_SECS),
        stream.next(),
    )
    .await
    .map_err(|_| "download inactivity deadline exceeded".to_string())?
    {
        if cancel.load(Relaxed) {
            return Err("cancelled".into());
        }
        let chunk = chunk.map_err(|e| format!("{name} failed verification: {e}"))?;
        done += chunk.len() as u64;
        file.write_all(&chunk).await.map_err(|e| e.to_string())?;
        progress(done, total);
    }
    file.sync_all().await.map_err(|e| e.to_string())?;
    Ok(done)
}
