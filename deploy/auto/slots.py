"""Promote a verified backend into the idle release slot, then drain and retire the previous one.

Both generations share one database, so a promotion is one audited row change (P66, D-92): no
restart, no proxy reload, and a running match never moves. The previous slot keeps serving its
matches, reconnects and pending results until its own retirement gate passes. Every step is
journaled before it runs and every resume reads the directory, which is the truth, before it
trusts the journal, so a crashed, deferred or manually interrupted run converges instead of
repeating or abandoning a step.
"""
import json
import sqlite3
import subprocess
import time

from activate import RELEASES, STATE, sync_directory, write_json
from control import Deferred
from policy import require
from slot_host import SystemHost  # noqa: F401  (production host; tests and the rehearsal pass their own)

JOURNAL = STATE/'slot-transition.json'
ACTOR = 'auto-deploy'
# Lobby waiters (queue, rooms, invitations, rematch offers) live on one slot and cannot follow a promotion.
QUIET_WINDOW = 90
# After promotion the new slot must answer through the public proxy this many times in a row.
SETTLE_WINDOW = 60
SETTLE_STREAK = 3
SETTLE_STEP = 2
# The slot that must keep admitting is watched this long before the other one retires or the release rolls back.
KEEPER_WINDOW = 30
# Open idle sockets are not matches; they reconnect through admission, so they only delay retirement briefly.
IDLE_GRACE = 120
# A drain that outlives a day is a stuck match or result, not a long one: say so instead of waiting silently.
MAX_DRAIN = 24 * 3600
MIN_MEMORY_MB = 768
MIN_DISK = 512 * 1024 * 1024
BUSY_DATABASE = ('Db(', 'database is locked')


class Rejected(ValueError):
    """The candidate was turned away and the previous release is serving: latch it, nothing needs a person."""

    def __init__(self, message, source=None):
        super().__init__(message)
        self.source = source


class OperatorRequired(ValueError):
    """The state is not one automation may repair: hold the controller until a person has looked."""


class Busy(RuntimeError):
    """The shared database was only busy: nothing is wrong, the next run retries."""


def busy(text):
    return any(marker in text for marker in BUSY_DATABASE)


def admin(host, *args, attempts=3):
    """An admin command, retried while the shared database is only busy."""
    for _ in range(attempts):
        code, text = host.admin(*map(str, args))
        if code == 0 or not busy(text):
            break
        host.sleep(1)
    return code, text


def refused(text, message):
    """A failed admin step is a busy database to retry later, or a state a person has to look at."""
    return (Busy if busy(text) else OperatorRequired)(f'{message}: {text}')


def by_slot(host):
    return {row['slot']: row for row in host.directory()}


def in_slot_mode(host):
    """True once a slot process has registered (a single process never writes the directory) or a transition is open."""
    try:
        rows = host.directory()
    except sqlite3.OperationalError as error:
        if 'locked' in str(error) or 'busy' in str(error):
            raise Deferred(f'the release directory cannot be read now: {error}') from None
        return JOURNAL.exists()      # no directory table: a host that never ran a slot
    return JOURNAL.exists() or any(row['state'] in {'active', 'canary', 'ready', 'draining', 'warming'} for row in rows)


def lobby_waiting(metrics):
    """Players waiting on this slot for a partner, who cannot follow a promotion to the next slot."""
    if 'lobby_waiting' in metrics:
        return metrics['lobby_waiting']
    # A generation without the gauge: every socket that is not a seat or a spectator might be a waiter.
    return max(0, metrics['connections'] - 2 * metrics['matches_running'] - metrics['spectators']) + metrics['queue_len']


def other(slot):
    return 'b' if slot == 'a' else 'a'


def is_active(row, generation=None):
    return row is not None and row['state'] == 'active' and (generation is None or row['generation'] == generation)


def serving(host, slot):
    """The slot admits new players now: active in the directory, heartbeating and answering as that generation."""
    row = by_slot(host).get(slot)
    status = host.status(slot)
    return bool(is_active(row) and row['fresh'] and status is not None and status['admitting']
                and not status['shutdown'] and status['generation'] == row['generation'])


