#!/usr/bin/env bash
# Bundle the already-built native Linux package; graphics drivers stay on the host.
set -euo pipefail
project_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$project_dir"
linuxdeploy="$(realpath -- "${1:?Pass the official linuxdeploy AppImage}")"
version="$(python3 -c 'import tomllib; print(tomllib.load(open("crates/tbg-client/Cargo.toml", "rb"))["package"]["version"])')"
package="target/native/tupi-battle-grounds-$version-linux-x86_64"
appdir="$project_dir/target/native/AppDir"
mkdir -p "$appdir/usr/bin" "$appdir/usr/share/applications" "$appdir/usr/share/icons/hicolor/scalable/apps"
cp "$package/tbg-client" "$appdir/usr/bin/tbg-client"
cp -a "$package/assets" "$appdir/usr/bin/"
cp "$package/THIRD_PARTY_NOTICES.md" "$package/README.txt" "$package/build-info.json" "$appdir/usr/bin/"
for notice in "$package"/LICENSE*; do
    if [[ -f "$notice" ]]; then cp "$notice" "$appdir/usr/bin/"; fi
done
cp "$package/assets/lobby/crest-v2.svg" "$appdir/usr/share/icons/hicolor/scalable/apps/tupi-battle-grounds.svg"
cat > "$appdir/usr/share/applications/tupi-battle-grounds.desktop" <<'DESKTOP'
[Desktop Entry]
Type=Application
Name=Tupi Battle Grounds
Exec=tbg-client
Icon=tupi-battle-grounds
Terminal=false
Categories=Game;StrategyGame;
DESKTOP
export ARCH=x86_64
export OUTPUT="$project_dir/target/native/tupi-battle-grounds-$version-linux-x86_64.AppImage"
"$linuxdeploy" --appimage-extract-and-run --appdir "$appdir" --executable "$appdir/usr/bin/tbg-client" --desktop-file "$appdir/usr/share/applications/tupi-battle-grounds.desktop" --icon-file "$appdir/usr/share/icons/hicolor/scalable/apps/tupi-battle-grounds.svg" --output appimage
(cd "$(dirname -- "$OUTPUT")" && sha256sum "$(basename -- "$OUTPUT")" > "$(basename -- "$OUTPUT").sha256")
