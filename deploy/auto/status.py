#!/usr/bin/env python3
"""Read-only status of every delivery family: timer, flag, last receipt, last check, holds and journals."""
import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path

STATE = Path('/var/lib/tbg-deploy')
FLAGS = Path('/etc/tbg')
# family -> (systemd unit prefix, enable-flag name, receipt file, check file, journals)
FAMILIES = {
    'backend': ('tbg-auto-deploy', 'auto-deploy', 'current.json', 'checked.json', ['slot-transition.json', 'transition.json']),
    'web': ('tbg-web-deploy', 'web-deploy', 'web-current.json', 'web-checked.json', []),
    'static': ('tbg-static-deploy', 'static-deploy', 'static-current.json', 'static-checked.json', []),
    'native': ('tbg-native-catalog', 'native-catalog', 'native-current.json', 'native-checked.json', []),
}


def systemctl(*args):
    result = subprocess.run(['systemctl', *args], capture_output=True, text=True, timeout=10)
    return result.stdout.strip()


def read(path):
    try:
        return json.loads(path.read_text())
    except (OSError, ValueError):
        return None


def short(value):
    return str(value)[:12] if value else '-'


def modified(path):
    try:
        return datetime.fromtimestamp(path.stat().st_mtime, timezone.utc).strftime('%Y-%m-%d %H:%M:%SZ')
    except OSError:
        return '-'


def format_time(value):
    try:
        return datetime.fromtimestamp(float(value), timezone.utc).strftime('%Y-%m-%d %H:%M:%SZ')
    except (TypeError, ValueError, OSError):
        return '-'


def describe(name, unit, flag, receipt, check, journals):
    current, checked = read(STATE/receipt), read(STATE/check)
    lines = [f'{name}:',
             f'  timer   {systemctl("is-enabled", unit + ".timer") or "-"} / {systemctl("is-active", unit + ".timer") or "-"}'
             f'   flag {"present" if (FLAGS/(flag + ".enabled")).exists() else "absent"}'
             f'   last run {systemctl("show", unit + ".timer", "-p", "LastTriggerUSec", "--value") or "-"}'
             f' ({systemctl("show", unit + ".service", "-p", "Result", "--value") or "-"})',
             f'  applied {short((current or {}).get("source_commit") or (current or {}).get("tag"))}   ({modified(STATE/receipt)})']
    if checked:
        lines.append(f'  checked {short(checked.get("source_commit") or checked.get("tag"))}: {checked.get("outcome", "?")}'
                     + (f' — {checked["reason"]}' if checked.get('reason') else ''))
    for journal in journals:
        if (STATE/journal).exists():
            data = read(STATE/journal) or {}
            lines.append(f'  JOURNAL {journal}: phase {data.get("phase", "?")} {short(data.get("source_commit"))}'
                         f' (updated {modified(STATE/journal)}, started {format_time(data.get("started"))})')
    return lines


def main():
    hold = read(STATE/'operator-hold.json')
    print('operator hold: ' + (f'{hold.get("family")} — {hold.get("reason")} (clear with pull.py --release-hold)' if hold else 'none'))
    for name, details in FAMILIES.items():
        print('\n'.join(describe(name, *details)))
    print('release slots:')
    print(subprocess.run(['runuser', '-u', 'tbg', '--', '/opt/tbg/bin/tbg-release-admin', '--db', '/var/lib/tbg/tbg.sqlite3', 'status'],
                         capture_output=True, text=True, timeout=15).stdout.rstrip() or '  (unavailable)')


if __name__ == '__main__':
    main()