def wait_serving(host, slot, seconds):
    deadline = host.now() + seconds
    while True:
        if serving(host, slot):
            return True
        if host.now() >= deadline:
            return False
        host.sleep(SETTLE_STEP)


def preflight_host(host):
    """Read-only checks that need no candidate: one active slot, a free idle slot, room for two generations."""
    rows = by_slot(host)
    active = [r for r in rows.values() if r['state'] == 'active' and r['fresh']]
    require(len(active) == 1, 'exactly one fresh active release slot is required')
    current = active[0]
    idle = other(current['slot'])
    held = rows.get(idle)
    if host.is_active(idle):
        if held is not None and held['state'] not in {'retired', 'failed'}:
            raise Deferred(f"slot {idle} is {held['state']}: the previous release is still in progress")
        raise ValueError(f'slot {idle} is running but not a live part of the directory; operator review required')
    status = host.status(current['slot'])
    require(status is not None and status['slot'] == current['slot'], f"slot {current['slot']} does not answer")
    require(not status['shutdown'] and status['admitting'], 'the active slot is not admitting')
    require(not any(r['ranked_authority'] for r in rows.values() if r['slot'] != current['slot']),
            'ranked authority sits on the idle slot; operator review required')
    require(host.free_disk() >= MIN_DISK, 'insufficient release space')
    require(host.free_memory_mb() >= MIN_MEMORY_MB, 'not enough free memory to run two generations')
    return current, idle


def preflight(host, manifest, coordinated_contract=None):
    """Read-only: the active slot and the idle one for this candidate, or why it cannot start."""
    current, idle = preflight_host(host)
    if coordinated_contract is not None:
        require(manifest['contract'] == coordinated_contract, 'candidate differs from the operator-confirmed contract')
    if current['contract'] != manifest['contract'] and coordinated_contract is None:
        raise Rejected('contract change staged: coordinated client release required', manifest['source_commit'])
    if host.schema() != manifest['database_schema']:
        raise Rejected('database change staged: migration review required', manifest['source_commit'])
    return current, idle


def gate(host, ignore_lobby=False):
    """Can a release start now? Cheap, read-only and asked before anything is downloaded."""
    current, _ = preflight_host(host)
    if not ignore_lobby:
        await_quiet(host, current['slot'], QUIET_WINDOW)


def generation_for(source):
    """The first unused immutable generation id for this source (a failed attempt is never reused)."""
    for attempt in range(1, 100):
        name = source[:12] + ('' if attempt == 1 else f'-{attempt}')
        if not (RELEASES/name).exists():
            return name
    raise ValueError('too many generations for one source')


def load_journal():
    return json.loads(JOURNAL.read_text()) if JOURNAL.exists() else None


def save(journal, **changes):
    journal.update(changes, updated=time.time())
    write_json(JOURNAL, journal)


def stop_unpromoted(host, slot, generation=None):
    """Tear down a slot that never admitted a player; refuses when it holds players, work or another generation."""
    row = by_slot(host).get(slot)
    if row is not None and row['state'] not in {'retired', 'failed'}:
        if row['state'] in {'active', 'canary'}:
            raise OperatorRequired(f'slot {slot} admits players; not stopping it')
        if generation is not None and row['generation'] != generation:
            raise OperatorRequired(f"slot {slot} holds generation {row['generation']}, not the journaled {generation}")
        if row['matches_running'] or row['results_pending']:
            raise OperatorRequired(f'slot {slot} holds matches or results; not stopping it')
    host.stop_slot(slot)
    row = by_slot(host).get(slot)
    if row is not None and row['state'] not in {'retired', 'failed'}:
        # A candidate that died while warming is still `warming`, and only a drained slot can be retired.
        if row['state'] not in {'draining', 'ready'}:
            admin(host, 'drain', slot, '--actor', ACTOR)
        code, text = admin(host, 'retire', slot, '--actor', ACTOR)
        if code != 0:
            raise refused(text, f'slot {slot} could not be retired after stopping')


def abandon(host, journal):
    """Tear down the candidate slot that never admitted a player, forget its files and close the journal."""
    stop_unpromoted(host, journal['to'], journal['generation'])
    host.discard(journal['generation'])
    JOURNAL.unlink(missing_ok=True)


