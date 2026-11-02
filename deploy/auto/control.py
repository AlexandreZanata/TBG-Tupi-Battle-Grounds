"""Shared deployment lock, transient deferral and durable operator holds for every controller."""
import contextlib
import fcntl
import json
import subprocess
import sys
from urllib.error import URLError

from activate import STATE, write_json

HOLD = STATE/'operator-hold.json'


class Deferred(Exception):
    """A transient condition: nothing changed, nothing is latched, the next run retries."""


class OperatorHold(Exception):
    """A previous failure needs a person; automation refuses to act until it is released."""


@contextlib.contextmanager
def deployment_lock():
    # One lock serializes promotions across families. No step may block
    # indefinitely while holding it; long waits are resumable states instead.
    STATE.mkdir(mode=0o700, exist_ok=True)
    with (STATE/'lock').open('w') as lock:
        try:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            raise Deferred('another deployment step holds the shared lock') from None
        yield


def hold(family, reason):
    write_json(HOLD, {'family': family, 'reason': str(reason)[:500]})


def require_no_hold():
    if HOLD.exists():
        held = json.loads(HOLD.read_text())
        raise OperatorHold(f"{held.get('family')} requires an operator: {held.get('reason')}; "
                           'inspect it, then rerun with --release-hold')


def release_hold():
    HOLD.unlink(missing_ok=True)


def run_controller(body):
    """Exit 0 for success or a deferral; any other failure is a visible non-zero exit."""
    try:
        body()
    except Deferred as reason:
        print(f'Deferred: {reason}')
    except (OperatorHold, OSError, URLError, ValueError, KeyError, subprocess.SubprocessError) as error:
        print(str(error), file=sys.stderr)
        raise SystemExit(1) from None
