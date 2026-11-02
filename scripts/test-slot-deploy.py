#!/usr/bin/env python3
"""Release-slot coordinator on a host that obeys the real admin rules: promotion, gates, rollback, drain and crash resume."""
import ast
import hashlib
import http.server
import json
import re
import sqlite3
import subprocess
import sys
import tempfile
import threading
import unittest
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT/'deploy/auto'))
import activate
import control
import pull
import slot_host
import slots
import status

SOURCE = 'b'*40
OLD = 'a'*40
CONTRACT = 'w31-r6-097385fee9a10b0f'
SERVER, ADMIN = b'\x7fELFserver', b'\x7fELFadmin'
CONFIG = {'public_api': 'https://api.example.com', 'public_health': 'https://api.example.com/health'}


class Crash(BaseException):
    """The controller process was killed: nothing after this point runs, cleanup included."""


def manifest(**changes):
    value = {'schema': 1, 'source_commit': SOURCE, 'contract': CONTRACT, 'target': 'x86_64-unknown-linux-gnu',
             'database_schema': 10, 'files': {'tbg-server': hashlib.sha256(SERVER).hexdigest(),
                                             'tbg-release-admin': hashlib.sha256(ADMIN).hexdigest()}}
    value.update(changes)
    return value


def row(slot, state, generation, **changes):
    value = {'slot': slot, 'generation': generation, 'contract': CONTRACT, 'state': state, 'ranked_authority': 0,
             'matches_running': 0, 'results_pending': 0, 'ranked_inflight': 0, 'heartbeat_unix': 1000.0, 'version': 1}
    value.update(changes)
    return value


def quiet():
    return {'connections': 0, 'spectators': 0, 'queue_len': 0, 'lobby_waiting': 0}


