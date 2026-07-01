# Agent Harness — TBG workspace

This repository is now **harness-only**. All prior TBG game/character pipeline code lives in [`archive/`](archive/) — not used by CI or new work.

## What is active

| Path | Purpose |
|------|---------|
| [AGENTS.md](AGENTS.md) | Entry point for coding agents |
| [agent-harness/](agent-harness/) | Install + resolve scripts (symlink → `.agent-harness/harness/`) |
| [agent-rules/](agent-rules/) | Rule library (symlink → `.agent-harness/rules/`) |
| [.cursor/rules/](.cursor/rules/) | Cursor always-on rules |
| [.local/](.local/) | Private tasks and overrides (gitignored) |
| [archive/](archive/) | Legacy TBG project backup |

## Quick start

```bash
pip install -r agent-harness/requirements.txt
./agent-harness/rules-path.sh
./agent-harness/resolve-rules.sh api auth domain
```

## Agent tasks

1. Read [AGENTS.md](AGENTS.md)
2. Open `.local/IMPLEMENTATION-PLAN.md` for current direction
3. Pick a phase under `.local/phases/` → `README.md` → `TASKS.md` → `CURSOR-PROMPT.md`
4. Copy `_example-task/` when starting a new phase

## Archive

Full TBG character studio, Unity client, content, tests, and docs:

```bash
ls archive/
# content/ docs/ tools/ tests/ unity/ scripts/ third_party/ .local/ …
```

Restore a single file:

```bash
cp archive/tools/assembly_studio/README.md ./   # example only — wire back only with a new task
```

## License

See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) and `.agent-harness/LICENSE`.
