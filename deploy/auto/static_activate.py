"""Promote public static sites without touching the backend, game assets or Caddy config."""
import json
import os
import re
import shutil
from pathlib import Path

from activate import STATE, sync_directory, sync_file, write_json
from policy import digest, require
from static_policy import SITES
from web_activate import probe

ROOT = Path('/srv/tbg')
JOURNAL = STATE/'static-transition.json'
RECEIPT = STATE/'static-current.json'


def entry(site, target):
    require(site in SITES, 'unknown static site')
    root = ROOT/site
    pointer = root/'current'
    if target is None:
        pointer.unlink(missing_ok=True)
    else:
        require(re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9._-]{0,100}', target), 'invalid static pointer')
        require((root/'releases'/target).is_dir(), 'static release missing')
        temporary = root/'current.next'
        temporary.unlink(missing_ok=True)
        temporary.symlink_to('releases/' + target)
        os.replace(temporary, pointer)
    sync_directory(root)


def recover():
    if JOURNAL.exists():
        previous = json.loads(JOURNAL.read_text())
        for site, target in previous['previous'].items():
            entry(site, target)
        if previous['receipt'] is None:
            RECEIPT.unlink(missing_ok=True)
        else:
            write_json(RECEIPT, previous['receipt'])
        JOURNAL.unlink()
        sync_directory(STATE)


def stage(staged, manifest):
    previous = {}
    for site, files in manifest['sites'].items():
        root = ROOT/site/'releases'
        root.mkdir(parents=True, exist_ok=True, mode=0o755)
        root.chmod(0o755)
        root.parent.chmod(0o755)
        candidate = root/manifest['source_commit']
        if not candidate.exists():
            temporary = root/(manifest['source_commit'] + '.staging')
            require(not temporary.exists(), 'static staging requires operator review')
            shutil.copytree(Path(staged)/site, temporary)
            temporary.chmod(0o755)
            for path in temporary.rglob('*'):
                path.chmod(0o755 if path.is_dir() else 0o644)
                if path.is_file():
                    sync_file(path)
            sync_directory(temporary)
            os.rename(temporary, candidate)
            sync_directory(root)
        require({p.relative_to(candidate).as_posix() for p in candidate.rglob('*') if p.is_file()} == set(files), 'static file collision')
        require(all(digest(candidate/name) == value for name, value in files.items()), 'immutable static collision')
        pointer = root.parent/'current'
        require(not pointer.exists() or pointer.is_symlink(), 'static entry adoption required')
        require(not pointer.is_symlink() or pointer.resolve().is_dir(), 'dangling static entry')
        require(not pointer.is_symlink() or pointer.resolve().parent == root.resolve(), 'static pointer escapes releases')
        previous[site] = pointer.resolve().name if pointer.is_symlink() else None
    return previous


def activate(staged, manifest):
    previous = stage(staged, manifest)
    write_json(JOURNAL, {'previous': previous, 'receipt': json.loads(RECEIPT.read_text()) if RECEIPT.exists() else None})
    try:
        for site in SITES:
            entry(site, manifest['source_commit'])
        for site, public in SITES.items():
            probe(public + '/', manifest['sites'][site]['index.html'])
        write_json(RECEIPT, manifest)
        JOURNAL.unlink()
        sync_directory(STATE)
    except BaseException:
        recover()
        raise