class FakeHost(slot_host.SystemHost):
    """Two slot processes, the shared directory with the real admin rules (store/release_admin.rs) and a public proxy."""

    def __init__(self):
        self.t = 1000.0
        self.rows = {'a': row('a', 'active', 'old', ranked_authority=1, version=3)}
        self.running = {'a'}
        self.idle = {'a': quiet()}
        self.events = []             # (time, callable), applied as the clock moves
        self.fail = set()            # failure injection keys
        self.calls = 0               # state-changing calls so far (the crash points)
        self.crash_at = None         # (call number, 'before' | 'after')
        self.violations = []         # a slot stopped while it held work or admitted players
        self.log = []
        self.busy = {}               # command -> database-busy answers still to give
        self.lost_reply = set()      # commands that commit and then fail to answer (a timeout)
        self.ranked_busy = 0
        self.public_slot = None      # the public route keeps answering this slot
        self.schema_version = 10
        self.identity = None         # what --build-info says, when it should lie
        self.inspect_error = None    # raised by --build-info (a hang, a crash)

    # --- time and bookkeeping -------------------------------------------------------------------------
    def now(self):
        return self.t

    def sleep(self, seconds):
        self.t += seconds
        for event in [e for e in self.events if e[0] <= self.t]:
            self.events.remove(event)
            event[1]()

    def step(self, name, effect):
        self.calls += 1
        if self.crash_at == (self.calls, 'before'):
            raise Crash(name)
        result = effect()
        if self.crash_at == (self.calls, 'after'):
            raise Crash(name)
        self.log.append(name)
        return result

    def active_slot(self):
        return next((s for s, r in self.rows.items() if r['state'] == 'active'), None)

    def kill(self, slot):
        self.running.discard(slot)

    def load(self, slot, **work):
        self.rows[slot].update({k: v for k, v in work.items() if k in {'matches_running', 'results_pending', 'ranked_inflight'}})
        self.idle[slot].update({k: v for k, v in work.items() if k in self.idle[slot]})

    def touch(self, slot):
        self.rows[slot]['version'] += 1

    # --- reads ------------------------------------------------------------------------------------------
    def directory(self):
        for slot in self.running:
            self.rows[slot]['heartbeat_unix'] = self.t
        return [dict(r, fresh=self.t - r['heartbeat_unix'] <= slot_host.FRESH_SECONDS) for _, r in sorted(self.rows.items())]

    def schema(self):
        return self.schema_version

    def status(self, slot):
        if slot not in self.running or slot not in self.rows:
            return None
        r = self.rows[slot]
        admits = r['state'] in {'active', 'canary'}
        return {'slot': slot, 'generation': r['generation'], 'contract': r['contract'], 'admitting': admits,
                'ranked': admits and bool(r['ranked_authority']), 'shutdown': False}

    def metrics(self, slot):
        if slot not in self.running:
            return None
        r = self.rows[slot]
        return {'connections': self.idle[slot]['connections'] + 2 * r['matches_running'], 'matches_running': r['matches_running'],
                'spectators': self.idle[slot]['spectators'], 'queue_len': self.idle[slot]['queue_len'],
                'lobby_waiting': self.idle[slot]['lobby_waiting']}

    def store_ok(self, slot):
        return 'store' not in self.fail

    def public_admission(self, base, contract):
        slot = self.public_slot or self.active_slot()
        return {'status': 'ok', 'endpoint': f'/ws/slot-{slot}'} if slot and 'admission' not in self.fail else None

    def public_ok(self, base):
        return 'health' not in self.fail

    def public_ws(self, base, slot, contract):
        return slot in self.running and 'ws' not in self.fail

    def is_active(self, slot):
        return slot in self.running

    def free_disk(self):
        return 10**10

    def free_memory_mb(self):
        return 4096

    def inspect(self, staged, manifest):
        if self.inspect_error:
            raise self.inspect_error
        return self.identity or {key: manifest[key] for key in ['source_commit', 'contract', 'database_schema', 'target']}

    # --- the admin tool, with the rules of release_admin.rs -----------------------------------------------
    def admin(self, *args):
        command, slot = args[0], args[1] if len(args) > 1 else ''
        if command not in {'promote', 'rollback', 'ranked', 'drain', 'retire'}:
            return self.admin_effect(command, slot, args)
        if self.busy.get(command):
            self.busy[command] -= 1
            return 1, 'Db("database is locked")'
        result = self.step(f'admin {command} {slot}', lambda: self.admin_effect(command, slot, args))
        return (1, 'timed out waiting for the answer') if command in self.lost_reply else result

    def admin_effect(self, command, slot, args):
        rows, now = self.rows, self.t
        if command in self.fail:
            return 1, f'WrongState("{command} refused")'
        if slot not in rows:
            return 1, 'UnknownSlot'
        target = rows[slot]
        fresh = now - target['heartbeat_unix'] <= slot_host.FRESH_SECONDS
        if '--expect' in args and int(args[args.index('--expect') + 1]) != target['version']:
            return 1, 'Stale'
        if command in {'promote', 'rollback', 'ranked'} and not fresh:
            return 1, 'NotLive'                  # `drain` and `retire` never ask whether the process is alive
        if command in {'promote', 'rollback'}:
            allowed = {'ready', 'canary'} | ({'draining'} if command == 'rollback' else set())
            if target['state'] not in allowed:
                return 1, f'WrongState("{target["state"]}")'
            for other in rows.values():
                if other['slot'] != slot and other['state'] in {'active', 'canary'}:
                    other['state'] = 'draining'
                    self.touch(other['slot'])
            target['state'] = 'active'
            self.touch(slot)
        elif command == 'drain':
            if target['state'] in {'retired', 'failed', 'draining'}:
                return 1, f'WrongState("{target["state"]}")'
            target['state'] = 'draining'
            self.touch(slot)
        elif command == 'ranked':
            holder = next((r for r in rows.values() if r['ranked_authority'] and r['slot'] != slot), None)
            if self.ranked_busy:
                self.ranked_busy -= 1
                return 1, 'Busy("ranked results in flight")'
            if holder and holder['ranked_inflight'] > 0:
                return 1, 'Busy("ranked results in flight")'
            for r in rows.values():
                r['ranked_authority'] = int(r['slot'] == slot)
        elif command in {'retire-check', 'retire'}:
            why = [text for ok, text in [(target['state'] in {'draining', 'ready', 'retired'}, f'state is {target["state"]}'),
                                         (not target['matches_running'], 'matches running'),
                                         (not target['results_pending'], 'results pending'),
                                         (not target['ranked_inflight'], 'ranked in flight'),
                                         (not target['ranked_authority'], 'holds ranked authority')] if not ok]
            if command == 'retire-check':
                return (3, 'not yet — ' + '; '.join(why)) if why else (0, 'safe')
            if why:
                return 1, f'Busy("{", ".join(why)}")'
            target['state'] = 'retired'
            self.touch(slot)
        return 0, ''

    # --- systemd and the slot script -----------------------------------------------------------------------
    def start_slot(self, slot, generation, binary, build):
        def effect():
            target = slots.RELEASES/generation
            target.mkdir()
            (target/'tbg-server').write_bytes(Path(binary).read_bytes())
            version = self.rows[slot]['version'] + 1 if slot in self.rows else 1
            self.rows[slot] = row(slot, 'warming', generation, version=version, heartbeat_unix=self.t)
            self.idle[slot] = quiet()
            if 'start' in self.fail:
                return 1, 'not ready'              # registered, then the process died while warming
            self.running.add(slot)
            self.rows[slot]['state'] = 'ready'
            self.touch(slot)
            return 0, 'ready'
        return self.step(f'start {slot}', effect)

    def stop(self, slot):
        held = self.rows.get(slot)
        if held and (held['matches_running'] or held['results_pending'] or held['state'] in {'active', 'canary'}):
            self.violations.append(f'stopped slot {slot} in state {held["state"]} with {held["matches_running"]} matches')
        others = [r for name, r in self.rows.items() if name != slot and r['state'] == 'active' and name in self.running]
        if held and held['state'] != 'active' and slot in self.running and not others:
            self.violations.append(f'stopped slot {slot} while no other slot admitted players')
        if 'stop' in self.fail:
            raise OSError('systemd refused')
        if slot in self.running and held is not None and not held['matches_running'] and not held['results_pending']:
            held['state'], held['ranked_authority'] = 'retired', 0       # the server's own retire_self on a drained exit
            self.touch(slot)
        self.running.discard(slot)

    def stop_slot(self, slot):
        self.step(f'stop {slot}', lambda: self.stop(slot))

    def retire_slot(self, slot):
        code, text = self.admin('retire-check', slot)
        if code:
            return 3, text
        self.stop_slot(slot)
        return self.admin('retire', slot, '--actor', slots.ACTOR)

    def keep_files(self, staged, generation, manifest):
        return self.step('keep files', lambda: super(FakeHost, self).keep_files(staged, generation, manifest))

    def adopt_admin(self, release):
        self.step('adopt admin', lambda: self.log.append(f'adopted {release.name}'))


class SlotCase(unittest.TestCase):
    def setUp(self):
        folder = tempfile.TemporaryDirectory()
        self.addCleanup(folder.cleanup)
        root = Path(folder.name)
        self.state, self.releases, self.staged = root/'state', root/'releases', root/'staged'
        for directory in [self.state, self.releases, self.staged]:
            directory.mkdir()
        (self.staged/'tbg-server').write_bytes(SERVER)
        (self.staged/'tbg-release-admin').write_bytes(ADMIN)
        for module in [slots, slot_host]:
            patcher = patch.object(module, 'RELEASES', self.releases)
            patcher.start()
            self.addCleanup(patcher.stop)
        patcher = patch.multiple(slots, STATE=self.state, JOURNAL=self.state/'slot-transition.json')
        patcher.start()
        self.addCleanup(patcher.stop)
        self.host = FakeHost()

    def go(self, **options):
        slots.activate(self.host, self.staged, manifest(), CONFIG, **options)

    def journal(self):
        return slots.load_journal()

    def names(self):
        return self.host.log

    def receipt(self):
        path = self.state/'current.json'
        return json.loads(path.read_text()) if path.exists() else None

    def finish_old_matches(self, slot='a', after=40):
        self.host.load(slot, matches_running=2, results_pending=2)
        self.host.events.append((self.host.t + after, lambda: self.host.load(slot, matches_running=0, results_pending=0)))


