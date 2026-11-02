#!/usr/bin/env python3
"""Select game-browser releases without publishing admin, landing or mobile frontends."""


def relevant(paths):
    exact = {'Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', 'docs/patches/catalog.json',
             '.github/workflows/web-release.yml', 'scripts/package-web.py', 'scripts/web-release-changes.py',
             'scripts/release_scope.py', 'scripts/install-web-build-tools.sh'}
    return any(p in exact or p.startswith(('crates/tbg-client/', 'crates/tbg-core/')) for p in paths)


def main():
    import release_scope
    release_scope.decide('web', relevant, 'Game browser source')


if __name__ == '__main__':
    main()
