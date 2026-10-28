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

(cd "$ROOT/crates/tbg-client" && trunk build --release)

rsync -az --delete -e "ssh -p $PORT" \
	--exclude target/ --exclude dist/ --exclude .git/ --exclude archive/ \
	--exclude deploy/.env.deploy "$ROOT/" "$TARGET:tbg-src/"
rsync -az --delete -e "ssh -p $PORT" "$ROOT/crates/tbg-client/dist/" "$TARGET:tbg-web/"

if [ "${1:-}" = "--provision" ]; then
	"${SSH[@]}" "sudo bash ~/tbg-src/deploy/provision.sh '$TBG_DOMAIN' '$TBG_SSH_USER'"
fi

"${SSH[@]}" "bash -lc 'cd ~/tbg-src && cargo build --release -p tbg-server --bin tbg-server' \
	&& sudo install -m 755 ~/tbg-src/target/release/tbg-server /opt/tbg/bin/ \
	&& sudo install -m 755 ~/tbg-src/deploy/backup.sh /opt/tbg/bin/ \
	&& sudo rsync -a --delete ~/tbg-web/ /srv/tbg/web/ \
	&& sudo systemctl restart tbg-server"

for _ in $(seq 1 30); do
	if curl -fsS "https://$TBG_DOMAIN/health" | grep -q ok; then
		echo "deploy ok: https://$TBG_DOMAIN"
		exit 0
	fi
	sleep 2
done
echo "deploy finished but https://$TBG_DOMAIN/health did not answer ok" >&2
exit 1
