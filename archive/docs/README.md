# TBG — Tupi Battle Grounds — Documentation

> Central index. **Active:** Phase 05 modular character assembler.

---

## Quick links

| Document | Purpose |
|----------|---------|
| [GLOSSARY.md](GLOSSARY.md) | Ubiquitous language |
| [design/modular-character-assembler.md](design/modular-character-assembler.md) | **Current** character production pipeline |
| [adr/ADR-007-modular-character-assembler.md](adr/ADR-007-modular-character-assembler.md) | Pipeline pivot decision |
| [API-CONTRACT.md](API-CONTRACT.md) | API surface |
| [architecture/infrastructure-overview.md](architecture/infrastructure-overview.md) | Stack + deploy |

---

## Architecture Decision Records

| ADR | Title | Status |
|-----|-------|--------|
| [ADR-001](adr/ADR-001-real-time-competitive-design.md) | Real-time competitive design | Accepted |
| [ADR-002](adr/ADR-002-nakama-self-hosted-infrastructure.md) | Nakama self-hosted VPS | Accepted |
| [ADR-007](adr/ADR-007-modular-character-assembler.md) | Modular assembler (Phase 05) | Accepted |

Legacy ADRs (003–006): [`../archive/docs/adr/`](../archive/docs/adr/)

---

## Content

| Path | Purpose |
|------|---------|
| [../content/characters/library/](../content/characters/library/) | Modular bases, parts, exports |
| [../content/characters/definitions/](../content/characters/definitions/) | Unit JSON schemas |
| [../archive/content/](../archive/content/) | Legacy seed GLB + shared profiles |

---

## Tools

| Path | Purpose |
|------|---------|
| [../tools/README.md](../tools/README.md) | Active tools index |
| [../tools/assembly_studio/](../tools/assembly_studio/) | Blender assembly (rebuild) |
| [../archive/tools/](../archive/tools/) | Legacy tooling (backup) |

---

## For coding agents

1. Read [AGENTS.md](../AGENTS.md)
2. Tasks: `.local/phases/05-modular-character-assembler/TASKS.md`
3. `./agent-harness/resolve-rules.sh blender api`
