#!/usr/bin/env bash
# Build/copy to the VPS, build the server there, restart and check health.
# Reads ignored deploy/.env.deploy. --backend-only preserves the web bundle.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck disable=SC1091
source "$ROOT/deploy/.env.deploy"
: "${TBG_HOST:?}" "${TBG_SSH_USER:?}" "${TBG_DOMAIN:?}"
PORT="${TBG_SSH_PORT:-22}"
TARGET="$TBG_SSH_USER@$TBG_HOST"
SSH=(ssh -o BatchMode=yes -p "$PORT" "$TARGET")
PROVISION=0
BACKEND_ONLY=0
for argument in "$@"; do
	case "$argument" in
		--provision) PROVISION=1 ;;
		--backend-only) BACKEND_ONLY=1 ;;
		*) echo "unknown deploy option: $argument" >&2; exit 2 ;;
	esac
done

if [ "$BACKEND_ONLY" = 0 ]; then
	(cd "$ROOT/crates/tbg-client" && flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" \
		env CARGO_BUILD_JOBS=2 trunk build --release)
fi

rsync -az --delete -e "ssh -o BatchMode=yes -p $PORT" \
	--exclude target/ --exclude dist/ --exclude .git/ --exclude archive/ \
	--exclude .local/ --exclude .agents/ --exclude .codex/ --exclude .ssh/ \
	--exclude '.env*' --exclude '*.pem' "$ROOT/" "$TARGET:tbg-src/"
if [ "$BACKEND_ONLY" = 0 ]; then
	rsync -az --delete -e "ssh -o BatchMode=yes -p $PORT" \
		"$ROOT/crates/tbg-client/dist/" "$TARGET:tbg-web/"
fi

if [ "$PROVISION" = 1 ]; then
	"${SSH[@]}" "sudo bash ~/tbg-src/deploy/provision.sh '$TBG_DOMAIN' '$TBG_SSH_USER' '${TBG_APP_DOMAIN:-}'"
fi

# Compile on the destination to avoid glibc mismatches. Preserve the local
# validation slot for client work; the remote host has its own lock and cache.
SOURCE_COMMIT="$(git -C "$ROOT" rev-parse HEAD)"
"${SSH[@]}" "bash -lc 'cd ~/tbg-src && flock -n -E 75 /tmp/tbg-validation-\$(id -u).lock env CARGO_BUILD_JOBS=2 TBG_SOURCE_COMMIT=$SOURCE_COMMIT cargo build --locked --release -p tbg-server --bin tbg-server --bin tbg-release-admin'"

# SIGTERM drains real matches without a time limit. Back up an existing database
# before migration; opening a nonexistent database as root would break first boot.
"${SSH[@]}" "sudo install -m 755 ~/tbg-src/deploy/backup.sh ~/tbg-src/deploy/release-slot.sh /opt/tbg/bin/ \
	&& sudo install -m 755 ~/tbg-src/target/release/tbg-release-admin /opt/tbg/bin/ \
	&& sudo systemctl stop tbg-server \
	&& if [ -f /var/lib/tbg/tbg.sqlite3 ]; then sudo /opt/tbg/bin/backup.sh --predeploy; else echo 'first install: no database to back up'; fi \
	&& sudo install -m 755 ~/tbg-src/target/release/tbg-server /opt/tbg/bin/"
if [ "$BACKEND_ONLY" = 0 ]; then
	# Open tabs keep their hashed bundles during a release transition.
	"${SSH[@]}" 'sudo rsync -a ~/tbg-web/ /srv/tbg/web/'
fi
"${SSH[@]}" 'sudo systemctl start tbg-server'

for _ in $(seq 1 30); do
	if curl -fsS --connect-timeout 5 --max-time 8 "https://$TBG_DOMAIN/health" | grep -qx ok; then
		echo "deploy ok: https://$TBG_DOMAIN"
		exit 0
	fi
	sleep 2
done
echo "deploy finished but https://$TBG_DOMAIN/health did not answer ok" >&2
exit 1
