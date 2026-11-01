#!/usr/bin/env python3
"""Prove the two CI test lanes together equal `cargo test --workspace`.

The `test` lane runs every workspace member except the balance lab and the
`balance` lane runs only the lab, on separate runners. Cargo unifies features
across the packages it builds, so the split keeps the exact feature set of a
whole-workspace build only while the lab adds nothing but `tbg-core` features.
This guard recomputes both lane scopes from the manifests and fails when
`.github/workflows/ci.yml` drifts from them, instead of silently testing a
different configuration.
"""
import glob
from pathlib import Path
import re
import sys
import tomllib

ROOT = Path(__file__).resolve().parents[1]
WORKFLOW = ROOT/'.github/workflows/ci.yml'
LAB = 'tbg-balance'
CORE = 'tbg-core'
DEP_TABLES = ('dependencies', 'dev-dependencies', 'build-dependencies')
PLAIN_PATH_DEP = {'path', 'features', 'version'}
CARGO_TEST = re.compile(r'\bcargo\b.*\btest\b')


def dependency_specs(manifest):
    """Every (package name, spec) pair, including target-specific tables."""
    tables = [manifest.get(name, {}) for name in DEP_TABLES]
    for target in manifest.get('target', {}).values():
        tables += [target.get(name, {}) for name in DEP_TABLES]
    pairs = []
    for table in tables:
        for key, spec in table.items():
            name = spec.get('package', key) if isinstance(spec, dict) else key
            pairs.append((name, spec))
    return pairs


def members(root):
    """Package name -> parsed manifest for every workspace member."""
    workspace = tomllib.loads((root/'Cargo.toml').read_text())['workspace']
    found = set()
    for pattern in workspace.get('members', []):
        found.update(path for path in glob.glob(str(root/pattern)) if Path(path).is_dir())
    manifests = {}
    for directory in sorted(found):
        manifest = tomllib.loads((Path(directory)/'Cargo.toml').read_text())
        manifests[manifest['package']['name']] = manifest
    return manifests


def lab_core_features(manifest):
    """The lab's own tbg-core features; anything else it depends on breaks the split."""
    deps = dependency_specs(manifest)
    if [name for name, _ in deps] != [CORE]:
        raise ValueError(f'{LAB} must depend on {CORE} only (found {sorted(n for n, _ in deps)}); '
                         f'excluding it from the test lane would change feature unification')
    spec = deps[0][1]
    if not isinstance(spec, dict) or not set(spec) <= PLAIN_PATH_DEP or 'path' not in spec:
        raise ValueError(f'{LAB} must use a plain path dependency on {CORE}')
    return set(spec.get('features', []))


def others_core_features(manifests):
    """tbg-core features enabled by every member except the lab."""
    features = set()
    for name, manifest in manifests.items():
        if name == LAB:
            continue
        for dep, spec in dependency_specs(manifest):
            if dep == CORE and isinstance(spec, dict):
                features.update(spec.get('features', []))
    return features


def feature_args(features):
    return ['--features', ','.join(f'{CORE}/{f}' for f in sorted(features))] if features else []


def expected_scopes(manifests):
    """The cargo package/feature arguments each lane must use."""
    if LAB not in manifests or CORE not in manifests:
        raise ValueError(f'{LAB} and {CORE} must both be workspace members')
    test = ['--workspace', '--exclude', LAB] + feature_args(lab_core_features(manifests[LAB]))
    balance = ['-p', LAB] + feature_args(others_core_features(manifests))
    return {'test': ' '.join(test), 'balance': ' '.join(balance)}


def workflow_scopes(text):
    """Job id -> TEST_SCOPE value, read line by line (no YAML dependency)."""
    scopes, job, in_jobs = {}, None, False
    for line in text.splitlines():
        if re.match(r'^\s*#', line):
            continue
        if re.match(r'^\S', line):
            in_jobs, job = line.startswith('jobs:'), None
            continue
        heading = re.match(r'^  ([A-Za-z0-9_-]+):\s*$', line)
        if in_jobs and heading:
            job = heading.group(1)
            continue
        scope = re.match(r'^\s+TEST_SCOPE:\s*(.+?)\s*$', line)
        if in_jobs and job and scope:
            scopes[job] = scope.group(1)
    return scopes


def problems(expected, text):
    """Every way the workflow differs from the expected lane scopes."""
    found = workflow_scopes(text)
    issues = [f'{lane}: TEST_SCOPE is {found.get(lane)!r}, expected {scope!r}'
              for lane, scope in sorted(expected.items()) if found.get(lane) != scope]
    issues += [f'{lane}: unexpected TEST_SCOPE lane' for lane in sorted(set(found) - set(expected))]
    commands = [line.strip() for line in text.splitlines()
                if CARGO_TEST.search(line) and not line.strip().startswith('#')]
    issues += [f'cargo test without the checked scope: {line}' for line in commands if '$TEST_SCOPE' not in line]
    if not commands:
        issues.append('no cargo test invocation found')
    return issues


def main():
    try:
        expected = expected_scopes(members(ROOT))
    except (ValueError, KeyError, OSError, tomllib.TOMLDecodeError) as error:
        print(f'Test lane split is unsafe: {error}', file=sys.stderr)
        return 1
    issues = problems(expected, WORKFLOW.read_text())
    if issues:
        print('\n'.join(issues), file=sys.stderr)
        return 1
    print(f'Test lanes cover the workspace with unified features: {expected}')
    return 0


if __name__ == '__main__':
    sys.exit(main())
