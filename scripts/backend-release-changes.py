#!/usr/bin/env python3
"""Keep frontend-only and documentation merges from restarting the game backend."""


def relevant(paths):
    exact = {'Cargo.toml', 'Cargo.lock', 'rust-toolchain.toml', 'docs/patches/catalog.json',
             '.github/workflows/backend-release.yml', 'scripts/package-backend.py',
             'scripts/backend-release-changes.py', 'scripts/release_scope.py'}
    return any(p in exact or p.startswith(('crates/tbg-core/', 'crates/tbg-server/')) for p in paths)


def main():
    import release_scope
    release_scope.decide('server', relevant, 'Backend source')


if __name__ == '__main__':
    main()