def verify_ready(host, slot, generation, manifest):
    status = host.status(slot)
    row = by_slot(host).get(slot)
    require(status is not None and row is not None, f'slot {slot} does not answer')
    require(row['generation'] == generation and row['state'] == 'ready' and row['fresh'], f'slot {slot} is not ready')
    require(status['generation'] == generation and status['contract'] == manifest['contract'], 'slot identity mismatch')
    require(not status['shutdown'] and host.store_ok(slot), f'slot {slot} store check failed')


def await_quiet(host, slot, window):
    """Wait for an empty lobby on the active slot, for at most `window` seconds."""
    deadline = host.now() + window
    while True:
        metrics = host.metrics(slot)
        require(metrics is not None, f'slot {slot} does not answer')
        if lobby_waiting(metrics) == 0:
            return
        if host.now() >= deadline:
            raise Deferred(f'players are waiting in the lobby of slot {slot}; the promotion waits for a quiet moment')
        host.sleep(1)


def settle(host, slot, generation, manifest, base, window):
    """The promoted slot answers admission, health and a WebSocket upgrade through the public proxy."""
    deadline, streak = host.now() + window, 0
    while True:
        status = host.status(slot)
        answer = host.public_admission(base, manifest['contract'])
        good = (status is not None and status['slot'] == slot and status['generation'] == generation and status['admitting']
                and not status['shutdown'] and answer is not None and answer.get('status') == 'ok'
                and answer.get('endpoint') == f'/ws/slot-{slot}' and host.public_ok(base)
                and host.public_ws(base, slot, manifest['contract']))
        streak = streak + 1 if good else 0
        if streak >= SETTLE_STREAK:
            return status
        if host.now() >= deadline:
            return None
        host.sleep(SETTLE_STEP)


def move_ranked(host, slot, attempts=100):
    """New ranked matches need the ranked authority; it moves once the holder has nothing in flight."""
    for _ in range(attempts):
        status = host.status(slot)
        if status is not None and status['ranked']:
            return True
        code, _ = admin(host, 'ranked', slot, '--actor', ACTOR)
        if code == 0:
            return True
        host.sleep(0.2)
    return False


def receipt(journal):
    write_json(STATE/'current.json', {**journal['manifest'], 'slot': journal['to'], 'generation': journal['generation']})


def restore_receipt(journal):
    """Undo the receipt of a candidate that is no longer the live one, if this transition wrote it."""
    current = STATE/'current.json'
    if not current.exists() or json.loads(current.read_text()).get('source_commit') != journal['source_commit']:
        return
    previous = journal.get('previous_receipt')
    if previous is None:
        current.unlink()
    else:
        write_json(current, previous)


def roll_back(host, journal, reason):
    """New admissions return to the previous slot, which is still running, and the candidate slot drains."""
    save(journal, phase='rolling_back')
    # Recovery is worth a longer wait for a busy database than a routine step.
    code, text = admin(host, 'rollback', journal['from'], '--actor', ACTOR, attempts=20)
    if code != 0:
        rows = by_slot(host)
        if not (is_active(rows.get(journal['from'])) and not is_active(rows.get(journal['to']), journal['generation'])):
            raise refused(text, f"rollback to slot {journal['from']} refused")
    restore_receipt(journal)
    move_ranked(host, journal['from'])
    save(journal, phase='rolled_back', retiring=journal['to'], reason=reason[:300])


def stuck(host, journal, what):
    """Raise instead of deferring once a drain has outlived a day."""
    if host.now() - journal.get('started', host.now()) > MAX_DRAIN:
        raise OperatorRequired(f'{what} for over {MAX_DRAIN // 3600} hours; a match or result is stuck')


