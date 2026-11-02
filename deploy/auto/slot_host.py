"""The machine behind the slot coordinator: loopback status, the shared database, systemd and the proxy."""
import base64
import http.client
import json
import os
import shutil
import sqlite3
import subprocess
import time
from pathlib import Path
from urllib.parse import urlsplit
from urllib.request import Request, urlopen

from activate import BIN, DB, RELEASES, sync_directory, sync_file
from policy import digest, require

PORTS = {'a': 8080, 'b': 8082}
# A slot process heartbeats every second; the server itself treats 15 s of silence as gone (P66).
FRESH_SECONDS = 15


def fetch_json(url, timeout=5):
    request = Request(url, headers={'User-Agent': 'TBG-release-slots/1.0'})
    with urlopen(request, timeout=timeout) as response:
        return response.status, json.loads(response.read(65536))


class SystemHost:
    """Everything the coordinator touches; tests and the local rehearsal substitute their own."""

    def now(self):
        return time.time()

    def sleep(self, seconds):
        time.sleep(seconds)

    def directory(self):
        with sqlite3.connect(f'file:{DB}?mode=ro', uri=True, timeout=5) as connection:
            connection.row_factory = sqlite3.Row
            rows = connection.execute(
                'SELECT slot,generation,contract,state,ranked_authority,matches_running,results_pending,'
                'ranked_inflight,heartbeat_unix,version FROM release_slots ORDER BY slot').fetchall()
        return [dict(row, fresh=self.now() - row['heartbeat_unix'] <= FRESH_SECONDS) for row in rows]

    def schema(self):
        with sqlite3.connect(f'file:{DB}?mode=ro', uri=True, timeout=5) as connection:
            return connection.execute('SELECT max(version) FROM schema_version').fetchone()[0]

    def status(self, slot):
        """This slot's own /api/release, or None while its process is not answering."""
        try:
            return fetch_json(f'http://127.0.0.1:{PORTS[slot]}/api/release')[1]
        except (OSError, ValueError):
            return None

    def metrics(self, slot):
        try:
            return fetch_json(f'http://127.0.0.1:{PORTS[slot]}/metrics')[1]
        except (OSError, ValueError):
            return None

    def store_ok(self, slot):
        try:
            return fetch_json(f'http://127.0.0.1:{PORTS[slot]}/api/leaderboard?limit=1')[0] == 200
        except (OSError, ValueError):
            return False

    def public_admission(self, base, contract):
        """What a player's browser is told right now, through the public proxy."""
        try:
            return fetch_json(f'{base}/api/admission?contract={contract}&build=auto-deploy&outcome=none')[1]
        except (OSError, ValueError):
            return None

    def public_ok(self, base):
        try:
            request = Request(base + '/health', headers={'User-Agent': 'TBG-release-slots/1.0'})
            with urlopen(request, timeout=5) as response:
                return response.status == 200 and response.read(8) == b'ok'
        except OSError:
            return False

    def public_ws(self, base, slot, contract):
        """A WebSocket upgrade through the public proxy reaches this slot: the proxy routes /ws/slot-x and the
        server answers 404 for another slot's path, so a 101 proves the route players will use. No Origin header: a
        native-style upgrade, accepted whatever origins the server is configured with."""
        parts = urlsplit(base)
        connection = (http.client.HTTPSConnection if parts.scheme == 'https' else http.client.HTTPConnection)(parts.netloc, timeout=5)
        try:
            connection.request('GET', f'/ws/slot-{slot}?contract={contract}', headers={
                'Connection': 'Upgrade', 'Upgrade': 'websocket', 'Sec-WebSocket-Version': '13',
                'Sec-WebSocket-Key': base64.b64encode(os.urandom(16)).decode(), 'User-Agent': 'TBG-release-slots/1.0'})
            return connection.getresponse().status == 101
        except (OSError, http.client.HTTPException):
            return False
        finally:
            connection.close()

    def admin(self, *args):
        result = subprocess.run(['runuser', '-u', 'tbg', '--', str(BIN/'tbg-release-admin'), '--db', str(DB), *map(str, args)],
                                capture_output=True, text=True, timeout=60)
        return result.returncode, (result.stdout + result.stderr).strip()

    def is_active(self, slot):
        return subprocess.run(['systemctl', 'is-active', '--quiet', f'tbg-server@{slot}']).returncode == 0

    def inspect(self, staged, manifest):
        """The candidate describes itself, as nobody, before it touches the shared database."""
        folder = RELEASES/('.inspect-' + manifest['source_commit'][:12])
        shutil.rmtree(folder, ignore_errors=True)
        folder.mkdir(mode=0o755)
        try:
            folder.chmod(0o755)
            binary = folder/'tbg-server'
            shutil.copy2(Path(staged)/'tbg-server', binary)
            binary.chmod(0o755)
            result = subprocess.run(['runuser', '-u', 'nobody', '--', 'env', '-i', 'PATH=/usr/bin:/bin', str(binary), '--build-info'],
                                    capture_output=True, timeout=10)
            require(result.returncode == 0, 'the candidate does not describe itself')
            return json.loads(result.stdout)
        finally:
            shutil.rmtree(folder, ignore_errors=True)

    def start_slot(self, slot, generation, binary, build):
        result = subprocess.run([str(BIN/'release-slot.sh'), 'start', slot, generation, str(binary), build],
                                capture_output=True, text=True, timeout=240)
        return result.returncode, (result.stdout + result.stderr).strip()

    def retire_slot(self, slot):
        """One retirement check, then stop: the script's own wait loop must not hold the shared lock."""
        try:
            result = subprocess.run([str(BIN/'release-slot.sh'), 'retire', slot], capture_output=True, text=True, timeout=120,
                                    env={'PATH': '/usr/sbin:/usr/bin:/sbin:/bin', 'TBG_RETIRE_CHECKS': '1'})
        except subprocess.TimeoutExpired:
            # The stop job has no deadline by design (a drain never kills a match); it outlives this call.
            return 3, 'the slot is still stopping'
        return result.returncode, (result.stdout + result.stderr).strip()

    def stop_slot(self, slot):
        # SIGTERM is a drain: an idle slot exits at once, a busy one would wait without a deadline.
        subprocess.run(['systemctl', 'disable', '--now', f'tbg-server@{slot}'], check=True, timeout=300)

    def free_disk(self):
        return shutil.disk_usage(RELEASES).free

    def free_memory_mb(self):
        for line in Path('/proc/meminfo').read_text().splitlines():
            if line.startswith('MemAvailable:'):
                return int(line.split()[1]) // 1024
        return 0

    def discard(self, generation):
        """Forget the files of a generation that never admitted a player; a served generation is never deleted."""
        shutil.rmtree(RELEASES/generation, ignore_errors=True)

    def keep_files(self, staged, generation, manifest):
        """The slot script installs the server; keep the operator tool and check both digests."""
        release = RELEASES/generation
        shutil.copy2(Path(staged)/'tbg-release-admin', release/'tbg-release-admin')
        (release/'tbg-release-admin').chmod(0o755)
        sync_file(release/'tbg-release-admin')
        for name, expected in manifest['files'].items():
            require(digest(release/name) == expected, f'installed {name} digest mismatch')
        return release

    def adopt_admin(self, release):
        """Point the operator tool at the promoted generation (same schema, same contract)."""
        link = BIN/'tbg-release-admin.next'
        link.unlink(missing_ok=True)
        link.symlink_to(release/'tbg-release-admin')
        link.replace(BIN/'tbg-release-admin')
        sync_directory(BIN)
