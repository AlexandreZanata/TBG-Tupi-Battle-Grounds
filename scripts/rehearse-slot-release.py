#!/usr/bin/env python3
"""Rehearse automatic slot promotion on this computer: real server generations, the real coordinator, synthetic bots.

Nothing here touches production: the database, ports, release directories and accounts are temporary.
Two or more release swaps run while bots play casual and ranked matches through an admission step, and the
run reports every interruption a player could have seen (a failed match, a failed admission, a failed
health check) together with the coordinator's timeline.

  scripts/rehearse-slot-release.py --old-bin <tbg-server of the live generation> --new-bin <tbg-server> \\
      --admin-bin <tbg-release-admin> --bots-bin <tbg-bots> --out <empty dir>
"""
import argparse
import hashlib
import itertools
import json
import os
import shutil
import signal
import statistics
import subprocess
import sys
import tempfile
import threading
import time
from pathlib import Path
from urllib.request import urlopen

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT/'deploy/auto'))
import control
import slot_host
import slots

REFUSED = ('ServerFull', 'connect', 'Connection refused', 'peer closed')


def http_json(url, timeout=3):
    with urlopen(url, timeout=timeout) as response:
        return json.loads(response.read(65536))


class LocalHost(slots.SystemHost):
    """The same machine interface as production, backed by local processes and a private database."""

    def __init__(self, work, ports, admin_bin, order=('a', 'b')):
        self.work, self.admin_bin, self.order = Path(work), Path(admin_bin), order
        self.processes = {}
        self.timeline = []
        self.stale_to = None      # a drill: the public route keeps answering with this slot
        slot_host.PORTS.update(ports)
        slot_host.DB = self.work/'db'/'tbg.sqlite3'
        slot_host.BIN = self.work/'bin'
        slot_host.RELEASES = slots.RELEASES = self.work/'releases'
        slots.STATE = self.work/'state'
        slots.JOURNAL = slots.STATE/'slot-transition.json'
        for directory in [slot_host.DB.parent, slot_host.BIN, slots.RELEASES, slots.STATE]:
            directory.mkdir(parents=True, exist_ok=True)

    def log(self, message):
        self.timeline.append((time.time(), message))
        print(f'{time.strftime("%H:%M:%S")} {message}', flush=True)

    def admin(self, *args):
        result = subprocess.run([str(self.admin_bin), '--db', str(slot_host.DB), *map(str, args)],
                                capture_output=True, text=True, timeout=30)
        if args[0] not in {'retire-check'}:
            self.log(f'admin {" ".join(map(str, args[:2]))} -> {result.returncode}')
        return result.returncode, (result.stdout + result.stderr).strip()

    def is_active(self, slot):
        process = self.processes.get(slot)
        return process is not None and process.poll() is None

    def start_slot(self, slot, generation, binary, build):
        target = slots.RELEASES/generation
        if target.exists():
            return 1, f'{target} exists'
        target.mkdir()
        shutil.copy2(binary, target/'tbg-server')
        env = {**os.environ, 'TBG_ADDR': f'127.0.0.1:{slot_host.PORTS[slot]}', 'TBG_GENERATION': generation,
               'TBG_SERVER_BUILD': build, 'TBG_SLOT': slot, 'TBG_WS_ENDPOINT': f'/ws/slot-{slot}', 'TBG_DB': str(slot_host.DB),
               'TBG_MAX_PER_IP': '1000', 'TBG_REGISTER_PER_IP_HOUR': '100000', 'TBG_LOGIN_PER_IP_MIN': '100000',
               'TBG_API_PER_IP_MIN': '100000'}
        log = (self.work/f'slot-{slot}-{generation}.log').open('wb')
        self.processes[slot] = subprocess.Popen([str(target/'tbg-server')], env=env, stdout=log, stderr=log,
                                                cwd=self.work)
        self.log(f'slot {slot} started with generation {generation}')
        for _ in range(120):
            row = {r['slot']: r for r in self.directory()}.get(slot) if slot_host.DB.exists() else None
            if row and row['generation'] == generation and row['state'] == 'ready':
                return 0, 'ready'
            time.sleep(0.5)
        return 1, 'not ready'

    def stop_slot(self, slot):
        process = self.processes.get(slot)
        if process is not None and process.poll() is None:
            process.send_signal(signal.SIGTERM)
            process.wait(timeout=120)
            self.log(f'slot {slot} stopped')

    def retire_slot(self, slot):
        code, text = self.admin('retire-check', slot)
        if code != 0:
            return 3, text
        self.stop_slot(slot)
        return self.admin('retire', slot, '--actor', slots.ACTOR)

    def inspect(self, staged, manifest):
        # The same binary stands in for different sources, so only its own contract, schema and target are real.
        result = subprocess.run([str(Path(staged)/'tbg-server'), '--build-info'], capture_output=True, text=True, timeout=10)
        return {**json.loads(result.stdout), 'source_commit': manifest['source_commit']}

    def public_ws(self, base, slot, contract):
        return super().public_ws(f'http://127.0.0.1:{slot_host.PORTS[slot]}', slot, contract)

    def proxy(self, path):
        """The shared public route: the first slot that answers, like Caddy's `lb_policy first`."""
        for slot in self.order:
            try:
                return http_json(f'http://127.0.0.1:{slot_host.PORTS[slot]}{path}'), slot
            except (OSError, ValueError):
                continue
        return None, None

    def public_admission(self, base, contract):
        if self.stale_to:
            return {'status': 'ok', 'endpoint': f'/ws/slot-{self.stale_to}'}
        return self.proxy(f'/api/admission?contract={contract}&build=rehearsal&outcome=none')[0]

    def public_ok(self, base):
        for slot in self.order:
            try:
                with urlopen(f'http://127.0.0.1:{slot_host.PORTS[slot]}/health', timeout=3) as response:
                    return response.status == 200
            except OSError:
                continue
        return False


