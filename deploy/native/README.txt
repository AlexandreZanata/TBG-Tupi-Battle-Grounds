TBG (Tupi Battle Grounds) — 0.1.1-rc.1 consolidated Linux prerelease candidate

This candidate uses protocol 30 and server schema 10. The previously published
production server uses protocol 29 and is NOT compatible with this update.
Practice and the training laboratory work offline. Online acceptance uses a
separate compatible test server; this release does not deploy a new backend.
Windows packaging and exact packaged Windows/Linux visual acceptance remain
pending. Existing native.2/native.3/p68.1 downloads and installations are preserved.

Linux: extract the complete tar.gz and run ./tbg-client from any working
directory. Keep assets, licenses and metadata beside it. ./install.sh adds the
TBG launcher for the current user; administrator/Rust access is unnecessary.
Do not replace an existing installation until you choose to adopt this beta.
If FUSE is unavailable, use --appimage-extract-and-run or the tar.gz package.

Runtime: x86_64 Linux, Vulkan-capable graphics, X11/Wayland, libasound2,
libxkbcommon0 and platform X11/Wayland libraries. AppImage bundles application
libraries; your GPU driver and per-user Secret Service remain on the host.
The game does not install a keyring service, updater service or system package.

Accounts: Access credentials stay in memory. Optional "Keep me signed in"
uses the real current-user Secret Service for rotating refresh credentials.
If the vault is locked or unavailable, a visible temporary session is used;
no new key/token is saved to ordinary preferences. AccountKeys still support
manual recovery. A saved legacy key is removed only after confirmed secure
migration AND explicit recovery-backup acknowledgment. Keep that backup safe.
Signing out offline stays pending until server revocation can be confirmed.
Natural Access expiry does not end an admitted match; security revocation does
invalidate its lease. Passive restoration never takes over another controller.
A short-lived match-resume record (reconnect handle, server slot and account
binding) is kept in the same Secret Service by the same coordinator so a
restarted game can return to its match; it is never written to preferences.

F11 or Alt+Enter: fullscreen. Settings: audio, language, VSync and shadows.
--server ws://127.0.0.1:8080/ws selects a compatible local test backend.
Public custom servers must use WSS/HTTPS. --room ABCDE, --challenge TOKEN,
--watch MATCH_ID and --lang pt-BR remain available. Never put account keys or
Access/Refresh credentials in URLs or launch arguments.

Native preferences/logs: $XDG_DATA_HOME/tupi-battle-grounds or
~/.local/share/tupi-battle-grounds. Credentials live separately in Secret
Service. TBG_DATA_DIR isolates nonsecret preferences during testing, but does
not create a separate OS vault or bypass the per-origin process lock.
--version reports package, protocol and source before graphics startup.
Missing assets fail with a useful error; release builds never borrow assets
from a source checkout. file-manifest.json and build-info.json identify inputs.
Verify the .sha256 file before installing. No production deployment, installer
execution or existing-user credential conversion is performed by downloading.
