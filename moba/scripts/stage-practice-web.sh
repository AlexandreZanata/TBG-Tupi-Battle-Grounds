#!/usr/bin/env bash
# Stage the opt-in 1v1 and 5v5 desktop web preview under one local web root.
set -euo pipefail
root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
output="$(realpath -m -- "${1:-$root/.local/practice-5v5-web}")"
assets="$root/crates/tbg-client/assets"
# Keep Trunk's two staging directories separate. The 1v1 build replaces its dist.
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" \
    env CARGO_BUILD_JOBS=2 bash -eu -c '
        cd "$1/crates/tbg-client"
        trunk build --dist "$2" --locked
        cd "$1/moba/client-moba"
        trunk build --dist "$2/moba" --public-url /moba/ --locked
    ' _ "$root" "$output"
mkdir -p "$output/moba/assets/lobby"
cp -R "$assets/battle" "$output/moba/assets/"
cp -R "$assets/lobby/native-icons" "$output/moba/assets/lobby/"
printf 'Serve %s and open /?practice5v5=1 on that local server.\n' "$output"
