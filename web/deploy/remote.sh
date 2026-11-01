#!/usr/bin/env bash
# Host side of web/deploy/publish.sh, run as root through sudo. It touches only
# /srv/tbg/{landing,admin}, /etc/caddy/tbg-web, the import line of the main Caddyfile and
# /var/backups/tbg-web. Never the game server, /srv/tbg/web, /srv/tbg/updates, /opt/tbg,
# /var/lib/tbg or /var/backups/tbg.
set -euo pipefail
umask 022

SRV=/srv/tbg
CADDYFILE=/etc/caddy/Caddyfile
INCLUDE_DIR=/etc/caddy/tbg-web
IMPORT_LINE='import tbg-web/*.caddy'
BACKUPS=/var/backups/tbg-web

die() {
	echo "remote: $*" >&2
	exit 1
}

site_dir() {
	case "$1" in
	landing | admin) echo "$SRV/$1" ;;
	*) die "unknown site: $1" ;;
	esac
}

check_release() {
	[[ "$1" =~ ^[0-9]{8}T[0-9]{6}Z-[0-9a-f]{7,40}$ ]] || die "bad release id: $1"
}

# Uploads arrive in the SSH user's private ~/tbg-web-upload; refuse anything else before deleting.
check_upload() {
	[[ "$1" =~ ^/(root|home/[a-z_][a-z0-9_-]*)/tbg-web-upload/[A-Za-z0-9.-]+$ ]] || die "unexpected upload path: $1"
}

install_release() { # <site> <release> <upload dir>
	local dir target
	dir=$(site_dir "$1")
	check_release "$2"
	check_upload "$3"
	target="$dir/releases/$2"
	[ -f "$3/index.html" ] || die "upload has no index.html"
	if [ -e "$target" ] || [ -e "$target.partial" ]; then die "release already exists: $target"; fi
	install -d -o root -g root -m 755 "$dir" "$dir/releases"
	cp -R "$3" "$target.partial"
	chown -R root:root "$target.partial"
	find "$target.partial" -type d -exec chmod 755 {} +
	find "$target.partial" -type f -exec chmod 644 {} +
	mv -T "$target.partial" "$target"
	rm -rf -- "$3"
	echo "installed $1 release $2 (not active yet)"
}

activate() { # <site> <release>
	local dir
	dir=$(site_dir "$1")
	check_release "$2"
	[ -f "$dir/releases/$2/index.html" ] || die "no such release: $1 $2"
	# rename(2) swaps the link atomically: requests see the old or the new release, never a mix.
	ln -sfn "releases/$2" "$dir/current.next"
	mv -T "$dir/current.next" "$dir/current"
	echo "$(date -u +%Y-%m-%dT%H:%M:%SZ) $2" >>"$dir/activations.log"
	echo "active $1 release: $2"
}