def drain(host, journal):
    """Retire the slot named by the journal once its retirement gate passes; otherwise wait for the next run.

    The slot that stays must be admitting first: retiring the old slot while its successor is down would
    leave nobody to take new players, and a slot that still holds players is never the one to stop.
    """
    retiring = journal['retiring']
    keep = journal['to'] if retiring == journal['from'] else journal['from']
    row = by_slot(host).get(retiring)
    if row is None or (row['state'] == 'retired' and not host.is_active(retiring)):
        return
    if not wait_serving(host, keep, KEEPER_WINDOW):
        if journal['phase'] == 'promoted':
            roll_back(host, journal, f'slot {keep} stopped admitting while slot {retiring} drained')
            raise Rejected('the promoted slot stopped admitting; admissions returned to the previous release',
                           journal['source_commit'])
        raise OperatorRequired(f'slot {keep} does not admit players while slot {retiring} still runs; operator review required')
    row = by_slot(host).get(retiring)
    if row is None or (row['state'] == 'retired' and not host.is_active(retiring)):
        return
    lists_work = row['matches_running'] or row['results_pending'] or row['ranked_inflight']
    if not host.is_active(retiring) and not row['fresh'] and lists_work:
        # Its counters are stale: either systemd brings it back or its work needs a person; an idle one retires below.
        if host.now() - row['heartbeat_unix'] > 600:
            raise OperatorRequired(f'slot {retiring} is not running but still lists work; its matches or results need review')
        raise Deferred(f'slot {retiring} is not answering; waiting for systemd to restart it')
    code, text = admin(host, 'retire-check', retiring)
    if code != 0:
        stuck(host, journal, f'slot {retiring} has been draining')
        raise Deferred(f'slot {retiring} still drains: {text}')
    drained = journal.get('drained_at') or host.now()
    save(journal, drained_at=drained)
    metrics = host.metrics(retiring) or {'connections': 0}
    if metrics['connections'] > 0 and host.now() - drained < IDLE_GRACE:
        stuck(host, journal, f'slot {retiring} has kept idle connections')
        raise Deferred(f"slot {retiring} has {metrics['connections']} idle connections; they reconnect through admission")
    code, text = host.retire_slot(retiring)
    if code == 3:
        raise Deferred(f'slot {retiring} became busy again or is still stopping: {text}')
    if code != 0 and not host.is_active(retiring):
        # The slot stopped but the last step, marking it retired, failed (a busy database): do just that.
        code, text = admin(host, 'retire', retiring, '--actor', ACTOR)
    require(code == 0, f'retiring slot {retiring} failed: {text}')
    row = by_slot(host).get(retiring)
    require(row is not None and row['state'] == 'retired', f'slot {retiring} is not retired after stopping')


def finish(host, journal):
    """The candidate serves and the previous slot is gone: the journal closes and the receipt stays."""
    host.adopt_admin(RELEASES/journal['generation'])
    JOURNAL.unlink()
    sync_directory(STATE)


def promote(host, journal, config):
    """Promote, verify through the public proxy and roll back if the candidate does not serve."""
    manifest = journal['manifest']
    save(journal, phase='promoting')
    code, text = 1, ''
    for _ in range(3):
        # A database that was only busy, or a row version that moved under us, is worth another look.
        code, text = admin(host, 'promote', journal['to'], '--expect', by_slot(host)[journal['to']]['version'], '--actor', ACTOR)
        if code == 0 or 'Stale' not in text:
            break
    if code != 0 and not is_active(by_slot(host).get(journal['to']), journal['generation']):
        abandon(host, journal)
        raise Rejected(f'promotion refused: {text}', manifest['source_commit'])
    save(journal, phase='promoted', retiring=journal['from'])
    move_ranked(host, journal['to'])
    if settle(host, journal['to'], journal['generation'], manifest, config['public_api'], SETTLE_WINDOW) is None:
        roll_back(host, journal, 'the promoted slot did not answer through the public proxy')
        raise Rejected('candidate failed live verification; admissions returned to the previous release',
                       manifest['source_commit'])
    save(journal, verified=True)
    receipt(journal)


