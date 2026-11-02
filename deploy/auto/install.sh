#!/usr/bin/env bash
# Install the reviewed controller from this computer; never enable it implicitly.
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
if systemctl is-enabled --quiet tbg-auto-deploy.timer || systemctl is-active --quiet tbg-auto-deploy.service \
	|| systemctl is-enabled --quiet tbg-web-deploy.timer || systemctl is-active --quiet tbg-web-deploy.service \
	|| systemctl is-enabled --quiet tbg-static-deploy.timer || systemctl is-active --quiet tbg-static-deploy.service \
	|| systemctl is-enabled --quiet tbg-native-catalog.timer || systemctl is-active --quiet tbg-native-catalog.service; then
	echo 'Disable the timer and let any current deployment finish before replacing the controller.' >&2
	exit 1
fi
install -d -m 755 /opt/tbg/deploy /opt/tbg/releases /srv/tbg/web /srv/tbg/landing /srv/tbg/admin /srv/tbg/downloads
install -d -m 700 /var/lib/tbg-deploy
install -m 644 "$HERE/policy.py" "$HERE/activate.py" "$HERE/control.py" "$HERE/github.py" "$HERE/pull.py" "$HERE/slots.py" "$HERE/status.py" "$HERE/web_policy.py" "$HERE/web_activate.py" "$HERE/web_pull.py" "$HERE/static_policy.py" "$HERE/static_activate.py" "$HERE/static_pull.py" "$HERE/native_policy.py" "$HERE/native_pull.py" /opt/tbg/deploy/
# The slot coordinator drives these two; the bootstrap script is only run by an operator.
install -m 755 "$HERE/../release-slot.sh" "$HERE/../slot-bootstrap.sh" /opt/tbg/bin/
install -d -m 755 /opt/tbg/slots
install -m 644 "$HERE/tbg-auto-deploy.service" "$HERE/tbg-auto-deploy.timer" "$HERE/tbg-web-deploy.service" "$HERE/tbg-web-deploy.timer" "$HERE/tbg-static-deploy.service" "$HERE/tbg-static-deploy.timer" "$HERE/tbg-native-catalog.service" "$HERE/tbg-native-catalog.timer" /etc/systemd/system/
systemctl daemon-reload
printf 'Installed, disabled. Review the runbook and private config before activation.\n'
