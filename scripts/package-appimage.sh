#!/usr/bin/env bash
# Bundle the already-built native Linux package; graphics drivers stay on the host.
set -euo pipefail
project_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$project_dir"
linuxdeploy="$(realpath -- "${1:?Pass the official linuxdeploy AppImage}")"
version="$(python3 -c 'import tomllib; print(tomllib.load(open("crates/tbg-client/Cargo.toml", "rb"))["package"]["version"])')"
package="target/native/tupi-battle-grounds-$version-linux-x86_64"
appdir="$project_dir/target/native/AppDir"
mkdir -p "$appdir/usr/bin" "$appdir/usr/share/applications" "$appdir/usr/share/icons/hicolor/512x512/apps"
cp "$package/tbg-client" "$appdir/usr/bin/tbg-client"
cp -a "$package/assets" "$appdir/usr/bin/"
cp "$package/THIRD_PARTY_NOTICES.md" "$package/README.txt" "$package/build-info.json" "$appdir/usr/bin/"
for notice in "$package"/LICENSE*; do
    if [[ -f "$notice" ]]; then cp "$notice" "$appdir/usr/bin/"; fi
done
cp "$package/assets/branding/tbg-icon-512.png" "$appdir/usr/share/icons/hicolor/512x512/apps/tupi-battle-grounds.png"
cat > "$appdir/usr/share/applications/tupi-battle-grounds.desktop" <<'DESKTOP'
[Desktop Entry]
Type=Application
Name=TBG
Exec=tbg-client
Icon=tupi-battle-grounds
Terminal=false
Categories=Game;StrategyGame;
DESKTOP
export ARCH=x86_64
export OUTPUT="$project_dir/target/native/tupi-battle-grounds-$version-linux-x86_64.AppImage"
"$linuxdeploy" --appimage-extract-and-run --appdir "$appdir" --executable "$appdir/usr/bin/tbg-client" --desktop-file "$appdir/usr/share/applications/tupi-battle-grounds.desktop" --icon-file "$appdir/usr/share/icons/hicolor/512x512/apps/tupi-battle-grounds.png"
# linuxdeploy patches ELF library paths; identify the bundled executable after that mutation.
python3 - "$appdir/usr/bin" <<'PY'
import hashlib, json, sys
from pathlib import Path
folder = Path(sys.argv[1])
path = folder / "build-info.json"
info = json.loads(path.read_text())
info["unbundled_binary_sha256"] = info["binary_sha256"]
info["binary_sha256"] = hashlib.sha256((folder / "tbg-client").read_bytes()).hexdigest()
info["format"] = "appimage"
path.write_text(json.dumps(info, indent=2) + "\n")
PY
tool_dir="$project_dir/target/native/linuxdeploy-tool"
mkdir -p "$tool_dir"
(cd "$tool_dir" && "$linuxdeploy" --appimage-extract > /dev/null)
"$tool_dir/squashfs-root/plugins/linuxdeploy-plugin-appimage/appimagetool-prefix/AppRun" --mksquashfs-opt -processors --mksquashfs-opt 2 "$appdir" "$OUTPUT"
(cd "$(dirname -- "$OUTPUT")" && sha256sum "$(basename -- "$OUTPUT")" > "$(basename -- "$OUTPUT").sha256")
