//! Managed per-user install layout (P66 §5.1): a stable launcher, immutable
//! release directories keyed by build id, the `current`/`previous`
//! selection, staging, the transaction journal, notes receipts and the
//! trusted TUF metadata. Preferences, keys and logs stay in the game's own
//! data directory, never inside a release.

use std::path::{Path, PathBuf};

/// One managed installation root.
#[derive(Clone, Debug)]
pub struct Layout {
    pub root: PathBuf,
}

/// Build ids name directories: `[a-z0-9._-]`, 1–80 characters, never
/// `.`/`..`, so a signed descriptor can never name a path outside `releases`.
pub fn valid_build_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 80
        && id != "."
        && id != ".."
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"._-".contains(&b))
}

impl Layout {
    pub fn new(root: impl Into<PathBuf>) -> Layout {
        Layout { root: root.into() }
    }

    /// `$TBG_UPDATE_ROOT`, else `~/.local/opt/tupi-battle-grounds/managed`
    /// beside the legacy versioned installs (never inside them), or
    /// `%LOCALAPPDATA%\TupiBattleGrounds\managed` on Windows.
    pub fn default_root() -> Option<PathBuf> {
        if let Some(root) = std::env::var_os("TBG_UPDATE_ROOT") {
            return Some(root.into());
        }
        #[cfg(windows)]
        {
            let base = std::env::var_os("LOCALAPPDATA")?;
            Some(
                PathBuf::from(base)
                    .join("TupiBattleGrounds")
                    .join("managed"),
            )
        }
        #[cfg(not(windows))]
        {
            let home = PathBuf::from(std::env::var_os("HOME")?);
            Some(home.join(".local/opt/tupi-battle-grounds/managed"))
        }
    }

    pub fn releases(&self) -> PathBuf {
        self.root.join("releases")
    }

    pub fn release(&self, build: &str) -> PathBuf {
        self.releases().join(build)
    }

    pub fn staging(&self) -> PathBuf {
        self.root.join("staging")
    }

    pub fn state(&self) -> PathBuf {
        self.root.join("state")
    }

    pub fn journal(&self) -> PathBuf {
        self.state().join("journal.json")
    }

    pub fn current(&self) -> PathBuf {
        self.state().join("current")
    }

    pub fn previous(&self) -> PathBuf {
        self.state().join("previous")
    }

    pub fn receipts(&self) -> PathBuf {
        self.state().join("receipts")
    }

    pub fn trust(&self) -> PathBuf {
        self.root.join("trust")
    }

    /// The bundled trusted root (installed by the bootstrap).
    pub fn trusted_root(&self) -> PathBuf {
        self.trust().join("root.json")
    }

    /// tough's datastore: last trusted metadata (anti-rollback/freeze).
    pub fn datastore(&self) -> PathBuf {
        self.trust().join("datastore")
    }

    /// Repository location and channel (`trust/source.json`).
    pub fn source(&self) -> PathBuf {
        self.trust().join("source.json")
    }

    /// Per-launch health files and the single-instance lock.
    pub fn run(&self) -> PathBuf {
        self.root.join("run")
    }

    pub fn lock(&self) -> PathBuf {
        self.run().join("launcher.lock")
    }

    pub fn launcher_dir(&self) -> PathBuf {
        self.root.join("launcher")
    }

    /// The game executable inside a release directory.
    pub fn game_binary(&self, build: &str) -> PathBuf {
        let name = if cfg!(windows) {
            "tbg-client.exe"
        } else {
            "tbg-client"
        };
        self.release(build).join(name)
    }

    /// Read a selection file (`current`/`previous`): one valid build id.
    pub fn read_selection(path: &Path) -> Option<String> {
        let raw = crate::fsx::read_bounded(path, 256).ok()?;
        let id = String::from_utf8(raw).ok()?.trim().to_string();
        valid_build_id(&id).then_some(id)
    }

    pub fn current_build(&self) -> Option<String> {
        Layout::read_selection(&self.current())
    }

    pub fn previous_build(&self) -> Option<String> {
        Layout::read_selection(&self.previous())
    }
}
