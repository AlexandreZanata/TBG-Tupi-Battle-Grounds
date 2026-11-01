#!/usr/bin/env python3
"""Require reviewed public notes for every gameplay revision entering main."""
import argparse
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT, text=True).strip()


def require(condition, message):
    if not condition:
        raise SystemExit(message)


def text_pair(value, label, limit=1600):
    require(isinstance(value, dict), f'{label}: expected bilingual text')
    for lang in ['en', 'pt-BR']:
        require(isinstance(value.get(lang), str) and 0 < len(value[lang].encode('utf-8')) <= limit,
                f'{label}: invalid {lang} text')


def source_hash():
    paths = [line.split()[1] for line in (ROOT/'crates/tbg-core/ruleset-sources.txt').read_text().splitlines() if line.startswith('watch ')]
    result = 0xcbf29ce484222325
    for path in sorted(paths):
        data = path.encode()+b'\0'+(ROOT/'crates/tbg-core/src'/path).read_bytes()+b'\xff'
        for byte in data:
            result = ((result ^ byte)*0x100000001b3) & ((1 << 64)-1)
    return f'{result:016x}', paths


def validate_changes(note):
    changes = note.get('changes', [])
    require(0 < len(changes) <= 128, 'Each patch needs 1..128 concrete changes')
    for change in changes:
        require(all(key in change for key in ['subject', 'field', 'before', 'after', 'reason']), 'Incomplete before/after record')
        require(change['before'] != change['after'], 'A change must differ before and after')
        for key in ['subject', 'field', 'reason']:
            text_pair(change[key], note['id'])
        for key in ['before', 'after']:
            if isinstance(change[key], dict):
                text_pair(change[key], note['id'])

def validate_note(note, ids):
    require(re.fullmatch(r'[a-z0-9-]{1,64}', note.get('id', '')), 'Invalid patch id')
    require(note['id'] not in ids, 'Duplicate patch id')
    ids.add(note['id'])
    text_pair(note.get('title'), note['id'], 180)
    text_pair(note.get('summary'), note['id'])
    require(note.get('status') in ['candidate', 'preview', 'released'], 'Invalid public release status')
    require(re.fullmatch(r'[0-9a-f]{16}', note.get('ruleset_hash', '')), 'Missing ruleset source hash')
    validate_changes(note)
    for lang in ['en', 'pt-BR']:
        sections = note.get('sections', {}).get(lang, [])
        require(0 < len(sections) <= 32, f'Missing or excessive {lang} release detail')
        for section in sections:
            require(isinstance(section.get('title'), str) and 0 < len(section['title'].encode('utf-8')) <= 180, 'Invalid section title')
            require(isinstance(section.get('body'), str) and 0 < len(section['body'].encode('utf-8')) <= 1600, 'Invalid section body')
    document = ROOT/note.get('document', '')
    require(document.is_file() and document.resolve().is_relative_to(ROOT/'docs/patches'), 'Missing patch document')
    report = ROOT/note.get('evidence', {}).get('report', '')
    require(report.is_file() and report.resolve().is_relative_to(ROOT/'docs'), 'Missing evidence report')
    if note.get('kind') == 'balance':
        require(note['evidence'].get('accepted') is True, 'Numeric balance notes require accepted held-out evidence')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--base', default='origin/main')
    args = parser.parse_args()
    catalog_path = ROOT/'docs/patches/catalog.json'
    require(catalog_path.stat().st_size <= 262144, 'Patch catalog exceeds the public feed budget; paginate before expanding it')
    catalog = json.loads(catalog_path.read_text())
    require(catalog.get('schema') == 1, 'Unsupported patch catalog schema')
    notes = catalog.get('notes', [])
    require(0 < len(notes) <= 50, 'Public history supports 1..50 entries; add pagination before entry 51')
    ids = set()
    for note in notes:
        validate_note(note, ids)
    version_file = 'crates/tbg-core/src/ruleset_version.rs'
    current = int(re.search(r'RULESET_VERSION: u32 = (\d+)', (ROOT/version_file).read_text())[1])
    fingerprint, watched = source_hash()
    matching = [n for n in notes if n.get('ruleset_version') == current and n.get('ruleset_hash') == fingerprint]
    require(matching, f'No patch note identifies ruleset {current} / {fingerprint}')
    changed = set(git('diff', '--name-only', args.base).splitlines())
    gameplay = {f'crates/tbg-core/src/{path}' for path in watched}
    if changed & gameplay:
        previous = subprocess.run(['git', 'show', f'{args.base}:{version_file}'], cwd=ROOT, text=True, capture_output=True)
        found = re.search(r'RULESET_VERSION: u32 = (\d+)', previous.stdout)
        before = int(found[1]) if found else 0
        require(current > before, 'Gameplay changes require a new RulesetVersion and matching patch notes')
    print(f'Patch notes OK: {len(notes)} entries; current ruleset {current} / {fingerprint}')


if __name__ == '__main__':
    main()
