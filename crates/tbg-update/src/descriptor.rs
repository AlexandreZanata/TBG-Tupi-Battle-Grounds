//! Release identity (P66 §4, contracts §2): the channel pointer and the
//! release descriptor are TUF targets, so their trust comes from the
//! repository roles, never from a field inside them. Parsing is strict:
//! unknown fields, wrong schema or a malformed id is rejected before any
//! disk mutation.

use serde::{Deserialize, Serialize};

use crate::layout::valid_build_id;

/// `channel-<channel>-<platform>-<arch>.json`: the newest release. Target
/// names are flat so consistent-snapshot URLs (`<sha256>.<name>`) stay
/// plain immutable files on any static host or CDN.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChannelPointer {
    pub schema: u32,
    pub channel: String,
    pub build_id: String,
    pub release_sequence: u64,
    /// The descriptor target (`release-<build_id>.json`).
    pub descriptor: String,
}

/// Compatibility identities a release was built for.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Compatibility {
    /// The P66 §6.5 match contract (`w<wire>-r<ruleset>-<hash>`).
    pub match_contract: String,
    /// Presentation compatibility (assets that do not change play).
    pub presentation: String,
}

/// One release for one platform (`release-<build_id>.json`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseDescriptor {
    pub schema: u32,
    pub build_id: String,
    pub release_sequence: u64,
    pub display_version: String,
    pub channel: String,
    pub platform: String,
    pub architecture: String,
    pub install_mode: String,
    /// The package target (`package-<build_id>.tar.gz`).
    pub package_target: String,
    /// The notes target (`notes-<build_id>.json`); also inside the package.
    pub notes_target: String,
    pub compatibility: Compatibility,
    pub updater_abi: u32,
    pub minimum_bootstrap: u32,
    /// Releases below this sequence must not run (revocation floor).
    pub security_floor_sequence: u64,
    pub local_state_reader: Vec<u32>,
    pub local_state_writer: u32,
    pub minimum_glibc: Option<String>,
}

/// What this launcher understands.
pub const SCHEMA: u32 = 1;
pub const UPDATER_ABI: u32 = 1;
pub const BOOTSTRAP: u32 = 1;
/// Local state (journal/receipts) schema this launcher reads and writes.
pub const LOCAL_STATE: u32 = 1;

/// Why a descriptor cannot be installed here.
#[derive(Debug, PartialEq, Eq)]
pub enum Refusal {
    Schema,
    BadId,
    WrongPlatform,
    WrongChannel,
    NewerUpdater,
    BootstrapTooOld,
    StateUnreadable,
    BadTarget,
}

fn target_ok(name: &str, prefix: &str, build: &str, ext: &str) -> bool {
    name == format!("{prefix}{build}{ext}")
}

impl ReleaseDescriptor {
    /// Check the descriptor against this installation before staging.
    pub fn check(&self, channel: &str, platform: &str, architecture: &str) -> Result<(), Refusal> {
        if self.schema != SCHEMA {
            return Err(Refusal::Schema);
        }
        if !valid_build_id(&self.build_id) {
            return Err(Refusal::BadId);
        }
        if self.platform != platform
            || self.architecture != architecture
            || self.install_mode != "managed"
        {
            return Err(Refusal::WrongPlatform);
        }
        if self.channel != channel {
            return Err(Refusal::WrongChannel);
        }
        if self.updater_abi > UPDATER_ABI {
            return Err(Refusal::NewerUpdater);
        }
        if self.minimum_bootstrap > BOOTSTRAP {
            return Err(Refusal::BootstrapTooOld);
        }
        if !self.local_state_reader.contains(&LOCAL_STATE) {
            return Err(Refusal::StateUnreadable);
        }
        if !target_ok(&self.package_target, "package-", &self.build_id, ".tar.gz")
            || !target_ok(&self.notes_target, "notes-", &self.build_id, ".json")
        {
            return Err(Refusal::BadTarget);
        }
        Ok(())
    }
}

impl ChannelPointer {
    pub fn check(&self, channel: &str) -> Result<(), Refusal> {
        if self.schema != SCHEMA {
            return Err(Refusal::Schema);
        }
        if self.channel != channel {
            return Err(Refusal::WrongChannel);
        }
        if !valid_build_id(&self.build_id)
            || !target_ok(&self.descriptor, "release-", &self.build_id, ".json")
        {
            return Err(Refusal::BadId);
        }
        Ok(())
    }
}

/// The channel pointer target for this platform.
pub fn pointer_target(channel: &str, platform: &str, arch: &str) -> String {
    format!("channel-{channel}-{platform}-{arch}.json")
}

/// This machine's platform and architecture names.
pub fn this_platform() -> (&'static str, &'static str) {
    let platform = if cfg!(windows) { "windows" } else { "linux" };
    (platform, std::env::consts::ARCH)
}
