# P30-D — Deterministic server performance investigation

**Owner request:** 2026-10-30, deep server performance investigation.
**Baseline:** `d06279aa5287b67efb24e224c4710db55a8ca739`.
**Branch:** `phase/p30d-server-performance`.
**Scope:** experimental benchmarks, exact-behavior prototypes, research,
and a reviewable implementation plan. No merge, push, deploy, production
configuration changes, gameplay changes, or balance changes. The public
VPS is read-only; stress and capacity experiments are local only.

This supplements P30-B rather than silently expanding its server-only
contract. It covers the current 25-kind roster, support abilities,
Strongholds, Augments, Arcanas, snapshots, and authoritative input paths.

## Read first

- `AGENTS.md`; owner investigation attachment (2026-10-30).
- `docs/ARCHITECTURE.md` sections 4, 5, 10, 11, 12.
- P10 report; P16-B performance report; P30-B spec.
- Current core tick, targeting, support, effects, state, view, and protocol.
- Server match loop, connection routing, channels, metrics, persistence.

## Steps and checks

1. Preserve baseline provenance, existing test failures, environment and
   historical methodology. Re-run `army_load` in release. Check that no
   production artifact changes; record historical limitations explicitly.
2. Add a small reproducible native benchmark using existing dependencies.
   Measure 10/20/50/100/200/300/500/1000 total conventional units, multiple
   seeds/compositions, tick distributions, allocations, views, encoding,
   state sizes, and bytes. Separate instrumented runs from timing runs.
   Synthetic setup and top-ups are outside timed ticks and documented.
3. Audit algorithmic complexity and operation counts, profile available
   tools, and consult primary technical sources. Do not reopen P10's
   rejected allocator/encoding experiments without changed conditions.
4. Test one isolated optimization at a time, only after its baseline is
   preserved. Compare complete serialized state, events, and both views
   at every tick of deterministic adversarial traces; preserve live
   movement/death ordering and all targeting ties. Keep only reproducible
   improvements outside observed noise, with no correctness regression.
5. Investigate sequence-zero/duplicate commands and command-drain work
   bounds locally. Record confirmed defects separately from performance
   wins; never change a contract test or silently change protocol rules.
6. Re-measure local server capacity with load generation outside the
   server CPU affinity and resource constraints, where supported. Report
   unavailable counters/limits honestly. VPS measurements remain light,
   read-only observations of existing services.
7. Complete the report below and a concise, evidence-based follow-up plan.
   Run formatting, clippy, core/server tests, and WASM checks. Client
   builds/browser validation are needed only if client source changes.
   Commit logical steps independently; stop for review without deploying.

## Acceptance

- Historical claims identified by introducing commit, method, and known
  hardware limitations; current measurements do not inherit those claims.
- Reproducible release commands and compact raw benchmark results kept.
- Original baseline and each candidate measured independently; algorithms,
  operation counts, differential traces, memory and bandwidth documented.
- Full serialized-state/event/view equality, including deaths, spawns,
  healing, carries, evolutions, pauses, and Stronghold interactions.
- No rejected speculative code/dependencies left behind.
- Audit distinguishes measurements, external evidence and untested ideas.
- Tests/security limitations and unavailable tools reported exactly.
- No modification to the live VPS, shared services, or other projects.

## Phase report

In progress. Measurements and acceptance results will be appended here.

### Deviations

The owner's attachment explicitly prohibits deployment during this
investigation. Its specific restriction supersedes the repeated deploy
introduction; the already-published release stays unchanged.
