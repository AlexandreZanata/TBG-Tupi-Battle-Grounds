# Rig session archives (versioned skeleton placements)

Manual joint placements from **Rig Studio** are stored here so they are **not lost** when `.local/` is cleared.

| Unit | Body class | Bones | Notes |
|------|------------|-------|-------|
| [demo_guard_01](demo_guard_01/) | HumanoidHeavy | 21 | First manual Rig Studio pass (2026-06-30) |

## Files per unit

- **`bones.json`** — canonical `TBG_*` head/tail/parent (commit this)
- **`manifest.json`** — `unitId`, `bodyClass`, `archivedAt`, `boneCount`

## Commands

```bash
# After Save in Rig Studio
python3 tools/experimental/auto_rig/archive_rig_session.py --id demo_guard_01

# Bake walk on this skeleton (needs .local session.blend from prep)
PYTHONPATH=tools python3 -c "
from tools.experimental.auto_rig.run_session_walk_test import run_session_walk_test
print(run_session_walk_test('demo_guard_01'))
"
```

Full format spec: [docs/design/rig-studio-bones-format.md](../../docs/design/rig-studio-bones-format.md).
