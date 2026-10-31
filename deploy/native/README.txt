TBG (Tupi Battle Grounds) — native preview

Linux: extract the tar.gz, then run ./tbg-client from its folder or any other
working directory. Run ./install.sh for the TBG application-menu launcher.
The installed command is ~/.local/bin/tbg. Updates keep preferences, account
keys and older builds. Each source commit has its own installation folder.
No Rust installation or administrator access is required for the game.
The optional installer uses Python 3 (included on common Linux desktops).

Runtime: x86_64 Linux with Vulkan-capable graphics, X11 or Wayland, libasound2,
libxkbcommon0 and the platform's X11/Wayland client libraries. The AppImage
bundles application libraries; graphics drivers still come from your system.
If FUSE is unavailable, use --appimage-extract-and-run or the tar.gz package.

Windows: extract the complete ZIP before running tbg-client.exe. Keep the
assets folder beside the executable. This preview is unsigned.

Play: Practice works offline. Online modes connect to the same authoritative
server as the browser game. An account key works on every platform.
F11 or Alt+Enter: fullscreen. Settings: sound, volume, language, VSync/shadows.
--server ws://127.0.0.1:8080/ws selects a development server.
--room ABCDE, --challenge TOKEN, --watch MATCH_ID and --lang pt-BR are supported.
TBG_SERVER and other TBG_* environment overrides remain available.

Native data and diagnostic logs: Linux $XDG_DATA_HOME/tupi-battle-grounds
(or ~/.local/share/tupi-battle-grounds); Windows the OS user-data directory.
Account keys are stored as plain text in private local preferences, like the
web client's localStorage. Linux files are mode 0600. Keep a separate copy of
your key; uninstalling the application does not remove your account data.
TBG_DATA_DIR overrides this location for isolated testing.

Updates: use Settings > Downloads and updates. A protocol mismatch requires
a newer executable. Browser, server and native releases should use one source
revision. build-info.json identifies this package's source and executable.
Verify the archive's .sha256 file before installing.
