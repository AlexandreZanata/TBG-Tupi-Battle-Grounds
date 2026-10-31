# P64 — Complete English/pt-BR presentation across game and API consumers

> **Planning only — 2026-10-31.** Owner request: investigate missing
> Portuguese localization throughout the game, identify affected files,
> and plan corrections with an execution prompt that ends in a PR to main.
> This branch performs the audit and writes the specification; it does not
> implement translations or claim runtime verification.

Planning branch: `codex/p64-i18n-completeness-plan`.
Execution branch: `codex/p64-i18n-completeness`.
Parent planning commit: `55659133e391149fca32c176a1ab278af69cf6c7`.

## 1. Read first

- [AGENTS.md](../../AGENTS.md) and current `docs/VALIDATION.md` when present;
  the owner's minimal-validation policy overrides historical broad checks.
- [Audit and affected-file inventory](../planning/P64-I18N-AUDIT.md).
- [437-path source manifest](../planning/P64-I18N-SOURCE-MANIFEST.csv).
- [P41](P41-i18n-pt-br.md): existing resolver, catalogs, font rules and
  incomplete content/HUD/standalone-page migration steps.
- [DECISIONS D-83](../DECISIONS.md#d-83--complete-localization-at-the-presentation-boundary--2026-10-31).
- [ARCHITECTURE §20](../ARCHITECTURE.md#20-localization-completion-p64-planned).
- [Executor prompt](../planning/P64-EXECUTION-PROMPT.md).

Read the relevant implementation files for each work front, not `archive/`.
No new game mechanic, balance value, backend translation service or framework
is required. Existing English-only code/comment/doc policy remains; actual
Portuguese product strings belong in the translation catalogs.

## 2. Outcome and scope

When pt-BR is selected, every ordinary player-facing text in the integrated
desktop game resolves through the existing catalog or an explicitly reviewed
display-name policy: lobby, Arcanas, champions and abilities, Shop, Augments,
Strongholds/Fortification, arena, accounts, queues/challenges, spectating,
ranking/profiles, API errors, result/rematch and accessibility attributes.
English remains complete and selectable. Switching language while a page or
battle is open updates its presentation without losing the player's state.

Include desktop browser, standalone ranking/profile routes and the active
native desktop shell. Freeze actual integration status before coding; native
files in the audit are not present on main automatically. If a surface is
not yet integrated, keep its correction front and deferred evidence explicit.
Keep phone code compiling; phone layouts/touch QA are outside the task.

The audit finds **catalog parity already passes**, while many consumers bypass
the catalog. Adding missing Portuguese values alone will not solve this.
Do not report a numeric completion percentage based on equal key counts.

### Requirement mapping

| ID | Contract | Inventory / acceptance |
|---|---|---|
| L01 | Every active content ID/form/ability resolves localized presentation | Audit A–D; T01/T02 |
| L02 | Battle, lobby and common states use catalog templates | Audit A/B/F; T03/T04 |
| L03 | API codes/fields become localized labels without changing machine identity | Audit E/F; T05 |
| L04 | Browser, native and standalone views agree and refresh on language changes | Audit E/F/§4; T04/T06 |
| L05 | Text interpolation, plural, numbers/dates and glyphs are correct | Audit §4; T02/T06/T07 |
| L06 | Future additions cannot silently bypass coverage checks | Audit §4; T01/T08 |
| L07 | Gameplay, persistence, credentials, IDs and accessibility remain correct | Audit §5; T03/T05/T07 |

## 3. Implementation contracts

### 3.1 IDs and content presentation

Extend `src/i18n/content.rs` or focused child modules using stable core IDs:
unit kind, form tuple, Faction, Archetype, Synergy, status/on-hit, Augment
ID/rarity, Stronghold kind/branch/tier, Arcana family/node and ability variants.
Prefer exhaustive enum matches or checked complete tables. Register active,
summon and staged availability explicitly; don't make future kinds playable
merely because they receive keys. Include active P59 content only if integrated.

Use meaningful families such as `unit.<stable_id>.name`, `.form.*`,
`ability.*`, `augment.*`, `stronghold.*`, `arcana.*`, `status.*`, `ranking.*`,
`account.*`, `queue.*` and `result.*`, adapting existing families rather than
duplicating them. Respect `check_pair`'s indexed-family constraints; a sparse
domain must not be disguised as a contiguous numeric family.

Proposed naming policy: translate gameplay descriptions, role/Faction names,
status/effect labels and generic form/tower terms. Preserve brand and genuine
character proper names where intended, with a documented per-name decision.
Do not silently leave an entire ability or tower description in English
under a proper-name exception. Review translations with the owner at content
consolidation; untranslated mechanics are not an acceptable glossary choice.

Do not change core content names in place, API field names, core enum values,
selectors, filenames, storage identifiers or persisted codes. JSON sources
where `name` doubles as identity (Faction/rarity/Stronghold) need an explicit
display resolver or display-key reference, not a translated identity field.

### 3.2 Mechanics and parameterized descriptions

Separate translated sentence templates from canonical numbers. Fetch stats,
costs, durations, thresholds, stack/cap values and prerequisites through the
same core helpers or already-validated content data. Never hardcode balance
numbers independently in two languages. Templates must have matching named
placeholders; ensure all are filled and none remain visible.

Port English sentence builders in tips/inspection/Evolution/Fortification/
Arcanas/Augments to full localized templates. Avoid partial English string
replacement and concatenation that assumes English grammar. In particular,
replace the native `Energy?` workaround with a real points-cost key.

Preserve numeric mechanics tests. Where existing tests compare English
source text, retain English expectations and add localized presenter
assertions instead of deleting useful coverage. Contract tests stay append-only
unless their guarded canon/decision is legitimately changed in the same commit.

### 3.3 API, errors and user identity

The server keeps language-independent data and current cache/HTTP semantics.
Client maps profile Faction names, W/L/D, EndReason strings, mode/status slugs,
AccountError/Rejection and HTTP failure categories to local messages. Support
historical codes still emitted by stored records; unknown codes get a safe
localized fallback. Never show raw response/error text as a translated label.
Technical detail may be logged separately in English.

Replace English-message-prefix error classification with stable local error
categories. Preserve parsing, response limits and security. No SQL migration,
server-side locale parameter or protocol change is expected for this task.
If a genuine missing semantic discriminator requires a contract change,
document the exact need and update every consumer compatibly; do not use it
as a reason to rewrite API schemas or translated persistence.

Keep player names and credentials unchanged. Built-in Guest/You/Training Bot
need explicit semantic identity, not string substitution on any player's
chosen name. Do not translate values merely because they equal an English word.
Use textContent or safe text nodes for translated text and API arguments;
do not introduce innerHTML interpolation or unescaped usernames.

### 3.4 Runtime language changes and shared surfaces

Use existing `Lang`, generated Msg tables and `TBG_I18N`; no second runtime.
Move native ranking's bilingual `words()` pairs into the shared catalogs.
Remove that helper only after all its consumers migrate.

Include language in cached reports/projection invalidation. Rebuild text on
language or source-data changes, not on every frame by habit. Refresh newly
mounted dialogs, API results, tooltip/alt/title/aria text, result/rematch,
form errors and in-world labels. Switching with an unchanged MatchView must
still update content. Preserve scroll/filter/page, draft/ranks, selected
champion, input values/focus, match seed and pending actions.

Make async language changes generation-aware so older requests cannot replace
the newest selection. Use consistent startup/query/storage precedence from
P41 on every entry point. Standalone ranking/profile pages need the resolver
and catalog readiness under their existing CSP; do not relax script policy
with unsafe-inline or use a second translated HTML copy.

Use locale-aware numbers, percentages and dates at the display edge. Cache
Intl.NumberFormat by locale **and precision/options**, not locale alone.
Rust presenters use existing Lang helpers or narrow equivalents; numeric
API values remain numeric. Preserve date/timezone meaning and missing-value
states. Avoid accidental double conversion of Energy, ticks or basis points.

### 3.5 Coverage guards and bounded exceptions

Extend current parity/placeholder/charset checks with content-ID coverage,
literal referenced-key checks and enumerated dynamic-family checks. Account
for static fallback HTML, test fixtures, developer logs and allowed names.
Do not call a prefix such as `arenas.` a missing literal key.

Use a small reviewed exception list with path/reason for unlocalized strings
that are not normal UI: branding, proper names, technical diagnostics,
selectors and test messages. A regex for every English word is not a reliable
language detector and must not become an unmaintainable build blocker.
Coverage must catch a newly added active champion/form/Augment without a
presentation mapping. Identical EN/PT values require review, not automatic
failure. Keep fallback diagnostics useful without exposing raw keys to users.

## 4. Execution order

**Step 0 — refresh inventory.** Start from current main, inspect active native
integration and reconcile with the two audited baselines. Update each audit
row: applicable, already fixed, confirmed, excluded with reason, or deferred.
Freeze glossary/name policy, exact active content IDs and mounted entry points.
Do not blindly apply old line numbers or import unrelated branches.

**Step 1 — shared content and formatters.** Add paired keys, ID resolvers,
parameterized mechanics text and formatter/error categories. Write focused
coverage tests and keep current English/Portuguese catalog validation.

**Step 2 — battle and content pages.** Migrate champions/abilities/inspection/
Evolution, Shop/Augments/council, towers/Fortification and Arcanas. Fix both
producers and browser/native consumers. Preserve IDs and numeric equality.

**Step 3 — account, queue and results.** Migrate forms, warnings/toasts,
network/update messages, local identities, challenge/share, surrender,
result/rematch and common accessible attributes. Classify platform diagnostics.

**Step 4 — ranking/API/standalone.** Localize profile/result/reason/Faction/
presence/mode/error presentation, migrate native bilingual literals, wire
standalone routes and localize dates/percentages/plurals.

**Step 5 — switching and regression guard.** Complete invalidation, async
load ordering, dynamic DOM translation, fonts/glyphs and coverage exceptions.
Coordinate new P60/P61/P62/P63 screens if they have landed, without treating
their unimplemented plans as existing product code.

**Step 6 — consolidation and PR.** One reviewer runs the deduplicated affected
tests and desktop journeys once. Update P41/P64 roadmap/report truthfully,
publish pertinent commits and open/update a PR to main under the owner's
explicit execution authorization in the companion prompt. Do not merge or
deploy. Missing applicable acceptance keeps the PR Draft or explicitly pending.

## 5. Acceptance and validation

- **T01 Content coverage:** enumerate all active IDs, available forms,
  abilities, Augments/rarities, tower branches and Arcana nodes. Each has
  complete EN/PT presentation, unchanged numeric parameters and a reviewed
  naming policy. A newly active unmapped kind fails the focused guard.
- **T02 Description correctness:** compare representative damage, heal,
  status, summon, shield and conditional effects to core; check both
  Evolution branches, tower passives/tiers and Arcana prerequisites. Inspect
  all remaining mappings. No unresolved tokens or altered values.
- **T03 Battle/common flow:** Portuguese and English Shop, inspect,
  Evolution, War Council choices/wait/timeout, Fortification, accounts,
  connecting/queue/challenge, surrender, victory/defeat/draw and rematch.
  Same intention IDs, Energy, outcome and identity before/after switching.
- **T04 Live refresh:** switch EN→PT→EN on an open Arcana editor, tower page,
  ranking/profile, active inspect/council and Result. No new network view is
  needed to translate static content. Preserve drafts, scroll, input/focus
  and match progress. Simulate rapid switches and delayed responses.
- **T05 API fixtures:** profiles with every supported Faction and historical
  W/L/D/reason, all mode/status slugs, guest and actual user named Guest/You,
  missing/unknown codes, empty data, malformed payload, timeout/offline,
  unauthorized/not-found/rate-limit/version mismatch. Never expose raw
  machine codes, credentials or backend English error messages as UI text.
- **T06 Shared surfaces:** browser lobby/HUD, native equivalents and direct
  standalone ranking/player links respect selection/storage/query rules;
  preserve CSP, links and safe rendering. Validate locale-aware numbers,
  multiple precisions in one session, percentages, dates and 0/1/many plurals.
- **T07 Readability/accessibility:** accents, allowed glyphs, title/alt/aria,
  tooltips, live/validation announcements, long Portuguese text and desktop
  resizing at existing supported sizes. Fix wrapping/space, not shrinking
  everything to unreadable text. Keep keyboard access and icons intact.
- **T08 Regression prevention:** missing key, placeholder drift, missing
  content mapping and invalid glyph fail; dynamic prefixes and approved
  names do not. Plain-text API rendering remains safe against markup.

Current focused targets to reuse: client `i18n::tests`, `ui::glyphs`,
content-data tests and JS tests under `crates/tbg-client/tests`. Planned
target: a client test module filtered by `i18n_content_coverage`, and a
focused JS localization test covering presenter, switching and API fixtures.
Inventory actual names at Step 0; do not pretend planned tests already exist.

Future consolidation command example, not executed by this plan:

```bash
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" \
  env CARGO_BUILD_JOBS=2 cargo test -p tbg-client --bin tbg-client i18n
```

On fronts use formatting and at most one cheap scoped cached check; queue
tests for consolidation. Every compiling validation uses the shared
nonblocking lock with at most two jobs. Busy lock/cold cache means defer,
not wait, poll or bypass. No automatic workspace suite, full gallery,
balance run, benchmark or release build. Browser compatibility and native
OS coverage are separate evidence; no phone/touch QA is required.

## 6. Done, rollout and report

Deliver paired catalogs, migrated presentation, reviewed exclusions,
focused regressions and a refreshed file inventory. All applicable audit
rows and L01–L07 must have evidence; no raw-key fallback or untranslated
mechanical sentence is accepted as an unrecorded exception. English and
Portuguese both pass focused flows. Owner reviews wording/readability.

No game-rule change, database migration or server locale deployment is
needed by default. Deploy code and catalogs as a compatible client asset
set using current versioning; handle cache mismatch safely. Rollback uses
a normal revert/coordinated asset version and preserves the saved locale.
Do not reset player accounts/presets to fix a translation cache issue.

Executor report: integration SHA, updated audit rows, key/content counts
(not completeness percentages), glossary exceptions, actual commands and
results, deferred checks/reasons, desktop screenshots/evidence, API fixture
coverage, translation review, commit list and PR URL/base main.

Planning validation and limitations are recorded in the audit and delivery
report. Implementation, runtime verification and translation approval remain
pending; this branch contains documentation only.