class Promotion(SlotCase):
    def test_idle_host_promotes_the_idle_slot_then_retires_the_previous_one(self):
        self.go()
        self.assertEqual(self.host.active_slot(), 'b')
        self.assertEqual(self.host.rows['a']['state'], 'retired')
        self.assertEqual(self.host.running, {'b'})
        self.assertTrue(self.host.rows['b']['ranked_authority'])
        order = [self.names().index(name) for name in ['start b', 'admin promote b', 'admin ranked b', 'admin retire a', 'adopt admin']]
        self.assertEqual(order, sorted(order))
        self.assertEqual((self.receipt()['source_commit'], self.receipt()['slot'], self.receipt()['generation']), (SOURCE, 'b', SOURCE[:12]))
        self.assertIsNone(self.journal())
        self.assertEqual((self.releases/SOURCE[:12]/'tbg-release-admin').read_bytes(), ADMIN)
        self.assertEqual(self.host.violations, [])

    def test_the_previous_slot_keeps_its_matches_and_retires_only_after_they_end(self):
        self.finish_old_matches()
        with self.assertRaises(control.Deferred):
            self.go()
        self.assertEqual(self.host.active_slot(), 'b')
        self.assertIn('a', self.host.running)
        self.assertEqual(self.journal()['phase'], 'promoted')
        self.assertEqual(self.receipt()['source_commit'], SOURCE, 'the candidate already serves: the floor moves forward')
        self.host.sleep(60)
        slots.resume(self.host, CONFIG)
        self.assertEqual(self.host.running, {'b'})
        self.assertEqual((self.host.violations, self.journal()), ([], None))

    def test_idle_connections_get_a_short_grace_then_reconnect_through_admission(self):
        self.host.load('a', connections=3)
        with self.assertRaises(control.Deferred):
            self.go()
        self.assertIn('a', self.host.running)
        self.host.sleep(slots.IDLE_GRACE + 1)
        slots.resume(self.host, CONFIG)
        self.assertEqual(self.host.running, {'b'})

    def test_ranked_authority_waits_for_results_in_flight_and_is_retried(self):
        self.host.ranked_busy = 3
        self.go()
        self.assertTrue(self.host.rows['b']['ranked_authority'])

    def test_a_busy_database_is_retried_instead_of_rejecting_the_candidate(self):
        self.host.busy = {'promote': 2, 'retire': 1}
        self.go()
        self.assertEqual(self.host.active_slot(), 'b')

    def test_a_promotion_that_committed_but_never_answered_is_taken_as_made(self):
        self.host.lost_reply = {'promote'}
        self.go()
        self.assertEqual((self.host.active_slot(), self.host.running), ('b', {'b'}))

    def test_a_promotion_the_directory_refuses_stops_the_unpromoted_slot_and_rejects(self):
        self.host.fail.add('promote')
        with self.assertRaises(slots.Rejected):
            self.go()
        self.assertEqual((self.host.active_slot(), self.host.running), ('a', {'a'}))
        self.assertIsNone(self.journal())


class QuietGate(SlotCase):
    def test_waiting_players_defer_before_anything_is_started(self):
        self.host.load('a', lobby_waiting=2)
        with self.assertRaises(control.Deferred):
            self.go()
        self.assertEqual(self.names(), [])
        self.assertIsNone(self.journal())

    def test_a_lobby_that_empties_within_the_window_proceeds(self):
        self.host.load('a', lobby_waiting=1)
        self.host.events.append((self.host.t + 20, lambda: self.host.load('a', lobby_waiting=0)))
        self.go()
        self.assertEqual(self.host.active_slot(), 'b')

    def test_a_player_who_queues_while_the_candidate_warms_up_cancels_the_switch(self):
        original = self.host.start_slot

        def start(*args):
            result = original(*args)
            self.host.load('a', lobby_waiting=1)
            return result
        self.host.start_slot = start
        with self.assertRaises(control.Deferred):
            self.go()
        self.assertEqual((self.host.active_slot(), self.host.running), ('a', {'a'}))
        self.assertEqual(self.host.rows['b']['state'], 'retired')
        self.assertIsNone(self.journal())

    def test_an_operator_can_promote_without_waiting_for_the_lobby(self):
        self.host.load('a', lobby_waiting=3)
        self.go(ignore_lobby=True)
        self.assertEqual(self.host.active_slot(), 'b')

    def test_a_generation_without_the_gauge_counts_sockets_that_are_not_seats(self):
        for metrics, waiting in [({'connections': 2, 'matches_running': 1, 'spectators': 0, 'queue_len': 0}, 0),
                                 ({'connections': 3, 'matches_running': 1, 'spectators': 0, 'queue_len': 0}, 1),
                                 ({'connections': 4, 'matches_running': 1, 'spectators': 2, 'queue_len': 1}, 1)]:
            self.assertEqual(slots.lobby_waiting(metrics), waiting)

    def test_the_gate_asks_before_a_download_and_never_changes_anything(self):
        slots.gate(self.host)
        self.host.load('a', lobby_waiting=1)
        with self.assertRaises(control.Deferred):
            slots.gate(self.host)
        slots.gate(self.host, ignore_lobby=True)
        self.assertEqual(self.names(), [])


