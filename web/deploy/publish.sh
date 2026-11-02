#!/usr/bin/env bash
# Publish the static web frontends (landing, public admin placeholder) without touching the game
# backend. Releases are uploaded into new immutable directories and activated by an atomic symlink
# switch; the Caddy include is validated before any reload. See web/deploy/README.md.
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/../.." && pwd)
HERE="$ROOT/web/deploy"
SITE_DOMAIN=${TBG_SITE_DOMAIN:-tupibg.com}
ADMIN_DOMAIN=${TBG_ADMIN_DOMAIN:-admin.$SITE_DOMAIN}
DEST=${TBG_WEB_SSH:-}
PORT=""
CONFIRM=0
ALLOW_DIRTY=0
ARGS=()

usage() {
	cat <<'EOF'
usage: web/deploy/publish.sh [--host <ssh destination>] [--confirm] <command> [args]

  check                      run the local web checks (no network)
  upload <landing|admin>     stage and upload a new release; does not activate it
  activate <site> <release>  switch the site to an uploaded release      (needs --confirm)
  rollback <site>            switch back to the previously active release (needs --confirm)
  caddy-check                back up the live Caddyfile privately and validate the candidate
                             (live config + web include) on the host; changes nothing served
  caddy-apply                validate, install the include, reload Caddy  (needs --confirm)
  caddy-disable              remove the web include and reload Caddy      (needs --confirm)
  status                     releases, active links and the Caddy include state

The host comes from --host, TBG_WEB_SSH or the ignored deploy/.env.deploy. Domains default to
tupibg.com, www.tupibg.com and admin.tupibg.com (TBG_SITE_DOMAIN, TBG_ADMIN_DOMAIN).
EOF
}

while [ $# -gt 0 ]; do
	case "$1" in
	--host) DEST=$2; shift 2 ;;
	--confirm) CONFIRM=1; shift ;;
	--allow-dirty) ALLOW_DIRTY=1; shift ;;
	-h | --help) usage; exit 0 ;;
	*) ARGS+=("$1"); shift ;;
	esac
done
set -- "${ARGS[@]+"${ARGS[@]}"}"

resolve_host() {
	if [ -z "$DEST" ] && [ -f "$ROOT/deploy/.env.deploy" ]; then
		# shellcheck disable=SC1091
		source "$ROOT/deploy/.env.deploy"
		DEST="${TBG_SSH_USER:?}@${TBG_HOST:?}"
		PORT=${TBG_SSH_PORT:-}
	fi
	[ -n "$DEST" ] || { echo "no host: pass --host <ssh alias> or set TBG_WEB_SSH" >&2; exit 2; }
	SSH=(ssh -o BatchMode=yes ${PORT:+-p "$PORT"} "$DEST")
	RSYNC_SSH="ssh -o BatchMode=yes${PORT:+ -p $PORT}"
}

require_confirm() {
	if [ "$CONFIRM" != 1 ]; then
		echo "$1 changes what the public hosts serve; re-run with --confirm after the owner's review." >&2
		exit 2
	fi
}

stage_name() {
	case "$1" in
	landing) echo landing ;;
	admin) echo admin-public ;;
	*) echo "unknown site '$1' (landing|admin)" >&2; exit 2 ;;
	esac
}

prepare_remote() {
	# shellcheck disable=SC2016 # $HOME expands on the host
	"${SSH[@]}" 'install -d -m 700 "$HOME/tbg-web-upload"'
}

remote() { # <remote.sh arguments...>; the helper and its inputs live in ~/tbg-web-upload
	prepare_remote
	rsync -rlt -e "$RSYNC_SSH" "$HERE/remote.sh" "$DEST:tbg-web-upload/remote.sh"
	"${SSH[@]}" "sudo -n bash \"\$HOME/tbg-web-upload/remote.sh\" $*"
}

render_include() { # <output file>
	sed -e "s/__TBG_SITE_DOMAIN__/$SITE_DOMAIN/g" -e "s/__TBG_ADMIN_DOMAIN__/$ADMIN_DOMAIN/g" \
		"$HERE/Caddyfile.web.template" >"$1"
	if grep -q '__TBG_' "$1"; then echo "unrendered placeholder in the Caddy include" >&2; exit 1; fi
}

upload_include() {
	local include
	include=$(mktemp)
	render_include "$include"
	rsync -rlt -e "$RSYNC_SSH" "$include" "$DEST:tbg-web-upload/sites.caddy"
	rm -f -- "$include"
}

private_caddy_copy() {
	local dir="${XDG_STATE_HOME:-$HOME/.local/state}/tbg-web-deploy"
	install -d -m 700 "$dir"
	"${SSH[@]}" 'sudo -n cat /etc/caddy/Caddyfile' >"$dir/Caddyfile.$(date -u +%Y%m%dT%H%M%SZ)"
	echo "local private copy of the live Caddyfile: $dir"
}

cmd_upload() {
	local site=$1 stage_site release stage dirty
	stage_site=$(stage_name "$site")
	dirty=$(git -C "$ROOT" status --porcelain -- web docs/patches crates/tbg-client/assets art/branding)
	if [ -n "$dirty" ] && [ "$ALLOW_DIRTY" != 1 ]; then
		echo "uncommitted web/content changes; commit first so the release matches a revision" >&2
		exit 2
	fi
	python3 "$ROOT/web/scripts/check.py"
	release="$(date -u +%Y%m%dT%H%M%SZ)-$(git -C "$ROOT" rev-parse --short=12 HEAD)"
	stage=$(mktemp -d)
	python3 "$ROOT/web/scripts/stage.py" "$stage_site" "$stage/$site"
	resolve_host
	prepare_remote
	# A fresh per-release directory: nothing existing is overwritten or deleted.
	rsync -rlt -e "$RSYNC_SSH" "$stage/$site/" "$DEST:tbg-web-upload/$site-$release/"
	rm -rf -- "$stage"
	remote install-release "$site" "$release" "\"\$HOME/tbg-web-upload/$site-$release\""
	echo "next, after review: web/deploy/publish.sh activate $site $release --confirm"
}

case "${1:-}" in
check) python3 "$ROOT/web/scripts/check.py" ;;
upload) [ $# = 2 ] || { usage; exit 2; }; cmd_upload "$2" ;;
activate)
	[ $# = 3 ] || { usage; exit 2; }
	stage_name "$2" >/dev/null
	require_confirm activate
	resolve_host
	remote activate "$2" "$3"
	;;
rollback)
	[ $# = 2 ] || { usage; exit 2; }
	stage_name "$2" >/dev/null
	require_confirm rollback
	resolve_host
	remote rollback "$2"
	;;
caddy-check)
	resolve_host
	prepare_remote
	private_caddy_copy
	upload_include
	remote caddy-check "\"\$HOME/tbg-web-upload/sites.caddy\""
	;;
caddy-apply)
	require_confirm caddy-apply
	resolve_host
	prepare_remote
	private_caddy_copy
	upload_include
	remote caddy-apply "\"\$HOME/tbg-web-upload/sites.caddy\""
	;;
caddy-disable)
	require_confirm caddy-disable
	resolve_host
	private_caddy_copy
	remote caddy-disable
	;;
status)
	resolve_host
	remote status
	;;
*) usage; exit 2 ;;
esac
