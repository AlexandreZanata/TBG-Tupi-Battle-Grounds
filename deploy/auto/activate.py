"""Activate compatible backend binaries with durable rollback, never restoring player data."""
import json
import os
import shutil
import sqlite3
import subprocess
import time
from pathlib import Path
from urllib.request import Request, urlopen

from policy import compatible, digest, require

BIN = Path('/opt/tbg/bin')
RELEASES = Path('/opt/tbg/releases')
DB = Path('/var/lib/tbg/tbg.sqlite3')
STATE = Path('/var/lib/tbg-deploy')
JOURNAL = STATE/'transition.json'


def write_json(path, value):
    temporary = path.with_suffix('.next')
    with temporary.open('w') as out:
        json.dump(value, out, sort_keys=True)
        out.write('\n')
        out.flush()
        os.fsync(out.fileno())
    temporary.chmod(0o600)
    os.replace(temporary, path)
    sync_directory(path.parent)


def sync_directory(path):
    fd = os.open(path, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(fd)
    finally:
        os.close(fd)


def sync_file(path):
    with path.open('rb') as stream:
        os.fsync(stream.fileno())
    sync_directory(path.parent)


def status():
    with urlopen('http://127.0.0.1:8080/api/release', timeout=5) as response:
        return json.loads(response.read(65536))


def require_running_binary(expected):
    # An unused legacy install path must never produce a false successful promotion.
    pid = subprocess.check_output(
        ['systemctl', 'show', 'tbg-server.service', '-p', 'MainPID', '--value'], text=True, timeout=5,
    ).strip()
    require(pid.isdecimal() and int(pid) > 0, 'game service has no running main process')
    actual = Path('/proc')/pid/'exe'
    require(actual.resolve(strict=True) == Path(expected).resolve(strict=True),
            'runtime executable path requires explicit adoption; no deployment applied')


def service(action):
    # TimeoutStopSec=infinity is intentional: never kill a match to finish deploy.
    subprocess.run(['systemctl', action, 'tbg-server.service'], check=True)


def switch(name, target):
    require(target.is_relative_to(RELEASES) and target.is_file(), 'invalid release target')
    temporary = BIN/(name+'.next')
    temporary.unlink(missing_ok=True)
    temporary.symlink_to(target)
    os.replace(temporary, BIN/name)
    sync_directory(BIN)


def healthy(contract, public_url):
    for _ in range(20):
        try:
            live = status()
            request = Request(public_url, headers={'User-Agent': 'TBG-release-health/1.0'})
            with urlopen(request, timeout=5) as response:
                public_ok = response.status == 200 and response.read(8) == b'ok'
            with urlopen('http://127.0.0.1:8080/api/leaderboard?limit=1', timeout=5) as response:
                store_ok = response.status == 200
            if live['contract'] == contract and live['admitting'] and public_ok and store_ok:
                return True
        except (OSError, ValueError, KeyError):
            pass
        time.sleep(1)
    return False


def rollback(journal):
    service('stop')
    for name, path in journal['previous'].items():
        switch(name, Path(path))
    service('start')
    require_running_binary(BIN/'tbg-server')
    require(healthy(journal['contract'], journal['public_health']), 'rollback health failed; operator required')
    if journal.get('previous_receipt') is None:
        (STATE/'current.json').unlink(missing_ok=True)
    else:
        write_json(STATE/'current.json', journal['previous_receipt'])
    JOURNAL.unlink()
    sync_directory(STATE)


def recover():
    if JOURNAL.exists():
        rollback(json.loads(JOURNAL.read_text()))


def check_compatibility(manifest):
    require_running_binary(BIN/'tbg-server')
    live = status()
    with sqlite3.connect(f'file:{DB}?mode=ro', uri=True) as connection:
        schema = connection.execute('SELECT max(version) FROM schema_version').fetchone()[0]
    compatible(manifest, live, schema)
    return live


def activate(staged, manifest, public_health):
    live = check_compatibility(manifest)
    require(shutil.disk_usage(RELEASES).free >= 512 * 1024 * 1024, 'insufficient release/backup space')
    candidate = RELEASES/manifest['source_commit']
    candidate.mkdir(mode=0o755, exist_ok=True)
    candidate.chmod(0o755)
    sync_directory(RELEASES)
    previous = {}
    for name, expected in manifest['files'].items():
        old = BIN/name
        old_copy = RELEASES/('previous-'+digest(old))
        old_copy.mkdir(mode=0o755, exist_ok=True)
        old_copy.chmod(0o755)
        if not (old_copy/name).exists():
            shutil.copy2(old, old_copy/name)
            sync_file(old_copy/name)
            sync_directory(RELEASES)
        previous[name] = str(old_copy/name)
        target = candidate/name
        if target.exists():
            require(digest(target) == expected, 'immutable release collision')
        else:
            shutil.copy2(Path(staged)/name, target)
            target.chmod(0o755)
            sync_file(target)
    inspected = subprocess.run(
        ['runuser', '-u', 'nobody', '--', 'env', '-i', 'PATH=/usr/bin:/bin', str(candidate/'tbg-server'), '--build-info'],
        check=True, capture_output=True, timeout=5,
    )
    info = json.loads(inspected.stdout)
    for key in ['source_commit', 'contract', 'database_schema', 'target']:
        require(info.get(key) == manifest.get(key), 'candidate identity mismatch')
    journal = {'previous': previous, 'contract': live['contract'], 'public_health': public_health,
               'previous_receipt': json.loads((STATE/'current.json').read_text()) if (STATE/'current.json').exists() else None}
    write_json(JOURNAL, journal)
    try:
        service('stop')
        subprocess.run([str(BIN/'backup.sh'), '--predeploy'], check=True)
        for name in manifest['files']:
            switch(name, candidate/name)
        service('start')
        require_running_binary(candidate/'tbg-server')
        require(healthy(manifest['contract'], public_health), 'candidate health failed')
        write_json(STATE/'current.json', manifest)
        JOURNAL.unlink()
        sync_directory(STATE)
    except BaseException:
        rollback(journal)
        raise