class Refusals(SlotCase):
    def test_contract_and_schema_changes_are_rejected_for_good_before_anything_starts(self):
        with self.assertRaisesRegex(slots.Rejected, 'contract'):
            slots.activate(self.host, self.staged, manifest(contract='w32-r6-' + '0'*16), CONFIG)
        self.host.schema_version = 11
        with self.assertRaisesRegex(slots.Rejected, 'database'):
            self.go()
        self.assertEqual(self.names(), [])

    def test_an_unfinished_previous_release_defers_and_a_stray_process_needs_review(self):
        self.host.rows['b'] = row('b', 'draining', 'older')
        self.host.running.add('b')
        with self.assertRaises(control.Deferred):
            self.go()
        self.host.rows['b']['state'] = 'retired'
        with self.assertRaisesRegex(ValueError, 'operator review'):
            self.go()
        self.assertEqual(self.names(), [])

    def test_a_candidate_that_describes_itself_differently_never_starts(self):
        self.host.identity = {**manifest(), 'contract': 'w31-r6-' + '1'*16}
        with self.assertRaisesRegex(slots.Rejected, 'identity'):
            self.go()
        self.assertEqual(self.names(), [])

    def test_a_candidate_that_never_gets_ready_is_torn_down_even_when_it_died_while_warming(self):
        self.host.fail.add('start')
        with self.assertRaises(slots.Rejected):
            self.go()
        self.assertEqual((self.host.active_slot(), self.host.rows['b']['state']), ('a', 'retired'))
        self.assertIsNone(self.journal())
        self.host.fail.clear()
        self.go()                               # a later release is not blocked by the dead row
        self.assertEqual(self.host.active_slot(), 'b')

    def test_a_dead_warming_row_left_by_a_crashed_attempt_is_reclaimed_by_the_next_release(self):
        self.host.rows['b'] = row('b', 'warming', 'died', heartbeat_unix=0.0)
        self.host.idle['b'] = quiet()
        self.go()
        self.assertEqual(self.host.active_slot(), 'b')
        self.assertEqual(self.host.violations, [])

    def test_failed_cleanup_leaves_a_journal_that_an_operator_can_clear_by_fixing_the_cause(self):
        self.host.fail.update({'start', 'stop'})
        with self.assertRaises(slots.OperatorRequired):
            self.go()
        self.assertEqual(self.journal()['phase'], 'cleanup_failed')
        with self.assertRaises(OSError):
            slots.resume(self.host, CONFIG)
        self.host.fail.clear()
        with self.assertRaises(control.Deferred):
            slots.resume(self.host, CONFIG)         # the cause is gone: cleanup completes and the release starts over
        self.assertIsNone(self.journal())

    def test_a_slot_that_holds_another_generation_is_never_stopped_for_a_stale_journal(self):
        slots.save({'source_commit': SOURCE, 'generation': SOURCE[:12], 'from': 'a', 'to': 'b', 'phase': 'ready', 'manifest': manifest()})
        self.host.rows['b'] = row('b', 'ready', 'someone-elses')
        self.host.running.add('b')
        with self.assertRaises(slots.OperatorRequired):
            slots.resume(self.host, CONFIG)
        self.assertIn('b', self.host.running)


class Hardening(SlotCase):
    """Second review round: transient errors retry, finished states are recognised, and nothing waits silently for ever."""

    def test_the_installer_ships_every_module_the_controllers_import(self):
        folder = ROOT/'deploy/auto'
        shipped = set(re.findall(r'"\$HERE/([a-z_]+)\.py"', (folder/'install.sh').read_text()))
        local = {path.stem for path in folder.glob('*.py')}
        self.assertEqual(local - shipped, set(), 'a controller module the installer does not copy')
        for name in shipped:
            for node in ast.walk(ast.parse((folder/f'{name}.py').read_text())):
                modules = [alias.name for alias in node.names] if isinstance(node, ast.Import) else [node.module] if isinstance(node, ast.ImportFrom) and node.module else []
                for module in modules:
                    if module.split('.')[0] in local:
                        self.assertIn(module.split('.')[0], shipped, f'{name}.py imports {module}')

    def test_the_rejected_slot_that_exited_cleanly_is_retired_instead_of_holding_every_release(self):
        self.host.fail.add('ws')
        with patch.object(slots, 'SETTLE_WINDOW', 6), self.assertRaises(slots.Rejected):
            self.go()
        self.host.kill('b')                                              # the candidate died after the rollback
        self.host.sleep(slot_host.FRESH_SECONDS + 30)
        slots.resume(self.host, CONFIG)
        self.assertEqual((self.host.rows['b']['state'], self.journal(), self.host.violations), ('retired', None, []))

    def test_a_manual_rollback_inside_the_verification_window_is_not_a_failure_to_roll_back(self):
        original = self.host.admin

        def admin(*args):
            result = original(*args)
            if args[0] == 'ranked' and args[1] == 'b':
                self.host.events.append((self.host.t + 3, lambda: self.host.admin_effect('rollback', 'a', ('rollback', 'a'))))
            return result
        self.host.admin = admin
        with patch.object(slots, 'SETTLE_WINDOW', 12), self.assertRaises(slots.Rejected):
            self.go()
        self.assertEqual((self.host.active_slot(), self.journal()['phase']), ('a', 'rolled_back'))

    def test_a_busy_database_during_cleanup_is_retried_on_the_next_run_and_never_holds(self):
        self.host.fail.add('start')
        self.host.busy = {'drain': 99, 'retire': 99}
        with self.assertRaises(slots.Busy):
            self.go()
        self.assertEqual(self.journal()['phase'], 'cleanup_failed')
        self.host.busy, self.host.fail = {}, set()
        with self.assertRaises(control.Deferred):
            slots.resume(self.host, CONFIG)
        self.assertEqual((self.journal(), self.host.rows['b']['state']), (None, 'retired'))

    def test_a_candidate_whose_build_info_hangs_is_rejected_once(self):
        self.host.inspect_error = subprocess.TimeoutExpired('build-info', 10)
        with self.assertRaises(slots.Rejected):
            self.go()
        self.assertEqual(self.names(), [])

    def test_a_host_that_never_ran_a_slot_is_not_a_slot_host_but_a_locked_database_defers(self):
        with patch.object(self.host, 'directory', side_effect=sqlite3.OperationalError('no such table: release_slots')):
            self.assertFalse(slots.in_slot_mode(self.host))
        with patch.object(self.host, 'directory', side_effect=sqlite3.OperationalError('database is locked')), \
             self.assertRaises(control.Deferred):
            slots.in_slot_mode(self.host)

    def test_a_drain_that_outlives_a_day_stops_being_a_quiet_deferral(self):
        self.finish_old_matches(after=10**9)
        with self.assertRaises(control.Deferred):
            self.go()
        self.host.sleep(slots.MAX_DRAIN + 60)
        with self.assertRaisesRegex(slots.OperatorRequired, 'stuck'):
            slots.resume(self.host, CONFIG)
        self.assertEqual(self.host.violations, [])

    def test_a_slot_is_never_stopped_when_it_is_the_last_one_admitting(self):
        self.host.stop('a')
        self.assertTrue(self.host.violations)