class Load:
    """Players: each loop asks admission, connects to the answer and plays one bounded match, again and again."""

    def __init__(self, host, bots_bin, contract, surrender, casual, ranked):
        self.host, self.bots_bin, self.contract, self.surrender = host, Path(bots_bin), contract, surrender
        self.matches, self.probes, self.stop = [], [], threading.Event()
        self.lock = threading.Lock()
        self.paused = threading.Event()
        self.seeds = itertools.count(int(time.time()) % 10**6 * 1000, 7)
        self.threads = [threading.Thread(target=self.player, args=(i, i >= casual), daemon=True) for i in range(casual + ranked)]
        self.threads.append(threading.Thread(target=self.probe, daemon=True))

    def start(self):
        for thread in self.threads:
            thread.start()

    def finish(self):
        self.stop.set()
        for thread in self.threads:
            thread.join(timeout=self.surrender * 3 + 60)

    def record(self, bucket, item):
        with self.lock:
            bucket.append(item)

    def probe(self):
        while not self.stop.is_set():
            started = time.time()
            answer, _ = self.host.proxy(f'/api/admission?contract={self.contract}&build=probe&outcome=none')
            ok = bool(answer) and answer.get('status') == 'ok'
            self.record(self.probes, {'at': started, 'ok': ok, 'ms': (time.time() - started) * 1000,
                                      'endpoint': (answer or {}).get('endpoint'), 'health': self.host.public_ok(None)})
            time.sleep(0.1)

    def player(self, index, ranked):
        while not self.stop.is_set():
            if self.paused.is_set():
                time.sleep(0.2)
                continue
            answer, port_slot = self.host.proxy(f'/api/admission?contract={self.contract}&build=rehearsal&outcome=none&bucket={index}')
            if not answer or answer.get('status') != 'ok':
                self.record(self.matches, {'at': time.time(), 'kind': 'admission', 'ok': False, 'detail': str(answer)})
                time.sleep(0.3)
                continue
            for attempt in (1, 2):
                started = time.time()
                endpoint = answer['endpoint']
                slot = endpoint.rsplit('-', 1)[-1]
                # The bot derives its HTTP auth origin from this URL, which must carry no query.
                url = f'ws://127.0.0.1:{slot_host.PORTS[slot]}{endpoint}' + ('' if ranked else f'?contract={self.contract}')
                # Account names come from the seeds: every run needs its own or the names run out.
                command = [str(self.bots_bin), '--url', url, '--matches', '1', '--surrender-s', str(self.surrender),
                           '--timeout-s', str(self.surrender + 60), '--seed', str(next(self.seeds))] + (['--ranked'] if ranked else [])
                result = subprocess.run(command, capture_output=True, text=True)
                output = (result.stdout + result.stderr)
                item = {'at': started, 'seconds': time.time() - started, 'kind': 'ranked' if ranked else 'casual',
                        'endpoint': endpoint, 'ok': result.returncode == 0, 'attempt': attempt}
                if result.returncode != 0:
                    item['detail'] = ' | '.join(line for line in output.splitlines() if 'failure' in line)[:300]
                self.record(self.matches, item)
                # A client that is refused because its slot is being drained asks admission again (client P66 re-admission).
                refused = result.returncode != 0 and time.time() - started < 5 and any(w in output for w in REFUSED)
                if not refused or attempt == 2:
                    break
                answer, _ = self.host.proxy(f'/api/admission?contract={self.contract}&build=rehearsal&outcome=none&bucket={index}')
                if not answer or answer.get('status') != 'ok':
                    break


