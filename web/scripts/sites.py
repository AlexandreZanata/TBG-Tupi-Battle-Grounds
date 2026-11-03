"""Site definitions and asset manifests shared by stage.py, serve.py and check.py.

A site is its own directory plus `site.manifest` lines that copy reviewed
files from elsewhere in the repository (game art, fonts, the patch catalog,
shared CSS/JS). The same mapping feeds the local preview and the staged
release, so what is previewed is what is published.
"""

import fnmatch
from dataclasses import dataclass
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
# Authoring files that never ship.
NEVER_SHIP = {"README.md", "site.manifest"}


@dataclass(frozen=True)
class Site:
    name: str
    root: Path
    # Local-only sites can be previewed but never staged for publication.
    local_only: bool = False


SITES = {
    "landing": Site("landing", REPO / "web/landing"),
    "admin-public": Site("admin-public", REPO / "web/admin/public"),
    "admin-app": Site("admin-app", REPO / "web/admin/app"),
    "admin-preview": Site("admin-preview", REPO / "web/admin/app", local_only=True),
}


class ManifestError(Exception):
    pass


def _manifest_entries(site):
    path = site.root / "site.manifest"
    if not path.is_file():
        return []
    entries = []
    for number, raw in enumerate(path.read_text().splitlines(), 1):
        line = raw.split("#", 1)[0].strip()
        if not line:
            continue
        parts = line.split()
        if len(parts) != 2:
            raise ManifestError(f"{path}:{number}: expected '<dest> <repo source>'")
        entries.append((parts[0], parts[1], f"{path.name}:{number}"))
    return entries


def _expand(dest, source, where):
    """Yield (published path, absolute source) pairs for one manifest line."""
    if dest.startswith("/") or ".." in Path(dest).parts or ".." in Path(source).parts:
        raise ManifestError(f"{where}: paths must stay inside the site and the repository")
    if any(ch in source for ch in "*?["):
        if not dest.endswith("/"):
            raise ManifestError(f"{where}: a glob source needs a directory destination ending in '/'")
        folder = REPO / Path(source).parent
        matches = sorted(p for p in folder.iterdir() if p.is_file() and fnmatch.fnmatch(p.name, Path(source).name))
        if not matches:
            raise ManifestError(f"{where}: {source} matches no file")
        for match in matches:
            yield dest + match.name, match
        return
    path = REPO / source
    if not path.is_file():
        raise ManifestError(f"{where}: missing source {source}")
    yield (dest + path.name if dest.endswith("/") else dest), path


def files(site):
    """Map every published path of a site to the file that provides it."""
    mapping = {}
    for path in sorted(site.root.rglob("*")):
        rel = path.relative_to(site.root)
        hidden = any(part.startswith(".") for part in rel.parts)
        if path.is_file() and rel.as_posix() not in NEVER_SHIP and not hidden:
            mapping[rel.as_posix()] = path
    for dest, source, where in _manifest_entries(site):
        for published, path in _expand(dest, source, where):
            if published in mapping:
                raise ManifestError(f"{where}: {published} is already provided by {mapping[published]}")
            mapping[published] = path
    return mapping
