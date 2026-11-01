#!/usr/bin/env bash
# Install the reviewed controller from this computer; never enable it implicitly.
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
if systemctl is-enabled --quiet tbg-auto-deploy.timer || systemctl is-active --quiet tbg-auto-deploy.service; then
	echo 'Disable the timer and let any current deployment finish before replacing the controller.' >&2
	exit 1
fi
install -d -m 755 /opt/tbg/deploy /opt/tbg/releases
install -d -m 700 /var/lib/tbg-deploy
install -m 644 "$HERE/policy.py" "$HERE/activate.py" "$HERE/pull.py" /opt/tbg/deploy/
install -m 644 "$HERE/tbg-auto-deploy.service" "$HERE/tbg-auto-deploy.timer" /etc/systemd/system/
systemctl daemon-reload
printf 'Installed, disabled. Review the runbook and private config before activation.\n'
