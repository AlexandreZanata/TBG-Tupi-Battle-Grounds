"""Manifest selection and visual-only budgets for the three generated art sets."""
import re

TOTAL_BUDGETS = {'arcana': 3_000_000, 'augments': 4_000_000, 'strongholds': 2_000_000}
FOLDERS = {'arcana': 'arcana', 'augments': 'augments', 'strongholds': 'stronghold'}
MANIFESTS = {'arcana': 'arcana.json', 'augments': 'augments.json', 'strongholds': 'strongholds.json'}


def entries(data, name):
    if name == 'arcana':
        rows = [(f['emblem'], 160, 250_000, 70_000, False) for f in data['families']]
        rows += [(f['banner'], 900, 400_000, 112_000, False) for f in data['families']]
        rows += [(n['asset'], 72, 150_000, 42_000, True) for n in data['nodes']]
    elif name == 'augments':
        rows = [(r['frame'], 300, 250_000, 35_000, False) for r in data['rarities']]
        rows += [(a['asset'], 200, 150_000, 18_000, False) for a in data['augments']]
    else:
        rows = [(k['portrait'], 160, 250_000, 42_000, True) for k in data['kinds']]
        rows += [(k['banner'], 450, 400_000, 70_000, False) for k in data['kinds']]
        rows += [(b['asset'], 72, 150_000, 25_000, True) for k in data['kinds'] for b in k['branches']]
    seen = set()
    for ref, display, ceiling, target, lossless in rows:
        filename = ref['svg']
        if not re.fullmatch(r'[a-z0-9-]+\.svg', filename) or filename in seen:
            raise ValueError('Unsafe or reused SVG: ' + filename)
        seen.add(filename)
        yield dict(name=filename[:-4], ref=ref, display_width=display,
                   budget=ceiling, target=target, lossless=lossless)
