#!/usr/bin/env bash
# Build locally, copy to the VPS, build the server there, restart.
#   bash deploy/deploy.sh [--provision]
# Reads deploy/.env.deploy (never committed). See deploy/README.md.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck disable=SC1091
source "$ROOT/deploy/.env.deploy"
: "${TBG_HOST:?}" "${TBG_SSH_USER:?}" "${TBG_DOMAIN:?}"
PORT="${TBG_SSH_PORT:-22}"
TARGET="$TBG_SSH_USER@$TBG_HOST"
SSH=(ssh -p "$PORT" "$TARGET")

(cd "$ROOT/crates/tbg-client" && flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" \
	env CARGO_BUILD_JOBS=2 trunk build --release)

rsync -az --delete -e "ssh -p $PORT" \
	--exclude target/ --exclude dist/ --exclude .git/ --exclude archive/ \
	--exclude .local/ --exclude .agents/ --exclude .codex/ --exclude .env --exclude '*.pem' \
	--exclude deploy/.env.deploy "$ROOT/" "$TARGET:tbg-src/"
rsync -az --delete -e "ssh -p $PORT" "$ROOT/crates/tbg-client/dist/" "$TARGET:tbg-web/"

if [ "${1:-}" = "--provision" ]; then
	"${SSH[@]}" "sudo bash ~/tbg-src/deploy/provision.sh '$TBG_DOMAIN' '$TBG_SSH_USER'"
fi

# P60: the new binary migrates the database on start, so stop the old one,
# take an integrity-checked pre-migration backup, and only then install and
# start. A failed backup aborts the deploy with the old binary stopped:
# start it again by hand after fixing storage (README "Restore drill").
# P66: this is the single-slot fallback. `systemctl stop` waits, without a
# time limit, until running matches end (new matches pause meanwhile); the
# two-slot release in README "Releases" keeps admitting instead. The web
# copy never deletes: open tabs keep fetching their hashed bundle.
"${SSH[@]}" "bash -lc 'cd ~/tbg-src && flock -n -E 75 /tmp/tbg-validation-\$(id -u).lock env CARGO_BUILD_JOBS=2 TBG_SOURCE_COMMIT=$(git -C "$ROOT" rev-parse HEAD) cargo build --locked --release -p tbg-server --bin tbg-server --bin tbg-release-admin' \
	&& sudo install -m 755 ~/tbg-src/deploy/backup.sh ~/tbg-src/deploy/release-slot.sh /opt/tbg/bin/ \
	&& sudo install -m 755 ~/tbg-src/target/release/tbg-release-admin /opt/tbg/bin/ \
	&& sudo systemctl stop tbg-server \
	&& sudo /opt/tbg/bin/backup.sh --predeploy \
	&& sudo install -m 755 ~/tbg-src/target/release/tbg-server /opt/tbg/bin/ \
	&& sudo rsync -a ~/tbg-web/ /srv/tbg/web/ \
	&& sudo systemctl start tbg-server"

for _ in $(seq 1 30); do
	if curl -fsS "https://$TBG_DOMAIN/health" | grep -q ok; then
		echo "deploy ok: https://$TBG_DOMAIN"
		exit 0
	fi
	sleep 2
done
echo "deploy finished but https://$TBG_DOMAIN/health did not answer ok" >&2
exit 1