def activate(host, staged, manifest, config, ignore_lobby=False, coordinated_contract=None):
    """Install the candidate into the idle slot and make it the admitting one; resumable via the journal.

    `ignore_lobby` is an operator decision (an urgent fix in a busy hour): players still waiting in a queue,
    room or invitation are then refused by the draining slot and ask admission again instead of being awaited.
    """
    source = manifest['source_commit']
    active, idle = preflight(host, manifest, coordinated_contract)
    if not ignore_lobby:
        await_quiet(host, active['slot'], QUIET_WINDOW)
    try:
        info = host.inspect(staged, manifest)
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        raise Rejected(f'the candidate cannot describe itself: {error}', source) from error
    if any(info.get(key) != manifest.get(key) for key in ['source_commit', 'contract', 'database_schema', 'target']):
        raise Rejected('candidate identity differs from its attested manifest', source)
    held = by_slot(host).get(idle)
    if held is not None and held['state'] not in {'retired', 'failed'}:
        # A previous attempt that died while warming left a row behind (preflight refuses a running slot).
        stop_unpromoted(host, idle)
    generation = generation_for(source)
    previous = STATE/'current.json'
    journal = {'source_commit': source, 'generation': generation, 'from': active['slot'], 'to': idle, 'phase': 'installing',
               'started': host.now(), 'manifest': manifest, 'coordinated_contract': coordinated_contract,
               'previous_receipt': json.loads(previous.read_text()) if previous.exists() else None}
    save(journal)
    try:
        code, text = host.start_slot(idle, generation, f'{staged}/tbg-server', source[:12])
        require(code == 0, f'slot {idle} did not become ready: {text}')
        host.keep_files(staged, generation, manifest)
        verify_ready(host, idle, generation, manifest)
        save(journal, phase='ready')
        # A player may have queued while the slot warmed up; the last look is immediately before the switch.
        if not ignore_lobby:
            await_quiet(host, active['slot'], QUIET_WINDOW)
    except (Exception, KeyboardInterrupt) as error:
        try:
            abandon(host, journal)
        except Exception as cleanup:
            save(journal, phase='cleanup_failed', reason=str(cleanup)[:300])
            if isinstance(cleanup, (Busy, OperatorRequired)):
                raise
            raise OperatorRequired(f'cleaning up slot {idle} failed: {cleanup}') from cleanup
        if isinstance(error, ValueError) and not isinstance(error, (Rejected, OperatorRequired)):
            raise Rejected(str(error), source) from error
        raise
    promote(host, journal, config)
    drain_after_promotion(host, journal)


def drain_after_promotion(host, journal):
    drain(host, journal)
    finish(host, journal)


def resume(host, config):
    """Continue an unfinished transition from its journal and the directory; return when none is left."""
    journal = load_journal()
    if journal is None:
        return
    phase, generation, source = journal['phase'], journal['generation'], journal['source_commit']
    rows = by_slot(host)
    origin, target = rows.get(journal['from']), rows.get(journal['to'])
    candidate_admits = is_active(target, generation)
    if phase == 'cleanup_failed':
        abandon(host, journal)
        raise Deferred('the failed cleanup was completed; the release starts again from the candidate')
    if not candidate_admits and phase in {'installing', 'ready', 'promoting'}:
        if not is_active(origin):
            raise OperatorRequired('no release slot admits players; operator review required')
        abandon(host, journal)
        raise Deferred('an interrupted release never reached promotion; it starts again from the candidate')
    if not candidate_admits:
        # Promoted earlier, and the previous release admits again: rolled back by this controller before a crash, or by hand.
        if not is_active(origin):
            raise OperatorRequired('no release slot admits players; operator review required')
        restore_receipt(journal)
        move_ranked(host, journal['from'], attempts=1)
        save(journal, phase='rolled_back', retiring=journal['to'])
        if phase != 'rolled_back':
            raise Rejected('admissions were returned to the previous release', source)
        drain(host, journal)
        JOURNAL.unlink()
        return
    if phase == 'rolled_back':
        raise OperatorRequired('the rejected candidate admits players again (promoted by hand?); operator review required')
    if phase != 'promoted':
        save(journal, phase='promoted', retiring=journal['from'])
    if not journal.get('verified'):
        if settle(host, journal['to'], generation, journal['manifest'], config['public_api'], SETTLE_WINDOW) is None:
            roll_back(host, journal, 'verification did not complete after an interruption')
            raise Rejected('candidate failed live verification; admissions returned to the previous release', source)
        save(journal, verified=True)
    receipt(journal)
    move_ranked(host, journal['to'], attempts=1)
    drain_after_promotion(host, journal)
