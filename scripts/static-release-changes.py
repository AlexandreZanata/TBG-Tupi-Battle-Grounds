#!/usr/bin/env python3
"""Publish changed public sites and their shared data/art without rebuilding the backend."""
import fnmatch
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MANIFESTS = ('web/landing/site.manifest', 'web/admin/public/site.manifest')


def manifest_sources():
    """Every repository source (file or glob) a public site packages, so the filter cannot drift."""
    sources = set()
    for manifest in MANIFESTS:
        for line in (ROOT/manifest).read_text().splitlines():
            fields = line.split()
            if len(fields) == 2 and not line.lstrip().startswith('#'):
                sources.add(fields[1])
    return sources


def relevant(paths):
    exact = {'docs/patches/catalog.json', '.github/workflows/static-release.yml',
             'scripts/package-static-sites.py', 'scripts/static-release-changes.py',
             'scripts/release_scope.py', *MANIFESTS}
    prefixes = ('web/', 'crates/tbg-client/assets/branding/', 'crates/tbg-client/assets/fonts/',
                'crates/tbg-client/assets/arenas/', 'crates/tbg-client/assets/battle/portraits/',
                'crates/tbg-client/assets/i18n/', 'crates/tbg-core/src/content', 'crates/tbg-core/src/types')
    sources = manifest_sources()
    return any(path in exact or path.startswith(prefixes) or any(fnmatch.fnmatchcase(path, source) for source in sources)
               for path in paths)


def main():
    import release_scope
    release_scope.decide('static', relevant, 'Public static site input')


if __name__ == '__main__':
    main()