rollback() { # <site>
	local dir now id previous=""
	dir=$(site_dir "$1")
	now=$(readlink "$dir/current" 2>/dev/null || true)
	now=${now#releases/}
	[ -n "$now" ] || die "no active $1 release"
	[ -f "$dir/activations.log" ] || die "no activation history for $1"
	while read -r id; do
		if [ "$id" != "$now" ] && [ -f "$dir/releases/$id/index.html" ]; then
			previous=$id
			break
		fi
	done < <(awk '{print $2}' "$dir/activations.log" | tac)
	[ -n "$previous" ] || die "no earlier $1 release to roll back to"
	activate "$1" "$previous"
}

backup_caddy() {
	local stamp
	stamp=$(date -u +%Y%m%dT%H%M%SZ)
	install -d -o root -g root -m 700 "$BACKUPS"
	cp -p "$CADDYFILE" "$BACKUPS/Caddyfile.$stamp"
	if [ -d "$INCLUDE_DIR" ]; then cp -a "$INCLUDE_DIR" "$BACKUPS/tbg-web.$stamp"; fi
	echo "$stamp"
}

# Candidate = live Caddyfile (+ the import line when missing) and the new include, in a private
# temporary directory; relative imports resolve next to the candidate, so validation covers both.
render_candidate() { # <snippet file>
	local candidate
	candidate=$(mktemp -d)
	cp "$CADDYFILE" "$candidate/Caddyfile"
	if ! grep -qxF "$IMPORT_LINE" "$candidate/Caddyfile"; then
		printf '\n# Static web frontends (web/deploy/publish.sh); an empty include is only a warning.\n%s\n' \
			"$IMPORT_LINE" >>"$candidate/Caddyfile"
	fi
	install -d -m 700 "$candidate/tbg-web"
	install -m 644 "$1" "$candidate/tbg-web/sites.caddy"
	echo "$candidate"
}

caddy_check() { # <snippet file>
	local stamp candidate status=0
	check_upload "$1"
	stamp=$(backup_caddy)
	candidate=$(render_candidate "$1")
	caddy validate --config "$candidate/Caddyfile" --adapter caddyfile || status=$?
	rm -rf -- "$candidate"
	[ "$status" = 0 ] || die "candidate INVALID; live configuration untouched (backup $stamp)"
	echo "candidate valid; nothing applied (private backup $BACKUPS/Caddyfile.$stamp)"
}

restore_caddy() { # <stamp>
	install -o root -g root -m 644 "$BACKUPS/Caddyfile.$1" "$CADDYFILE"
	rm -rf -- "$INCLUDE_DIR"
	if [ -d "$BACKUPS/tbg-web.$1" ]; then cp -a "$BACKUPS/tbg-web.$1" "$INCLUDE_DIR"; fi
}

caddy_apply() { # <snippet file>
	local stamp candidate
	check_upload "$1"
	stamp=$(backup_caddy)
	candidate=$(render_candidate "$1")
	if ! caddy validate --config "$candidate/Caddyfile" --adapter caddyfile; then
		rm -rf -- "$candidate"
		die "candidate invalid; nothing applied (backup $stamp)"
	fi
	install -d -o root -g root -m 755 "$INCLUDE_DIR"
	install -o root -g root -m 644 "$candidate/tbg-web/sites.caddy" "$INCLUDE_DIR/sites.caddy"
	install -o root -g root -m 644 "$candidate/Caddyfile" "$CADDYFILE"
	rm -rf -- "$candidate"
	if caddy validate --config "$CADDYFILE" --adapter caddyfile && systemctl reload caddy; then
		echo "caddy reloaded with the web include (backup $stamp)"
		return
	fi
	echo "remote: reload failed; restoring backup $stamp" >&2
	restore_caddy "$stamp"
	systemctl reload caddy || true
	exit 1
}

caddy_disable() {
	local stamp
	stamp=$(backup_caddy)
	rm -f -- "$INCLUDE_DIR/sites.caddy"
	if caddy validate --config "$CADDYFILE" --adapter caddyfile && systemctl reload caddy; then
		echo "web hosts removed from Caddy (backup $stamp); API/app blocks unchanged"
		return
	fi
	restore_caddy "$stamp"
	systemctl reload caddy || true
	die "could not reload without the include; backup $stamp restored"
}

status() {
	local site dir
	for site in landing admin; do
		dir="$SRV/$site"
		echo "== $site: active $(readlink "$dir/current" 2>/dev/null || echo none)"
		if [ -d "$dir/releases" ]; then find "$dir/releases" -mindepth 1 -maxdepth 1 -printf '   %f\n' | sort | tail -n 5; fi
	done
	if grep -qxF "$IMPORT_LINE" "$CADDYFILE"; then echo "caddy: main Caddyfile imports tbg-web/*.caddy"; else echo "caddy: import line absent"; fi
	if [ -f "$INCLUDE_DIR/sites.caddy" ]; then echo "caddy: web include installed"; else echo "caddy: web include not installed"; fi
	echo "caddy: $(caddy version)"
}

case "${1:-}" in
install-release) install_release "$2" "$3" "$4" ;;
activate) activate "$2" "$3" ;;
rollback) rollback "$2" ;;
caddy-check) caddy_check "$2" ;;
caddy-apply) caddy_apply "$2" ;;
caddy-disable) caddy_disable ;;
status) status ;;
*) die "usage: remote.sh install-release|activate|rollback|caddy-check|caddy-apply|caddy-disable|status" ;;
esac
