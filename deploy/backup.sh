#!/usr/bin/env bash
# Nightly SQLite backup on the VPS (cron: /etc/cron.d/tbg-backup).
# Keeps the 7 newest copies. Optional weekly off-site copy (Sundays) to
# TBG_BACKUP_REMOTE, an rsync/scp target set in /etc/default/tbg-backup.
set -euo pipefail

DB="${TBG_DB:-/var/lib/tbg/tbg.sqlite3}"
DIR="${TBG_BACKUP_DIR:-/var/backups/tbg}"
# shellcheck disable=SC1091
[ -f /etc/default/tbg-backup ] && . /etc/default/tbg-backup

mkdir -p "$DIR"
out="$DIR/tbg-$(date +%F).sqlite3"
sqlite3 "$DB" ".backup '$out'"
# Dated names sort chronologically; keep the 7 newest.
find "$DIR" -maxdepth 1 -name 'tbg-*.sqlite3' -printf '%f\n' | sort -r | tail -n +8 | while read -r f; do rm -- "$DIR/$f"; done
if [ -n "${TBG_BACKUP_REMOTE:-}" ] && [ "$(date +%u)" = "7" ]; then
	rsync -a "$out" "$TBG_BACKUP_REMOTE"
fi
echo "backup ok: $out"