class Rollback(SlotCase):
    def test_a_candidate_the_public_proxy_does_not_route_returns_admissions_to_the_old_slot(self):
        self.host.public_slot = 'a'
        with patch.object(slots, 'SETTLE_WINDOW', 6), self.assertRaises(slots.Rejected):
            self.go()
        self.assertEqual(self.host.active_slot(), 'a')
        self.assertTrue(self.host.rows['a']['ranked_authority'])
        self.assertIsNone(self.receipt())
        self.assertEqual(self.journal()['phase'], 'rolled_back')
        self.host.public_slot = None
        slots.resume(self.host, CONFIG)
        self.assertEqual((self.host.running, self.journal()), ({'a'}, None))

    def test_a_candidate_whose_websocket_route_is_missing_is_rolled_back(self):
        self.host.fail.add('ws')
        with patch.object(slots, 'SETTLE_WINDOW', 6), self.assertRaises(slots.Rejected):
            self.go()
        self.assertEqual(self.host.active_slot(), 'a')

    def test_the_rejected_slot_keeps_its_matches_until_they_finish(self):
        original = self.host.admin

        def admin(*args):
            result = original(*args)
            if args[0] == 'promote':
                self.host.load('b', matches_running=1)
                self.host.public_slot = 'a'
            return result
        self.host.admin = admin
        with patch.object(slots, 'SETTLE_WINDOW', 4), self.assertRaises(slots.Rejected):
            self.go()
        with self.assertRaises(control.Deferred):
            slots.resume(self.host, CONFIG)
        self.assertIn('b', self.host.running)
        self.host.load('b', matches_running=0)
        slots.resume(self.host, CONFIG)
        self.assertEqual((self.host.running, self.host.violations), ({'a'}, []))

    def test_the_old_slot_is_not_retired_while_its_successor_stopped_admitting(self):
        self.finish_old_matches(after=10**6)            # the old slot still drains a match
        with self.assertRaises(control.Deferred):
            self.go()
        self.host.kill('b')                             # the new generation dies and stays dead
        self.host.load('a', matches_running=0, results_pending=0)
        with patch.object(slots, 'KEEPER_WINDOW', 6), self.assertRaises(slots.Rejected):
            slots.resume(self.host, CONFIG)
        self.assertEqual((self.host.active_slot(), 'a' in self.host.running), ('a', True))
        self.assertEqual(self.host.violations, [])
        self.assertIsNone(self.receipt(), 'the rejected candidate no longer counts as the live release')

    def test_a_manual_rollback_during_the_transition_is_adopted_not_fought(self):
        self.finish_old_matches(after=10**6)
        with self.assertRaises(control.Deferred):
            self.go()
        self.host.admin('rollback', 'a', '--actor', 'operator')          # the README's manual rollback
        self.host.load('a', matches_running=0, results_pending=0)
        with self.assertRaises(slots.Rejected):
            slots.resume(self.host, CONFIG)                              # latched once
        self.assertEqual((self.host.active_slot(), self.journal()['phase']), ('a', 'rolled_back'))
        self.assertIsNone(self.receipt())
        slots.resume(self.host, CONFIG)                                  # the rejected slot had no matches: retired
        self.assertEqual((self.host.running, self.journal()), ({'a'}, None))

    def test_a_promotion_that_never_committed_starts_over_instead_of_rejecting_the_candidate(self):
        original = self.host.admin

        def admin(*args):
            if args[0] == 'promote':
                raise Crash('killed before the promotion committed')
            return original(*args)
        self.host.admin = admin
        with self.assertRaises(Crash):
            self.go()
        self.assertEqual(self.journal()['phase'], 'promoting')
        self.host.admin = original
        with self.assertRaises(control.Deferred):
            slots.resume(self.host, CONFIG)
        self.assertEqual((self.host.active_slot(), self.host.running, self.journal()), ('a', {'a'}, None))

    def test_a_previous_slot_that_died_with_work_waits_for_systemd_then_needs_a_person(self):
        self.finish_old_matches(after=10**6)
        with self.assertRaises(control.Deferred):
            self.go()
        self.host.kill('a')                                              # the process is gone, its counters are stale
        self.host.sleep(slot_host.FRESH_SECONDS + 5)
        with self.assertRaisesRegex(control.Deferred, 'not answering'):
            slots.resume(self.host, CONFIG)
        self.host.sleep(700)
        with self.assertRaises(slots.OperatorRequired):
            slots.resume(self.host, CONFIG)
        self.assertEqual(self.host.violations, [])

    def test_no_slot_admitting_is_a_state_for_a_person(self):
        slots.save({'source_commit': SOURCE, 'generation': SOURCE[:12], 'from': 'a', 'to': 'b', 'phase': 'promoted', 'manifest': manifest()})
        self.host.rows['a']['state'] = 'draining'
        with self.assertRaises(slots.OperatorRequired):
            slots.resume(self.host, CONFIG)


