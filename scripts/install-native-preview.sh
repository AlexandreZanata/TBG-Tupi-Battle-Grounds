#!/usr/bin/env bash
# Build and install the committed Linux preview for short UI feedback cycles.
set -euo pipefail
project_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$project_dir"
git diff --quiet HEAD -- Cargo.toml Cargo.lock crates scripts deploy/native LICENSE THIRD_PARTY_NOTICES.md || {
    printf 'Commit the requested source changes before installing a preview.\n' >&2
    exit 1
}
export TBG_SOURCE_COMMIT="$(git rev-parse HEAD)"
export CARGO_BUILD_JOBS=2
export CARGO_TARGET_DIR="$(realpath -m -- "${CARGO_TARGET_DIR:-$project_dir/target}")"
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" bash -eu -c '
    cargo build --locked -p tbg-client --bin tbg-client
    package_dir="$(python3 scripts/package-native.py --platform linux \
        --binary "$CARGO_TARGET_DIR/debug/tbg-client" \
        --output "$CARGO_TARGET_DIR/native-preview" --no-archive)"
    bash "$package_dir/install.sh"
    if command -v update-desktop-database >/dev/null; then
        update-desktop-database "${XDG_DATA_HOME:-$HOME/.local/share}/applications"
    fi
'
