#!/usr/bin/env bash
# One-time VPS setup (idempotent). Run on the VPS as root:
#   sudo bash ~/tbg-src/deploy/provision.sh <domain> <ssh-user>
# Ubuntu 22.04/24.04 or Debian 12. See deploy/README.md.
set -euo pipefail

DOMAIN="${1:?usage: provision.sh <domain> <ssh-user>}"
SSH_USER="${2:?usage: provision.sh <domain> <ssh-user>}"
HERE="$(cd "$(dirname "$0")" && pwd)"

apt-get update
if ! apt-cache show caddy >/dev/null 2>&1; then
	# Ubuntu 22.04 has no caddy package: official repository (caddyserver.com/docs/install).
	apt-get install -y debian-keyring debian-archive-keyring apt-transport-https curl gnupg
	curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | gpg --dearmor --yes -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
	curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' > /etc/apt/sources.list.d/caddy-stable.list
	apt-get update
fi
apt-get install -y caddy build-essential pkg-config curl sqlite3 ufw rsync

# 2 GB swap on small machines so cargo can link.
if [ "$(awk '/MemTotal/ {print $2}' /proc/meminfo)" -lt 3000000 ] && ! swapon --show | grep -q .; then
	fallocate -l 2G /swapfile && chmod 600 /swapfile && mkswap /swapfile && swapon /swapfile
	grep -q '/swapfile' /etc/fstab || echo '/swapfile none swap sw 0 0' >> /etc/fstab
fi

id -u tbg >/dev/null 2>&1 || useradd --system --home /var/lib/tbg --shell /usr/sbin/nologin tbg
install -d -o root -g root /opt/tbg/bin /srv/tbg/web /var/backups/tbg
install -d -o tbg -g tbg /var/lib/tbg

if ! sudo -u "$SSH_USER" bash -lc 'command -v cargo' >/dev/null 2>&1; then
	sudo -u "$SSH_USER" bash -lc 'curl https://sh.rustup.rs -sSf | sh -s -- -y --profile minimal'
fi

ufw allow OpenSSH
ufw allow 80,443/tcp
ufw --force enable

install -m 644 "$HERE/tbg-server.service" /etc/systemd/system/tbg-server.service
systemctl daemon-reload
systemctl enable tbg-server

sed "s/__TBG_DOMAIN__/$DOMAIN/" "$HERE/Caddyfile.template" > /etc/caddy/Caddyfile
systemctl enable caddy
systemctl reload caddy || systemctl restart caddy

echo '30 3 * * * root /opt/tbg/bin/backup.sh >> /var/log/tbg-backup.log 2>&1' > /etc/cron.d/tbg-backup
echo "provision ok for $DOMAIN"
