"""Atomically promote a compatible browser entry without restarting players or the proxy."""
import json
import os
import shutil
from pathlib import Path
from urllib.request import Request, urlopen

from activate import STATE, status, sync_directory, sync_file, write_json
from policy import SHA, digest, require

ROOT = Path('/srv/tbg/web')
JOURNAL = STATE/'web-transition.json'
RECEIPT = STATE/'web-current.json'


def entry(target):
    pointer = ROOT/'current'
    if target is None:
        pointer.unlink(missing_ok=True)
    else:
        require(bool(SHA.fullmatch(target)), 'invalid web pointer')
        path = ROOT/'releases'/target
        require(path.is_dir(), 'previous web release missing')
        temp = ROOT/'current.next'
        temp.unlink(missing_ok=True)
        temp.symlink_to('releases/' + target)
        os.replace(temp, pointer)
    sync_directory(ROOT)


def recover():
    if JOURNAL.exists():
        previous = json.loads(JOURNAL.read_text())
        entry(previous['previous'])
        if previous['receipt'] is None:
            RECEIPT.unlink(missing_ok=True)
        else:
            write_json(RECEIPT, previous['receipt'])
        JOURNAL.unlink()
        sync_directory(STATE)


def probe(url, expected):
    # Cloudflare Browser Integrity Check rejects Python's generic default agent.
    request = Request(url, headers={'User-Agent': 'TBG-release-health/1.0', 'Cache-Control': 'no-cache'})
    with urlopen(request, timeout=15) as response:
        require(response.status == 200, 'browser entry not available')
        import hashlib
        require(hashlib.sha256(response.read(2 * 1024 * 1024 + 1)).hexdigest() == expected,
                'public browser entry differs from attested candidate')


def activate(staged, manifest, public, adopt=False):
    live = status()
    require(live['contract'] == manifest['contract'] and not live.get('shutdown'),
            'browser contract transition/drain requires coordinated release')
    require(public == 'https://app.tupibg.com', 'reviewed browser origin required')
    require(not ROOT.is_symlink(), 'browser root requires explicit adoption')
    ROOT.mkdir(parents=True, exist_ok=True, mode=0o755)
    ROOT.chmod(0o755)
    root = ROOT/'releases'
    root.mkdir(parents=True, exist_ok=True, mode=0o755)
    root.chmod(0o755)
    require(shutil.disk_usage(root).free >= 2 * sum(p.stat().st_size for p in Path(staged).rglob('*') if p.is_file()),
            'insufficient web staging space')
    candidate = root/manifest['source_commit']
    if candidate.exists():
        require({p.relative_to(candidate).as_posix() for p in candidate.rglob('*') if p.is_file()} == set(manifest['files']),
                'immutable web release file collision')
        require(all(digest(candidate/name) == value for name, value in manifest['files'].items()), 'immutable web collision')
    else:
        temporary = root/(manifest['source_commit'] + '.staging')
        require(not temporary.exists(), 'incomplete staging requires operator review')
        shutil.copytree(staged, temporary)
        temporary.chmod(0o755)
        for path in temporary.rglob('*'):
            path.chmod(0o755 if path.is_dir() else 0o644)
            if path.is_file():
                sync_file(path)
        sync_directory(temporary)
        os.rename(temporary, candidate)
        sync_directory(root)
    # Verify immutable routing before changing the public entry. No Caddy reload.
    probe(public + manifest['public_url'] + 'index.html', manifest['files']['index.html'])
    pointer = ROOT/'current'
    require(not pointer.exists() or pointer.is_symlink(), 'legacy current entry requires adoption')
    require(not pointer.is_symlink() or pointer.resolve().is_dir(), 'dangling web pointer')
    previous = pointer.resolve().name if pointer.is_symlink() else None
    require(previous is None or pointer.resolve().parent == root.resolve(), 'web pointer escapes releases')
    receipt = json.loads(RECEIPT.read_text()) if RECEIPT.exists() else None
    # A manual or foreign publish is never overwritten silently (drift); the
    # first adoption of a hand-made entry is an explicit operator decision.
    require(adopt or previous == (receipt or {}).get('source_commit'),
            'browser entry changed outside the controller; review it, then rerun with --adopt')
    write_json(JOURNAL, {'previous': previous, 'receipt': receipt})
    try:
        entry(manifest['source_commit'])
        probe(public + '/', manifest['files']['index.html'])
        write_json(RECEIPT, manifest)
        JOURNAL.unlink()
        sync_directory(STATE)
    except BaseException:
        recover()
        raise