class CrashResume(SlotCase):
    """The controller is killed before or after every state-changing step, then run again the way the timer would."""

    VARIANTS = {
        'happy path': lambda host: None,
        'verification fails': lambda host: host.fail.add('ws'),
        'candidate never ready': lambda host: host.fail.add('start'),
        'promotion refused': lambda host: host.fail.add('promote'),
        'promotion commits, reply lost': lambda host: host.lost_reply.add('promote'),
        'successor dies while the old slot drains': lambda host: host.events.append((host.t + 20, lambda: host.kill('b'))),
    }

    def converge(self, host):
        """The timer runs the controller again and again; a kill only means the next run starts from the journal."""
        rejected = False
        for _ in range(80):
            try:
                if slots.load_journal() is not None:
                    slots.resume(host, CONFIG)
                elif not rejected and host.active_slot() == 'a' and 'adopt admin' not in host.log:
                    slots.activate(host, self.staged, manifest(), CONFIG)
                else:
                    return
            except Crash:
                host.crash_at = None
            except control.Deferred:
                host.sleep(60)
            except slots.Rejected:
                rejected = True
                host.sleep(60)
        self.fail('the transition never converged')

    def fresh_host(self, variant):
        for leftover in self.releases.iterdir():
            for child in leftover.iterdir():
                child.unlink()
            leftover.rmdir()
        for stale in self.state.glob('*.json'):
            stale.unlink()
        host = self.host = FakeHost()
        self.finish_old_matches(after=30)
        self.VARIANTS[variant](host)
        return host

    def test_every_crash_point_of_every_outcome_ends_with_one_admitting_slot_and_nothing_stopped_under_load(self):
        with patch.object(slots, 'SETTLE_WINDOW', 8), patch.object(slots, 'KEEPER_WINDOW', 8):
            for variant in self.VARIANTS:
                probe = self.fresh_host(variant)
                self.converge(probe)
                total = probe.calls
                self.assertGreaterEqual(total, 2, variant)
                for point in range(1, total + 2):
                    for when in ['before', 'after']:
                        with self.subTest(variant=variant, point=point, when=when):
                            host = self.fresh_host(variant)
                            host.crash_at = (point, when)
                            self.converge(host)
                            active = [r for r in host.directory() if r['state'] == 'active' and r['fresh']]
                            self.assertEqual(len(active), 1, host.log)
                            self.assertEqual(host.violations, [], host.log)
                            self.assertIsNone(slots.load_journal())
                            self.assertEqual(host.running, {active[0]['slot']}, host.log)
                            receipt = self.receipt()
                            if receipt is not None:
                                self.assertEqual(receipt['generation'], active[0]['generation'])
                            elif active[0]['generation'] != 'old':
                                self.fail(f'the candidate serves without a receipt: {host.log}')