def manifest_for(binary, admin_bin, source, build_info):
    digest = lambda path: hashlib.sha256(Path(path).read_bytes()).hexdigest()
    return {'schema': 1, 'source_commit': source, 'contract': build_info['contract'], 'target': build_info['target'],
            'database_schema': build_info['database_schema'], 'files': {'tbg-server': digest(binary), 'tbg-release-admin': digest(admin_bin)}}


def stage(work, name, binary, admin_bin):
    folder = work/'staged'/name
    folder.mkdir(parents=True)
    shutil.copy2(binary, folder/'tbg-server')
    shutil.copy2(admin_bin, folder/'tbg-release-admin')
    return folder


def lone_waiter(host, load, args):
    """One player waits alone in the queue of the active slot; the release has to wait for them.

    Pairs of bots share one queue, so a lone extra bot would shift every later pair by one. The other
    players therefore pause until the lobby is empty and come back when the waiter has left.
    """
    slot = next(r['slot'] for r in host.directory() if r['state'] == 'active')
    load.paused.set()
    for _ in range(120):
        metrics = host.metrics(slot)
        if metrics and metrics['lobby_waiting'] == 0 and metrics['matches_running'] == 0:
            break
        time.sleep(1)
    host.log(f'a lone player queues on slot {slot} for {args.waiter_s} s (the release must wait for them)')
    waiter = subprocess.Popen([str(args.bots_bin), '--url', f'ws://127.0.0.1:{slot_host.PORTS[slot]}/ws/slot-{slot}', '--matches', '1',
                               '--bots', '1', '--timeout-s', str(args.waiter_s)], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

    def release_the_others():
        waiter.wait()
        host.log('the lone player left; the other players come back')
        load.paused.clear()
    threading.Thread(target=release_the_others, daemon=True).start()
    time.sleep(2)
    return waiter


def rollback_drill(host, work, args, info, config):
    """A candidate that starts and promotes but is never routed to: admissions must return to the previous slot."""
    source = hashlib.sha1(b'rehearsal-rollback').hexdigest()
    staged = stage(work, source[:12], args.new_bin, args.admin_bin)
    manifest = manifest_for(args.new_bin, args.admin_bin, source, info)
    before = next(r['slot'] for r in host.directory() if r['state'] == 'active')
    host.stale_to = before
    host.log(f'drill: candidate {source[:12]} promotes but the public route keeps answering slot {before}')
    began, outcome = time.time(), 'not rejected'
    with patch_settle(8):
        for _ in range(60):
            try:
                slots.activate(host, staged, manifest, config) if slots.load_journal() is None else slots.resume(host, config)
            except slots.Rejected as reason:
                host.log(f'rejected: {reason}')
                outcome = 'rolled back'
                host.stale_to = None
            except control.Deferred as reason:
                host.log(f'deferred: {reason}')
                time.sleep(5)
                continue
            if slots.load_journal() is None:
                break
            time.sleep(5)
    host.stale_to = None
    active = next((r['slot'] for r in host.directory() if r['state'] == 'active'), None)
    host.log(f'drill finished: active slot {active} (was {before}), journal {"closed" if slots.load_journal() is None else "open"}')
    return {'number': 'drill', 'seconds': round(time.time() - began, 1), 'outcome': outcome, 'active': active, 'expected': before}


class patch_settle:
    def __init__(self, seconds):
        self.seconds = seconds

    def __enter__(self):
        self.saved, slots.SETTLE_WINDOW = slots.SETTLE_WINDOW, self.seconds

    def __exit__(self, *exc):
        slots.SETTLE_WINDOW = self.saved


def summarize(load, host, swaps):
    matches, probes = load.matches, load.probes
    failed = [m for m in matches if not m['ok']]
    unrecovered = [m for m in failed if m['kind'] == 'admission' or m.get('attempt') == 2 or not any(
        n['ok'] and n['at'] > m['at'] and n.get('attempt') == 2 for n in matches)]
    by_endpoint = {}
    for match in matches:
        if match['ok']:
            by_endpoint[match['endpoint']] = by_endpoint.get(match['endpoint'], 0) + 1
    latency = [p['ms'] for p in probes]
    return {
        'matches_completed': sum(1 for m in matches if m['ok']),
        'matches_by_endpoint': by_endpoint,
        'ranked_completed': sum(1 for m in matches if m['ok'] and m['kind'] == 'ranked'),
        'attempts_failed': len(failed),
        'failures_not_recovered_by_readmission': [m for m in unrecovered if not m['ok']][:20],
        'admission_probes': len(probes),
        'admission_failures': sum(1 for p in probes if not p['ok']),
        'health_failures': sum(1 for p in probes if not p['health']),
        'admission_ms_p50_max': [round(statistics.median(latency), 1), round(max(latency), 1)] if latency else None,
        'swaps': swaps,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    for name in ['old-bin', 'new-bin', 'admin-bin', 'bots-bin', 'out']:
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--ports', nargs=2, type=int, default=[18080, 18082])
    parser.add_argument('--swaps', type=int, default=2)
    parser.add_argument('--casual', type=int, default=3)
    parser.add_argument('--ranked', type=int, default=1)
    parser.add_argument('--surrender-s', type=int, default=20)
    parser.add_argument('--steady-s', type=int, default=25)
    parser.add_argument('--rollback-drill', action='store_true',
                        help='after the releases, one candidate whose public admission never follows it must be rolled back')
    parser.add_argument('--waiter-s', type=int, default=30,
                        help='before the last release a lone player waits this long in the queue (0: none)')
    args = parser.parse_args()
    args.out.mkdir(parents=True, exist_ok=True)
    work = Path(tempfile.mkdtemp(prefix='rehearsal-', dir=args.out))
    host = LocalHost(work, {'a': args.ports[0], 'b': args.ports[1]}, args.admin_bin)
    info = json.loads(subprocess.check_output([str(args.new_bin), '--build-info'], text=True))
    contract = info['contract']

    host.log('bootstrap: the live generation runs in slot a')
    code, text = host.start_slot('a', 'live-generation', args.old_bin, 'live')
    assert code == 0, text
    assert host.admin('promote', 'a', '--actor', 'rehearsal')[0] == 0
    assert host.admin('ranked', 'a', '--actor', 'rehearsal')[0] == 0
    time.sleep(2)

    load = Load(host, args.bots_bin, contract, args.surrender_s, args.casual, args.ranked)
    load.start()
    swaps = []
    time.sleep(args.steady_s)
    config = {'public_api': 'http://127.0.0.1'}
    for number in range(1, args.swaps + 1):
        source = hashlib.sha1(f'rehearsal-{number}'.encode()).hexdigest()
        staged = stage(work, source[:12], args.new_bin, args.admin_bin)
        manifest = manifest_for(args.new_bin, args.admin_bin, source, info)
        waiter = None
        if args.waiter_s and number == args.swaps:
            waiter = lone_waiter(host, load, args)
        began = time.time()
        host.log(f'release {number}: coordinator starts (candidate {source[:12]})')
        # The timer runs the same controller again and again: a deferral only means "later".
        for _ in range(60):
            try:
                if slots.load_journal() is None:
                    slots.activate(host, staged, manifest, config)
                else:
                    slots.resume(host, config)
                break
            except control.Deferred as reason:
                host.log(f'deferred: {reason}')
                time.sleep(5)
        outcome = 'done' if slots.load_journal() is None else 'unfinished'
        if waiter is not None:
            waiter.wait(timeout=60)
        swaps.append({'number': number, 'seconds': round(time.time() - began, 1), 'outcome': outcome,
                      'active': next((r['slot'] for r in host.directory() if r['state'] == 'active'), None)})
        host.log(f'release {number}: finished in {swaps[-1]["seconds"]} s; active slot {swaps[-1]["active"]}')
        time.sleep(args.steady_s)
    if args.rollback_drill:
        swaps.append(rollback_drill(host, work, args, info, config))
        time.sleep(args.steady_s)
    load.finish()
    for slot in list(host.processes):
        host.stop_slot(slot)
    report = summarize(load, host, swaps)
    report['timeline'] = [f'{time.strftime("%H:%M:%S", time.localtime(t))} {m}' for t, m in host.timeline]
    (work/'report.json').write_text(json.dumps(report, indent=1))
    print(json.dumps({k: v for k, v in report.items() if k != 'timeline'}, indent=1))
    print('report:', work/'report.json')
    return 0 if not report['failures_not_recovered_by_readmission'] and not report['admission_failures'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
