TBG (Tupi Battle Grounds) — @VERSION@ @PLATFORM@ beta candidate

Package source: @SOURCE@
MatchContract: @CONTRACT@
The executable's --build-info reports its exact version, source and protocol.
Online play requires this exact MatchContract on the server. The default public
endpoint is wss://api.tupibg.com/ws. Practice and training also work offline.
This download does not deploy a server, configure email recovery or enable TUF
updates. Exact Linux/Windows visual acceptance remains pending; see release notes.

Linux: extract the complete tar.gz and run ./tbg-client from any working
directory. Keep assets, licenses and metadata beside it. ./install.sh adds the
TBG launcher for the current user; administrator/Rust access is unnecessary.
AppImage: make it executable and launch it. If FUSE is unavailable, use
--appimage-extract-and-run or the tar.gz package.

Windows: extract the complete ZIP and launch tbg-client.exe, or use the per-user
installer. Windows 11 x64 / DirectX 12 is the current candidate target. No
administrator, developer SDK or separate Visual C++ runtime installer is needed.
Close TBG normally before reinstalling; the installer does not kill a match.
The candidate has no Authenticode publisher signature. GitHub provenance and
SHA-256 verify the pipeline and files, not publisher trust or SmartScreen reputation.

Linux runtime: x86_64, glibc 2.35+, Vulkan-capable graphics, X11/Wayland,
libasound2, libxkbcommon0 and platform X11/Wayland libraries. AppImage bundles
application libraries; GPU drivers and the per-user vault remain on the host.
The game does not install a keyring service, updater service or system package.

Accounts: Access credentials stay in memory. Optional "Keep me signed in"
uses the real current-user Secret Service (Linux) or Credential Manager (Windows)
for rotating refresh credentials. If the vault is locked or unavailable, a visible
temporary session is used; no new key/token is saved to ordinary preferences.
AccountKeys still support manual recovery. A saved legacy key is removed only
after confirmed secure migration AND explicit recovery-backup acknowledgment.
Signing out offline stays pending until server revocation can be confirmed.
Natural Access expiry does not end an admitted match; security revocation does
invalidate its lease. Passive restoration never takes over another controller.
A short-lived match-resume record is kept in the same OS vault by the same
coordinator; it is never written to preferences.

F11 or Alt+Enter: fullscreen. Settings: audio, language, VSync and shadows.
--server ws://127.0.0.1:8080/ws selects a compatible local test backend.
Public custom servers must use WSS/HTTPS. --room ABCDE, --challenge TOKEN,
--watch MATCH_ID and --lang pt-BR remain available. Never put account keys or
Access/Refresh credentials in URLs or launch arguments.

Preferences/logs: Linux uses $XDG_DATA_HOME/tupi-battle-grounds or
~/.local/share/tupi-battle-grounds; Windows uses the current user's local app data.
Credentials live separately in the OS vault. TBG_DATA_DIR isolates nonsecret
preferences during testing, but does not create a separate vault or bypass the
per-origin process lock. Uninstall keeps user data and other versions intact.
Missing assets fail before gameplay; release builds never borrow assets from a
source checkout. file-manifest.json and build-info.json identify package inputs.
Verify the .sha256 file before installing. Preserve your recovery backup.
