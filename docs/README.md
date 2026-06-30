# TBG — Tupi Battle Grounds — Documentation

> Central index for all project documentation.  
> **Language:** 100% English (code, docs, commits, agent output).

---

## Quick links

| Document | Purpose |
|----------|---------|
| [GLOSSARY.md](GLOSSARY.md) | Ubiquitous language — use these terms exactly in code and APIs |
| [NEW-PROJECT-CHECKLIST.md](NEW-PROJECT-CHECKLIST.md) | Pre-implementation checklist |
| [API-CONTRACT.md](API-CONTRACT.md) | API surface (to be filled as endpoints are defined) |
| [USE-CASE-EXAMPLE.md](USE-CASE-EXAMPLE.md) | Use-case template |

---

## Architecture Decision Records (ADRs)

Non-trivial technical and product decisions, recorded before implementation.

| ADR | Title | Status |
|-----|-------|--------|
| [ADR-001](adr/ADR-001-real-time-competitive-design.md) | Real-time competitive design (Option A) | Accepted |
| [ADR-002](adr/ADR-002-nakama-self-hosted-infrastructure.md) | Nakama self-hosted on single VPS | Accepted |

---

## Design

| Document | Purpose |
|----------|---------|
| [design/brand-system.md](design/brand-system.md) | Brand identity, voice, visual system, UI kit |
| [design/character-creation-guide.md](design/character-creation-guide.md) | Unit design, mechanics-first workflow, 2D art pipeline |
| [design/procedural-character-generator.md](design/procedural-character-generator.md) | Procedural units — geometry and color 100% in code, Character Studio |
| [design/pilot-silhouette-checklist.md](design/pilot-silhouette-checklist.md) | Phase 01 pilot unit silhouette validation |

---

## Content (versioned)

| Path | Purpose |
|------|---------|
| [../content/characters/](../content/characters/) | Unit JSON definitions, card PNGs, previews (studio CRUD) |

---

## Architecture

| Document | Purpose |
|----------|---------|
| [architecture/infrastructure-overview.md](architecture/infrastructure-overview.md) | Full stack, Docker Compose, deploy, security, scaling path |

---

## Use cases

Place actor-driven flows in `use-cases/` — one file per use case, named `UC-NNN-short-title.md`.

---

## For coding agents

1. Read [AGENTS.md](../AGENTS.md) at session start.
2. Load task-specific rules: `./agent-harness/resolve-rules.sh <keywords>`
3. Use [GLOSSARY.md](GLOSSARY.md) terms exactly — do not invent synonyms.
4. When a decision conflicts with existing code, **documented decisions prevail**.
