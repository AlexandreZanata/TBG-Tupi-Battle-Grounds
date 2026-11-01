"""Validate bounded, attested backend archives and exact compatibility before activation."""
import hashlib
import json
import re
import tarfile
from pathlib import Path

REPOSITORY = 'AlexandreZanata/TBG-Tupi-Battle-Grounds'
WORKFLOW = REPOSITORY + '/.github/workflows/backend-release.yml'
SHA = re.compile(r'[0-9a-f]{40}')
CONTRACT = re.compile(r'w\d+-r\d+-[0-9a-f]{16}')
FILES = {'tbg-server', 'tbg-release-admin'}
MAX_ARCHIVE = 64 * 1024 * 1024
MAX_CONTENT = 128 * 1024 * 1024


def require(condition, message):
    if not condition:
        raise ValueError(message)


def digest(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def unpack(archive, destination, source):
    """Only after signature verification. Never use generic tar extraction."""
    require(bool(SHA.fullmatch(source)), 'invalid source SHA')
    require(Path(archive).stat().st_size <= MAX_ARCHIVE, 'archive too large')
    with tarfile.open(archive, 'r:gz') as tar:
        members = tar.getmembers()
        names = [m.name for m in members]
        require(len(names) == 3 and set(names) == FILES | {'manifest.json'}, 'unexpected archive members')
        require(all(m.isfile() and m.size <= MAX_CONTENT for m in members), 'nonregular or oversized member')
        require(sum(m.size for m in members) <= MAX_CONTENT, 'unpacked content too large')
        require(tar.getmember('manifest.json').size <= 4096, 'manifest too large')
        manifest = json.load(tar.extractfile('manifest.json'))
        require(isinstance(manifest, dict), 'manifest must be an object')
        require(manifest.get('schema') == 1 and manifest.get('source_commit') == source, 'manifest source mismatch')
        require(manifest.get('target') == 'x86_64-unknown-linux-gnu', 'unsupported target')
        require(bool(CONTRACT.fullmatch(manifest.get('contract', ''))), 'invalid match contract')
        require(type(manifest.get('database_schema')) is int and manifest['database_schema'] > 0, 'invalid database schema')
        require(isinstance(manifest.get('files'), dict) and set(manifest['files']) == FILES, 'invalid binary manifest')
        destination = Path(destination)
        destination.mkdir(mode=0o700)
        for name in sorted(FILES):
            path = destination / name
            with path.open('xb') as out:
                stream = tar.extractfile(name)
                while chunk := stream.read(1024 * 1024):
                    out.write(chunk)
            require(digest(path) == manifest['files'][name], 'binary digest mismatch')
            with path.open('rb') as stream:
                require(stream.read(4) == b'\x7fELF', 'not an ELF binary')
            path.chmod(0o755)
        return manifest


def compatible(manifest, live, schema):
    require(live.get('slot') is None, 'two-slot activation requires the P66 rollout procedure')
    require(live.get('contract') == manifest['contract'], 'contract change staged: coordinated client release required')
    require(schema == manifest['database_schema'], 'database change staged: migration review required')
    require(not live.get('shutdown'), 'another drain is in progress')