class Controller(SlotCase):
    """pull.py in slot mode: gates before downloads, permanent refusals latch once, outcomes hold only when a person is needed."""

    def run_pull(self, apply=True, **patches):
        calls = {'fetch': 0}

        def fetch(url, limit):
            calls['fetch'] += 1
            return b'package'
        base = f'https://github.com/{pull.REPOSITORY}/releases/download/server-{SOURCE}/'
        release = {'tag_name': 'server-' + SOURCE, 'assets': [
            {'name': 'tbg-backend-' + SOURCE + '.tar.gz', 'browser_download_url': base + 'tbg-backend-' + SOURCE + '.tar.gz'},
            {'name': 'provenance.jsonl', 'browser_download_url': base + 'provenance.jsonl'}]}
        options = dict(candidate=lambda *a: release, api=lambda path: {'status': 'ahead'}, fetch=fetch,
                       verify=lambda *a, **k: None, unpack=lambda *a: manifest())
        options.update(patches)
        with patch.multiple(pull, STATE=self.state, **options), patch.object(pull.slots, 'SystemHost', return_value=self.host), \
             patch.object(control, 'HOLD', self.state/'hold.json'):
            pull.run({'minimum_source': OLD, 'public_health': CONFIG['public_health']}, apply, False)
        return calls

    def checked(self):
        path = self.state/'checked.json'
        return json.loads(path.read_text())['outcome'] if path.exists() else None

    def test_a_busy_lobby_defers_before_anything_is_downloaded(self):
        self.host.load('a', lobby_waiting=1)
        downloads = []
        with self.assertRaises(control.Deferred):
            self.run_pull(fetch=lambda url, limit: downloads.append(url))
        self.assertEqual((downloads, self.checked(), self.names()), ([], None, []))

    def test_a_contract_change_is_latched_once_and_not_downloaded_again(self):
        changed = lambda *a: manifest(contract='w32-r6-' + '0'*16)
        with self.assertRaises(slots.Rejected):
            self.run_pull(unpack=changed)
        self.assertEqual(self.checked(), 'rejected')
        calls = self.run_pull(unpack=changed)
        self.assertEqual(calls['fetch'], 0, 'a latched candidate is not fetched again')

    def test_a_dry_run_reports_the_slot_preflight_and_changes_nothing(self):
        calls = self.run_pull(apply=False)
        self.assertEqual((calls['fetch'], self.names(), self.checked()), (2, [], None))

    def test_activation_outcomes_are_latched_or_held_by_what_they_need(self):
        for failure, outcome, held in [(slots.Rejected('no', SOURCE), 'rejected', False),
                                       (slots.OperatorRequired('cleanup failed'), 'operator_required', True),
                                       (OSError('systemd'), None, False)]:
            with self.subTest(failure=type(failure).__name__), patch.object(pull.slots, 'activate', side_effect=failure):
                (self.state/'hold.json').unlink(missing_ok=True)
                (self.state/'checked.json').unlink(missing_ok=True)
                with patch.multiple(pull, STATE=self.state), patch.object(control, 'HOLD', self.state/'hold.json'), \
                     self.assertRaises(type(failure)):
                    pull.activate_in_slot(self.host, self.staged, manifest(), CONFIG, SOURCE)
                self.assertEqual(self.checked(), outcome)
                self.assertEqual((self.state/'hold.json').exists(), held)

    def test_a_promoted_candidate_that_still_drains_the_old_slot_is_recorded_healthy(self):
        (self.state/'current.json').write_text(json.dumps({'source_commit': SOURCE}))
        with patch.object(pull.slots, 'activate', side_effect=control.Deferred('slot a still drains')), \
             patch.multiple(pull, STATE=self.state), self.assertRaises(control.Deferred):
            pull.activate_in_slot(self.host, self.staged, manifest(), CONFIG, SOURCE)
        self.assertEqual(self.checked(), 'healthy')

    def test_recovery_holds_the_controller_only_when_a_person_is_needed(self):
        for failure, held in [(slots.OperatorRequired('rollback refused'), True), (OSError('database busy'), False)]:
            with self.subTest(failure=type(failure).__name__), patch.object(pull.slots, 'resume', side_effect=failure):
                (self.state/'hold.json').unlink(missing_ok=True)
                with patch.multiple(pull, STATE=self.state), patch.object(control, 'HOLD', self.state/'hold.json'), \
                     self.assertRaises(type(failure)):
                    pull.resume_transition(self.host, CONFIG)
                self.assertEqual((self.state/'hold.json').exists(), held)

    def test_a_slot_host_hands_the_candidate_to_the_coordinator_without_a_maintenance_window(self):
        with patch.object(pull, 'download_and_handle') as handle, \
             patch.multiple(pull, STATE=self.state, api=lambda p: {'status': 'ahead'},
                            candidate=lambda *a: {'tag_name': 'server-' + SOURCE, 'assets': []}), \
             patch.object(pull.slots, 'SystemHost', return_value=self.host), patch.object(control, 'HOLD', self.state/'hold.json'):
            pull.run({'minimum_source': OLD, 'public_health': CONFIG['public_health']}, True, False)
        self.assertIs(handle.call_args.args[5], self.host)

    def test_a_locked_directory_defers_instead_of_failing(self):
        import sqlite3
        with patch.object(self.host, 'directory', side_effect=sqlite3.OperationalError('database is locked')), \
             self.assertRaises(control.Deferred):
            slots.in_slot_mode(self.host)


class FrontendLiveStatus(unittest.TestCase):
    """Every frontend observes the authoritative active slot, never a retired or draining port."""

    def test_either_active_slot_is_selected_while_the_other_is_draining_or_offline(self):
        for target in ['a', 'b']:
            with self.subTest(active=target):
                host = FakeHost()
                old = slots.other(target)
                host.rows = {old: row(old, 'draining', 'old'),
                             target: row(target, 'active', 'new', contract='w36-r13-' + '0'*16)}
                host.running = {target}
                with patch.object(slot_host, 'SystemHost', return_value=host), patch.object(activate, 'urlopen') as legacy:
                    live = activate.status()
                self.assertEqual((live['slot'], live['generation'], live['contract']),
                                 (target, 'new', 'w36-r13-' + '0'*16))
                legacy.assert_not_called()

    def test_a_stale_directory_or_different_process_never_supplies_a_contract(self):
        host = FakeHost()
        host.rows['a']['state'] = 'draining'
        with patch.object(slot_host, 'SystemHost', return_value=host), self.assertRaises(ValueError):
            activate.status()
        host.rows['a']['state'] = 'active'
        with patch.object(slot_host, 'SystemHost', return_value=host), \
             patch.object(host, 'status', return_value={'slot': 'a', 'generation': 'other', 'admitting': True}), \
             self.assertRaisesRegex(ValueError, 'identity/admission'):
            activate.status()

    def test_legacy_fallback_requires_a_real_single_process_identity(self):
        from unittest.mock import MagicMock
        response = MagicMock()
        response.__enter__.return_value.read.return_value = b'{"contract":"legacy"}'
        with patch.object(slot_host.SystemHost, 'directory', return_value=[]), \
             patch.object(activate, 'urlopen', return_value=response):
            self.assertEqual(activate.status(), {'contract': 'legacy'})
            response.__enter__.return_value.read.return_value = b'{"slot":"a","contract":"legacy"}'
            with self.assertRaisesRegex(ValueError, 'missing from the directory'):
                activate.status()


