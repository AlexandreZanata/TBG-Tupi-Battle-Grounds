# Archive Manifest — Legacy TBG Preservation Record

> The `archive/` directory is a **read-only backup** of the pre-2.0 TBG project
> (Unity client, Nakama-typed design docs, Python/Blender character pipeline).
> It is **not** imported by any active build, test, or CI path.
> Git history remains the primary preservation mechanism; this directory is the
> secondary, human-browsable mechanism.

## Snapshot references

| Item | Value |
|------|-------|
| Pre-reboot backup branch | `backup/pre-tbg20-20261027` |
| Pre-reboot annotated tag | `pre-tbg20-20261027` |
| Original commit at reboot | `00d415a` — "chore: pivot to harness-only repo and strip archived binary assets" |
| Archive created | 2026-07-01 (full project pivot; see `archive/README.md`) |
| Manifest written | 2026-10-27 (TBG 2.0 repository reboot, Phase 00) |

Restore a single file without touching the archive:

```bash
git show pre-tbg20-20261027:<path> > /tmp/restore-target
# example: git show pre-tbg20-20261027:archive/docs/GLOSSARY.md
```

## What lives here

| Archived path | Original active path (pre-2026-07-01) | Contents | Status |
|---------------|----------------------------------------|----------|--------|
| `archive/docs/` | `docs/` | ADRs 001–007, `GLOSSARY.md`, `architecture/infrastructure-overview.md`, design specs, research notes, `API-CONTRACT.md` / `USE-CASE-EXAMPLE.md` templates | Preserved; game-design rules migrated to `docs/game-design/` with traceability |
| `archive/content/characters/` | `content/characters/` | JSON definitions, animation/behavior/material contracts, 17 small show-case/walk `.glb` clips | Preserved; large binaries (MB-Lab, Mixamo, seed GLBs, library exports) stripped 2026-07-01, see below |
| `archive/tools/` | `tools/` | Assembly Studio, Character Studio, Blender scripts, MB-Lab glue, auto-rig, mesh/material pipelines | Preserved, retired from active use |
| `archive/tests/` | `tests/` | Assembly/behavior/Blender/character-studio contract tests (Python/pytest) | Preserved, not run by active CI |
| `archive/unity/` | `unity/` | Unity client placeholder + `com.tbg.character-library` package | Preserved, retired |
| `archive/scripts/` | `scripts/` | `verify.sh`, studio/MB-Lab/Mixamo helper scripts | Preserved, retired |
| `archive/character_studio/` | `character_studio/` | Legacy Rig / Animation / Arena web UI (pre-pivot) | Preserved, retired |
| `archive/pytest.ini` | `pytest.ini` | Former test config | Preserved |
| `archive/README.md` | — | Pivot notes written 2026-07-01 | Preserved, unchanged |
| `archive/THIRD_PARTY_NOTICES.md` | — | MB-Lab (GPL-3.0/AGPL-3.0) + Mixamo (Adobe terms) notices for removed binaries; Ponytail MIT notice | Preserved, unchanged |

## What was already stripped (before this reboot)

Removed 2026-07-01 to shrink the repo (~3.5 GB), per `archive/README.md`:

- `third_party/MB-Lab/` (~279 MB) — re-vendor upstream tag `1_8_1` if needed.
- `content/characters/seed-glb-pipeline/` — seed GLB units (archived path kept as empty record).
- `content/characters/imports/mixamo/` — Mixamo FBX downloads (Adobe account terms).
- `content/characters/library/` — assembled GLB exports.
- `content/characters/mblab/` — MB-Lab blend templates.
- `.local/tmp/`, `.local/rig-studio/` — local Blender sessions (untracked, never committed).

## What moved during the 2.0 reboot (2026-10-27)

**Nothing was moved out of `archive/`.** The reboot only:

1. Fixed the `agent-harness` / `agent-rules` symlinks (broken absolute
   `/home/iiii/WebstormProjects/...` targets → relative `.agent-harness/...`).
2. Added new top-level scaffolding beside `archive/`: `Cargo.toml`,
   `rust-toolchain.toml`, `crates/`, `content/`, `tools/asset-factory/`,
   `docs/`, `tests/`, `deploy/`.
3. Rewrote root `README.md` and `AGENTS.md` for the TBG 2.0 identity
   (previous harness-only text superseded; full history in git).

No legacy source, doc, or test file was deleted, edited, or relocated.

## Why components were retired

| Retired technology / practice | Reason (authoritative: game rules kept, tech retired) |
|-------------------------------|--------------------------------------------------------|
| Unity client (C#) | Replaced by Bevy + Rust/WASM browser-first client; Unity licensing/build weight unjustified for solo scope |
| Nakama self-hosted server (ADR-002) + TypeScript match module | Replaced by minimal Rust (Axum + Tokio) authoritative server on the same single-VPS model; Nakama ops/custom-module weight retired, single-server principle kept |
| PostgreSQL (Nakama backend) | Replaced by SQLite for initial persistence; migration path kept if CCU demands it |
| Per-character Blender/MB-Lab/Mixamo rigging pipeline (ADRs 003–006) | Retired after measured ~8% auto weight-gate pass rate (ADR-007); future assets use modular/shared-rig procedural pipeline under `tools/asset-factory/` |
| Python studio/test tooling (pytest) | Retired from active CI; contracts inform `tbg-assets` validation schemas |
| Store IAP (StoreKit / Play Billing) | Deferred; cosmetics-only monetization principle kept, provider integration postponed to post-beta |

## Large binaries and licensing

- No Git LFS references exist in this repository (verified 2026-10-27).
- No file under `archive/` exceeds 1 MB (verified 2026-10-27).
- Remaining 17 `.glb` clips under `archive/content/characters/animations/clips/`
  are small showcase/walk samples kept as format references only.
- Third-party attributions kept in `archive/THIRD_PARTY_NOTICES.md`
  (MB-Lab GPL-3.0/AGPL-3.0, Mixamo Adobe terms) and root
  `THIRD_PARTY_NOTICES.md` (Ponytail MIT). Nothing under an active
  build path depends on GPL-licensed material.

## Recovery guide

| Need | Command |
|------|---------|
| Browse legacy docs | `ls archive/docs/adr/ archive/docs/design/` |
| Read legacy glossary | `archive/docs/GLOSSARY.md` |
| Read legacy game rules | `archive/docs/adr/ADR-001-real-time-competitive-design.md` |
| Read legacy infra | `archive/docs/architecture/infrastructure-overview.md` |
| List a legacy unit definition | `archive/content/characters/definitions/` |
| Full pre-reboot tree | `git checkout pre-tbg20-20261027` (detached) or browse the `backup/pre-tbg20-20261027` branch |
| Run legacy Python tests | `pip install -r archive/tools/requirements.txt && pytest -c archive/pytest.ini archive/tests/` (legacy only, not part of 2.0 CI) |
