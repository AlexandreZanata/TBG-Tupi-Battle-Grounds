#!/usr/bin/env bash
# Build and install TBG Arena 5v5 (client + local room server) for this user.
# Usage: moba/scripts/install-linux-moba.sh [--release]
# Installs to ~/.local/opt/tbg-moba/<version>-<commit>/, links ~/.local/bin/tbg-moba
# and ~/.local/bin/tbg-moba-server, and adds an application-menu entry.
# Compilation shares the host validation lock (docs/VALIDATION.md §2).
set -euo pipefail
moba_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
repo_dir="$(cd -- "$moba_dir/.." && pwd)"
profile=dev
out=debug
if [[ "${1:-}" == "--release" ]]; then
    profile=release
    out=release
fi
export CARGO_BUILD_JOBS=2
CARGO_TARGET_DIR="$(realpath -m -- "${CARGO_TARGET_DIR:-$moba_dir/target}")"
export CARGO_TARGET_DIR
cd "$moba_dir"
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" \
    cargo build --locked --profile "$profile" -p tbg-moba-client -p tbg-moba-server || {
    status=$?
    [[ $status -eq 75 ]] && printf 'Validation slot busy; try again when the other build ends.\n' >&2
    exit "$status"
}
version="$(sed -n 's/^version = "\(.*\)"/\1/p' client-moba/Cargo.toml | head -1)"
commit="$(git -C "$repo_dir" rev-parse --short=12 HEAD 2>/dev/null || echo local)"
if ! git -C "$repo_dir" diff --quiet HEAD -- moba 2>/dev/null; then
    commit="$commit-dirty"
fi
base="$HOME/.local"
install="$base/opt/tbg-moba/$version-$commit"
mkdir -p "$install" "$base/bin"
for bin in tbg-moba tbg-moba-server; do
    install -m 0755 "$CARGO_TARGET_DIR/$out/$bin" "$install/$bin"
    ln -sfn "$install/$bin" "$base/bin/$bin"
done
# The shared 1v1 card frames, portraits, crests and icons (D-112).
assets="$repo_dir/crates/tbg-client/assets"
rm -rf "$install/assets"
mkdir -p "$install/assets/lobby"
cp -R "$assets/battle" "$install/assets/battle"
cp -R "$assets/lobby/native-icons" "$install/assets/lobby/native-icons"
icon="$assets/branding/tbg-icon-512.png"
[[ -f "$icon" ]] && install -m 0644 "$icon" "$install/icon.png"
applications="${XDG_DATA_HOME:-$base/share}/applications"
mkdir -p "$applications"
cat > "$applications/tbg-arena-5v5.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=TBG Arena 5v5
GenericName=Tupi Battle Grounds MOBA
Comment=5v5 lanes, jungle and troops — practice against 9 bots
Exec="$install/tbg-moba"
Icon=$install/icon.png
Terminal=false
Categories=Game;StrategyGame;
StartupWMClass=tbg-moba
DESKTOP
if command -v update-desktop-database >/dev/null; then
    update-desktop-database "$applications" || true
fi
printf 'Installed TBG Arena 5v5 %s (%s profile) at %s\n' "$version-$commit" "$profile" "$install"
printf 'Play: open "TBG Arena 5v5" from the application menu, or run tbg-moba\n'
printf 'Local room server: tbg-moba-server, then tbg-moba --online\n'
