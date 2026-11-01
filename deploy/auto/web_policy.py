"""Bounded verified browser bundles: exact source URLs, hashes and safe regular files."""
import json
import re
import tarfile
from pathlib import Path, PurePosixPath

from policy import CONTRACT, SHA, digest, require

WORKFLOW = 'AlexandreZanata/TBG-Tupi-Battle-Grounds/.github/workflows/web-release.yml'
MAX_ARCHIVE = 256 * 1024 * 1024
MAX_CONTENT = 512 * 1024 * 1024
MAX_FILES = 4096


def safe_name(name):
    path = PurePosixPath(name)
    return (not path.is_absolute() and str(path) == name and '\\' not in name
            and all(part not in {'.', '..'} and not re.search(r'[\x00-\x1f\x7f:]', part) for part in path.parts))


def unpack(archive, destination, source):
    require(bool(SHA.fullmatch(source)), 'invalid source')
    require(Path(archive).stat().st_size <= MAX_ARCHIVE, 'web archive too large')
    with tarfile.open(archive, 'r:gz') as tar:
        members = []
        size = 0
        for member in tar:
            size += member.size
            require(member.isfile() and safe_name(member.name), 'unsafe web member')
            require(size <= MAX_CONTENT and len(members) < MAX_FILES + 1, 'web content limit exceeded')
            members.append(member)
        require(1 < len(members) <= MAX_FILES + 1, 'invalid web file count')
        names = [m.name for m in members]
        require(len(names) == len(set(names)), 'duplicate web member')
        require(all(m.isfile() and safe_name(m.name) for m in members), 'unsafe web member')
        require(sum(m.size for m in members) <= MAX_CONTENT, 'expanded web bundle too large')
        require('manifest.json' in names and tar.getmember('manifest.json').size <= 1024 * 1024, 'invalid web manifest')
        meta = json.load(tar.extractfile('manifest.json'))
        require(tar.getmember('web/index.html').size <= 2 * 1024 * 1024, 'web index too large')
        require(meta.get('schema') == 1 and meta.get('source_commit') == source, 'web source mismatch')
        require(bool(CONTRACT.fullmatch(meta.get('contract', ''))), 'invalid web contract')
        require(meta.get('public_url') == '/releases/' + source + '/', 'mutable web URLs refused')
        files = meta.get('files')
        require(isinstance(files, dict) and 'index.html' in files, 'missing web index')
        require(set(names) == {'manifest.json'} | {'web/' + n for n in files}, 'web manifest/member mismatch')
        require(all(safe_name(n) and n.split('/')[0] not in {'admin', 'landing', 'mobile', 'android', 'ios'} for n in files),
                'invalid or excluded frontend path')
        root = Path(destination)
        root.mkdir(mode=0o755)
        for name, expected in files.items():
            require(isinstance(expected, str) and re.fullmatch(r'[0-9a-f]{64}', expected), 'invalid web hash')
            path = root/name
            path.parent.mkdir(parents=True, exist_ok=True)
            with path.open('xb') as output:
                stream = tar.extractfile('web/' + name)
                while chunk := stream.read(1024 * 1024):
                    output.write(chunk)
            require(digest(path) == expected, 'web file hash mismatch')
        require('<base href="' + meta['public_url'] + '">' in (root/'index.html').read_text(), 'web base URL mismatch')
        return meta