class Machine(unittest.TestCase):
    """The production host adapter, with the system commands and the proxy replaced by local stand-ins."""

    def setUp(self):
        folder = tempfile.TemporaryDirectory()
        self.addCleanup(folder.cleanup)
        self.root = Path(folder.name)
        self.host = slot_host.SystemHost()

    def test_the_directory_is_read_without_writing_and_freshness_follows_the_server_rule(self):
        db = self.root/'tbg.sqlite3'
        with sqlite3.connect(db) as connection:
            connection.execute('CREATE TABLE release_slots (slot TEXT, generation TEXT, contract TEXT, state TEXT, ranked_authority INTEGER,'
                               ' matches_running INTEGER, results_pending INTEGER, ranked_inflight INTEGER, heartbeat_unix INTEGER, version INTEGER)')
            connection.execute("INSERT INTO release_slots VALUES ('a','g','c','active',1,0,0,0,990,4), ('b','h','c','ready',0,0,0,0,900,2)")
            connection.execute('CREATE TABLE schema_version (version INTEGER)')
            connection.execute('INSERT INTO schema_version VALUES (9), (10)')
        with patch.object(slot_host, 'DB', db), patch.object(self.host, 'now', return_value=1000.0):
            rows = {r['slot']: r for r in self.host.directory()}
            self.assertEqual(self.host.schema(), 10)
        self.assertEqual((rows['a']['fresh'], rows['b']['fresh'], rows['a']['version']), (True, False, 4))

    def test_a_websocket_upgrade_proves_the_route_only_when_the_proxy_reaches_that_slot(self):
        seen = []

        class Proxy(http.server.BaseHTTPRequestHandler):
            def do_GET(self):
                seen.append((self.path, self.headers.get('Upgrade'), self.headers.get('Origin')))
                self.send_response(101 if self.path.startswith('/ws/slot-b?contract=c1') else 404)
                self.send_header('Connection', 'Upgrade')
                self.end_headers()

            def log_message(self, *args):
                pass
        server = http.server.HTTPServer(('127.0.0.1', 0), Proxy)
        threading.Thread(target=server.serve_forever, daemon=True).start()
        self.addCleanup(server.server_close)
        base = f'http://127.0.0.1:{server.server_port}'
        self.assertTrue(self.host.public_ws(base, 'b', 'c1'))
        self.assertFalse(self.host.public_ws(base, 'a', 'c1'))
        self.assertEqual(seen[0], ('/ws/slot-b?contract=c1', 'websocket', None), 'native-style: no Origin header')
        server.shutdown()
        self.assertFalse(self.host.public_ws(base, 'b', 'c1'), 'a proxy that does not answer is not a route')

    def test_the_candidate_describes_itself_from_a_temporary_directory_that_is_always_removed(self):
        releases = self.root/'releases'
        releases.mkdir()
        staged = self.root/'staged'
        staged.mkdir()
        (staged/'tbg-server').write_bytes(SERVER)
        run = lambda stdout, code: type('R', (), {'stdout': stdout, 'returncode': code})()
        with patch.object(slot_host, 'RELEASES', releases):
            with patch.object(slot_host.subprocess, 'run', return_value=run(b'{"contract": "c1"}', 0)) as called:
                self.assertEqual(self.host.inspect(staged, manifest()), {'contract': 'c1'})
            command = called.call_args.args[0]
            self.assertEqual(command[:5], ['runuser', '-u', 'nobody', '--', 'env'])
            self.assertEqual(command[-1], '--build-info')
            with patch.object(slot_host.subprocess, 'run', return_value=run(b'', 1)), self.assertRaises(ValueError):
                self.host.inspect(staged, manifest())
            self.assertEqual(list(releases.iterdir()), [])

    def test_commands_run_as_the_right_user_and_a_slow_stop_is_a_deferral_not_a_failure(self):
        with patch.object(slot_host.subprocess, 'run', return_value=type('R', (), {'stdout': 'ok', 'stderr': '', 'returncode': 0})()) as run:
            self.host.admin('promote', 'b', '--expect', 4)
            self.assertEqual(run.call_args.args[0][:4], ['runuser', '-u', 'tbg', '--'])
            self.assertEqual(run.call_args.args[0][-4:], ['promote', 'b', '--expect', '4'])
            self.host.stop_slot('b')
            self.assertEqual(run.call_args.args[0], ['systemctl', 'disable', '--now', 'tbg-server@b'])
        with patch.object(slot_host.subprocess, 'run', side_effect=subprocess.TimeoutExpired('stop', 1)):
            code, text = self.host.retire_slot('a')
        self.assertEqual(code, 3)
        self.assertIn('still stopping', text)

    def test_a_generation_that_never_served_can_be_forgotten_and_nothing_else_is_touched(self):
        releases = self.root/'releases'
        (releases/'abc').mkdir(parents=True)
        (releases/'abc'/'tbg-server').write_bytes(SERVER)
        (releases/'other').mkdir()
        with patch.object(slot_host, 'RELEASES', releases):
            self.host.discard('abc')
        self.assertEqual([p.name for p in releases.iterdir()], ['other'])


class Status(unittest.TestCase):
    def test_every_family_reports_timer_receipt_check_journal_and_hold(self):
        with tempfile.TemporaryDirectory() as folder:
            state, flags = Path(folder)/'state', Path(folder)/'etc'
            state.mkdir(), flags.mkdir()
            (flags/'web-deploy.enabled').write_text('')
            (state/'current.json').write_text(json.dumps({'source_commit': SOURCE}))
            (state/'checked.json').write_text(json.dumps({'source_commit': SOURCE, 'outcome': 'rejected', 'reason': 'forged'}))
            (state/'slot-transition.json').write_text(json.dumps({'phase': 'promoted', 'source_commit': SOURCE}))
            (state/'operator-hold.json').write_text(json.dumps({'family': 'backend', 'reason': 'rollback refused'}))

            def systemctl(*args):
                return 'enabled' if args[0] == 'is-enabled' else 'active'
            lines = []
            with patch.multiple(status, STATE=state, FLAGS=flags, systemctl=systemctl), \
                 patch.object(status.subprocess, 'run', return_value=type('R', (), {'stdout': 'slot state\na active'})()), \
                 patch('builtins.print', side_effect=lambda *a: lines.append(' '.join(map(str, a)))):
                status.main()
            text = '\n'.join(lines)
        self.assertIn('operator hold: backend — rollback refused', text)
        for family in ['backend:', 'web:', 'static:', 'native:']:
            self.assertIn(family, text)
        self.assertIn('flag present', text)
        self.assertIn(f'checked {SOURCE[:12]}: rejected — forged', text)
        self.assertIn('JOURNAL slot-transition.json: phase promoted', text)
        self.assertIn('(active)', text, 'the last result of each service is shown')
        self.assertIn('slot state', text)


if __name__ == '__main__':
    unittest.main()
