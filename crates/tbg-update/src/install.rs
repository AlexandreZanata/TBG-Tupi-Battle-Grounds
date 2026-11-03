//! Trusted bootstrap and legacy adoption (P66 §5.2, U07): the one manual
//! step. It installs the launcher and the trusted root out of band, then
//! downloads the channel's release through TUF exactly like an update.
//! Existing versioned installs and user data are never deleted; the menu
//! entry and the `tbg` command are pointed at the stable launcher (the old
//! ones are kept as `.pre-managed` backups), so later updates need no
//! uninstall, reinstall or sudo.

use std::path::{Path, PathBuf};

use crate::layout::Layout;
use crate::trust::Source;
use crate::{activate, fsx, helper, journal, trust};

pub struct Options {
    pub root_json: PathBuf,
    pub source: Source,
    /// Create the desktop entry and the `tbg` command (Linux).
    pub desktop: bool,
}

/// Install or adopt, returning the installed build id.
pub fn install(layout: &Layout, opts: &Options) -> Result<String, String> {
    for dir in [
        layout.root.clone(),
        layout.state(),
        layout.trust(),
        layout.run(),
        layout.launcher_dir(),
    ] {
        fsx::private_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    let root = fsx::read_bounded(&opts.root_json, crate::limits::METADATA_BYTES)
        .map_err(|e| e.to_string())?;
    fsx::write_atomic(&layout.trusted_root(), &root).map_err(|e| e.to_string())?;
    fsx::write_json(&layout.source(), &opts.source).map_err(|e| e.to_string())?;
    let launcher = install_launcher(layout)?;
    let build = fetch_latest(layout)?;
    let from = layout.current_build().unwrap_or_default();
    activate::switch(layout, &from, &build).map_err(|e| e.to_string())?;
    journal::store(layout, None).map_err(|e| e.to_string())?;
    let icon = layout
        .release(&build)
        .join("assets/branding/tbg-icon-512.png");
    if icon.is_file() {
        std::fs::copy(icon, layout.launcher_dir().join("tbg-icon-512.png"))
            .map_err(|error| error.to_string())?;
    }
    if opts.desktop {
        desktop_entry(&launcher)?;
        command_link(&launcher)?;
    }
    Ok(build)
}

/// Copy this executable to `launcher/` (the stable path shortcuts use).
fn install_launcher(layout: &Layout) -> Result<PathBuf, String> {
    let me = std::env::current_exe().map_err(|e| e.to_string())?;
    let name = if cfg!(windows) {
        "tbg-launcher.exe"
    } else {
        "tbg-launcher"
    };
    let target = layout.launcher_dir().join(name);
    if me != target {
        let temp = layout.launcher_dir().join(format!(".{name}.new"));
        std::fs::copy(&me, &temp).map_err(|e| e.to_string())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&temp, std::fs::Permissions::from_mode(0o755))
                .map_err(|e| e.to_string())?;
        }
        std::fs::rename(&temp, &target).map_err(|e| e.to_string())?;
    }
    Ok(target)
}

/// Stage the channel's release through TUF (same path as an update).
fn fetch_latest(layout: &Layout) -> Result<String, String> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;
    let descriptor = rt.block_on(async {
        let (repo, source) = trust::open(layout).await?;
        let (_, d) = trust::latest(&repo, &source).await?;
        fsx::private_dir(&layout.staging()).map_err(|e| e.to_string())?;
        let archive = layout.staging().join(format!("{}.tar.gz.part", d.build_id));
        let cancel = std::sync::atomic::AtomicBool::new(false);
        let mut quiet = |_, _| {};
        trust::download(
            &repo,
            &d.package_target,
            &archive,
            crate::limits::PACKAGE_BYTES,
            &cancel,
            &mut quiet,
        )
        .await?;
        Ok::<_, String>((d, archive))
    })?;
    let (d, archive) = descriptor;
    if helper::install_staged(layout, &d.build_id, &archive, &d) != 0 {
        return Err("the downloaded release did not verify".into());
    }
    Ok(d.build_id)
}

fn home() -> Result<PathBuf, String> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| "HOME is not set".into())
}

/// Keep a one-time backup of a file this installer replaces.
fn backup_once(path: &Path) -> Result<(), String> {
    let backup = path.with_extension(format!(
        "{}pre-managed",
        path.extension()
            .map_or(String::new(), |e| format!("{}.", e.to_string_lossy()))
    ));
    if path.exists() && !backup.exists() {
        std::fs::copy(path, &backup).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Desktop Exec points at the stable launcher (quoted for spaces).
fn desktop_entry(launcher: &Path) -> Result<(), String> {
    if cfg!(windows) {
        return Ok(());
    }
    let data = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or(home()?.join(".local/share"));
    let dir = data.join("applications");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join("tupi-battle-grounds.desktop");
    backup_once(&path)?;
    let exec = launcher
        .to_string_lossy()
        .replace('\\', "\\\\")
        .replace('"', "\\\"");
    let icon = launcher
        .parent()
        .and_then(|p| p.parent())
        .map(|r| r.join("launcher/tbg-icon-512.png"))
        .filter(|p| p.is_file())
        .map(|p| format!("Icon={}\n", p.display()))
        .unwrap_or_default();
    let entry = format!(
        "[Desktop Entry]\nType=Application\nName=TBG\nComment=Tupi Battle Grounds\nExec=\"{exec}\"\n{icon}Terminal=false\nCategories=Game;StrategyGame;\nStartupWMClass=tupi-battle-grounds\n"
    );
    fsx::write_atomic(&path, entry.as_bytes()).map_err(|e| e.to_string())
}

/// `~/.local/bin/tbg` becomes a link to the launcher. A link to a legacy
/// versioned binary is replaced (its target is kept on disk); a regular
/// file that is not ours is left alone.
fn command_link(launcher: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        let bin = home()?.join(".local/bin");
        std::fs::create_dir_all(&bin).map_err(|e| e.to_string())?;
        for name in ["tbg", "tupi-battle-grounds"] {
            let link = bin.join(name);
            match std::fs::symlink_metadata(&link) {
                Ok(m) if !m.file_type().is_symlink() => {
                    eprintln!(
                        "tbg-launcher: {} is not a link; left unchanged",
                        link.display()
                    );
                    continue;
                }
                Ok(_) => {}
                Err(_) => {}
            }
            let temporary = link.with_extension("managed-new");
            if temporary.is_symlink() {
                std::fs::remove_file(&temporary).map_err(|error| error.to_string())?;
            }
            std::os::unix::fs::symlink(launcher, &temporary).map_err(|e| e.to_string())?;
            std::fs::rename(temporary, &link).map_err(|error| error.to_string())?;
        }
    }
    #[cfg(not(unix))]
    let _ = launcher;
    Ok(())
}
