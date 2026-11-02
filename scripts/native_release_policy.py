"""Which changes need a new desktop version, read from the reviewed sources."""
import fnmatch
import re
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
# Inputs that change the shipped packages. The workflow, guards and tests also
# rebuild packages (native-release.yml paths) but never need a new version alone.
RELEASE_PREFIXES = ('crates/tbg-client/', 'crates/tbg-core/', 'crates/tbg-update/', 'deploy/native/', 'docs/patches/')
RELEASE_FILES = {'Cargo.toml', 'Cargo.lock', '.gitattributes', 'rust-toolchain.toml', 'LICENSE',
                 'THIRD_PARTY_NOTICES.md', '.github/cargo-windows-release.toml', 'scripts/package-native.py',
                 'scripts/native-release-manifest.py', 'scripts/package-appimage.sh', 'scripts/install-linux.sh',
                 'scripts/managed-bootstrap.sh'}
RELEASE_GLOBS = ('scripts/*windows*.ps1',)
NOT_SHIPPED = ('/tests/', '/examples/', '/benches/')


def release_relevant(paths):
    def shipped(path):
        if path in RELEASE_FILES or any(fnmatch.fnmatchcase(path, glob) for glob in RELEASE_GLOBS):
            return True
        return path.startswith(RELEASE_PREFIXES) and not any(part in path for part in NOT_SHIPPED)
    return any(shipped(path) for path in paths)


def client_version(root=ROOT):
    return tomllib.loads((Path(root)/'crates/tbg-client/Cargo.toml').read_text())['package']['version']


def notes_version(root=ROOT):
    """The version deploy/native/RELEASE-NOTES.md was written for (its title line)."""
    title = (Path(root)/'deploy/native/RELEASE-NOTES.md').read_text().splitlines()[0]
    match = re.match(r'TBG (\S+) ', title)
    return match[1] if match else None
