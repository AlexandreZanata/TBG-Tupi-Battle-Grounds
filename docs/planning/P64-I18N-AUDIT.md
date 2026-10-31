# P64 — English/pt-BR localization audit

> Static source audit, 2026-10-31. This is evidence for a correction plan,
> not an implementation or a claim that every runtime screen was exercised.
> See [P64](../phases/P64-i18n-completeness.md) and the
> [complete scanned-source manifest](P64-I18N-SOURCE-MANIFEST.csv).

## 1. Baselines and scope

- Main-derived browser code: `9008f117c4289122ddff978c5d25720d2bd79687`;
  inspected in the planning tree at `55659133e391149fca32c176a1ab278af69cf6c7`,
  whose intervening changes are documentation only.
- Native desktop code: `926b0c9ab82cb2a7532fffb68d6ea3299c664ef4`, branch
  `codex/native-desktop-preview`. Its dirty AGENTS/VALIDATION documentation
  was not incorporated. Relevant audited client sources did not change
  between the scan and the final hash comparison.
- Enumerated client `.rs`, `.js`, `.html`, `.json` and `.svg` files: **370**
  main-derived and **420** native. Also inspected the core content/IDs and
  server profile/live/result producers listed in the manifest. No archive,
  build output, credentials, player database or production API was accessed.
- The manifest contains **437 unique paths**, with hashes per baseline and
  candidate literal line numbers. Empty hash means absent on that baseline.
  Candidate lines are a lexical search aid, **not confirmed defect counts**:
  they include comments missed by heuristics, tests, names, CSS, logs and
  dynamic key prefixes. No percentage of translation completeness is inferred.
- The tables below classify confirmed presentation gaps and supporting
  producers/guards. Paths are repository-relative; native-only paths may
  not exist in the planning branch. Line references belong to the stated
  audited baseline and must be refreshed on the implementation SHA.

## 2. Main finding: parity is not presentation coverage

Main's catalogs have **174 keys each**, native's **217 each**. Both have
zero English-only keys and zero Portuguese-only keys. `build.rs` and
`src/i18n/check.rs` already check key parity, placeholders, allowed glyphs
and contiguous indexed families. Those checks do not find a sentence
displayed directly from Rust, JS, a JSON content field or an API code.

`src/i18n/content.rs` maps Rejection only. Its comment explicitly defers
units/forms/Factions/Archetypes/Synergies/Augments to later P41 steps. Main's
catalog families are arenas/common/live/lobby/menu/meta/rejection/result/
spectate/toast. Native adds champion/native/shop, but not complete content
coverage. P41's roadmap/report already leaves later migration steps open.
P64 completes and updates that work; it does not establish a second i18n system.

The static literal-reference search found no confirmed missing complete
literal key. Two apparent misses were the dynamic `arenas.` prefix in
`assets/arenas/describe.js:23–24`; they are **false positives**, not missing
keys. Dynamic families still need content coverage checks.

There are 11 identical EN/PT values on main and 14 on native. Most are
legitimate brand/loan words: Tupi Battle Grounds, Lobby, Arenas, Ranking,
Casual, Arena and keyboard labels. Native also retains `native.rating =
Rating` and `shop.reroll = Reroll`; review terminology rather than blindly
failing all identical values. Equal translations are not intrinsically bugs.

## 3. Content sources and their presentation consumers

All paths below start at `crates/`. **Gap** means visible language bypass;
**support** means a stable source/guard to preserve, not text to translate in place.

### A. Champions, abilities, forms, status and battle information

| File | Evidence / correction responsibility |
|---|---|
| `tbg-core/src/content.rs` | **Support:** UnitDef/FormDef names, Stronghold names/branches and AugmentDef name/text; IDs, stats and mechanics remain canonical. Do not translate the simulation table in place. |
| `tbg-core/src/types.rs` | **Support:** UnitKind, Faction, Archetype, OnHit, statuses and active/staged pools; map IDs exhaustively, never display Debug output as translated text. |
| `tbg-client/src/i18n/content.rs` | **Gap:** only Rejection has an exhaustive mapper; missing presentation resolvers for content domains. |
| `tbg-client/src/ui/tips.rs:12` | **Gap:** English matchup table, `heals`, `dmg`, `Good vs`, `Weak vs`, role/Faction Debug names, tutorial HINTS and dynamic values. |
| `tbg-client/src/ui/inspect.rs:74` | **Gap:** `Heal`, `Damage`, `Base form`, tier, Slow/Root/Ready, role/Faction and decimals already formatted before the UI; localize report generation as well as headings. |
| `tbg-client/src/ui/evolve.rs:80` | **Gap:** champion/form names, `tier`, `Fully evolved`, attack-speed/damage/range/on-hit descriptions and costs. |
| `tbg-client/src/ui/hud_update.rs:84` | **Gap:** Shop names, Augment name/text, War Council wait/selection messages, owned Augment names and counters. |
| `tbg-client/src/ui/battle_hud.rs:184` | **Gap:** HTML projection carries raw Augment names/text and champion names. Fix the producer/consumer contract, not just the HTML heading. |
| `tbg-client/assets/battle/hud.js` | **Gap:** dynamic inspect, Shop, War Council and Fortification strings; preserve data attributes and kind/offer identity. |
| `tbg-client/src/ui/hud_top.rs:62` | **Gap:** hardcoded RANKED badge; player names are user data and must remain unchanged. |
| `tbg-client/src/ui/hud.rs` | **Gap:** native battle `Leave battle`; audit single-word buttons omitted by multiword literal heuristics. |
| `tbg-client/src/visuals/names.rs` | **Gap:** in-world unit labels/selected names use English display names; do not rename ECS identity or assets. |
| `tbg-client/assets/factions/factions.json` | **Support/gap:** English Faction names feed frame identity and display; add display mapping without breaking canonical name lookup. |
| `tbg-client/assets/factions/art.js:11` | **Support:** compares canonical names and frame metadata; preserve matching while translating tooltip/display labels. |

### B. Augments and native War Council

| File | Evidence / correction responsibility |
|---|---|
| `tbg-client/assets/augments/augments.json` | **Gap:** rarity names, Augment names and descriptions are English content outside the paired catalogs. |
| `tbg-client/assets/augments/art.js:44` | **Gap:** sets visible rarity from `def.rarity`; same name also performs asset lookup, so split display from identity. |
| `tbg-client/src/ui/native_battle/council/layout.rs:100` | **Gap, native:** Confirm choice, BATTLE PAUSED, Choose an Augment, rule explanation and actions. |
| `tbg-client/src/ui/native_battle/council.rs:263` | **Gap, native:** waiting/forgone/selected states; `Field::Name` and description read raw definitions. |
| `tbg-client/src/ui/native_battle/unit_panel.rs:148` | **Gap, native:** Damage/Per second/Attack every, report names, Evolution form text and ability descriptions. |
| `tbg-client/src/ui/native_battle/cards.rs:142` | **Gap, native:** Shop readout uses `def(...).name`; translated generic card labels do not translate the champion. |
| `tbg-client/src/ui/native_battle/score.rs:58` | **Gap, native:** GET READY; inspect repeated update path at line 239 too. |
| `tbg-client/src/ui/native_battle/fortify.rs:156` | **Gap, native:** Stronghold HP, raw kind/form names, Closed/chosen/max/paused/start states. |
| `tbg-client/src/ui/native_battle/chrome.rs`, `reserve.rs`, `arena.rs`, `resources.rs`, `mod.rs` | **Support/coverage:** central UI/update wiring and already-localized parts; verify refresh, accessibility and no hardcoded secondary branch. Not all literals here are defects. |

### C. Arcanas

| File | Evidence / correction responsibility |
|---|---|
| `tbg-client/assets/arcana/arcana.json` | **Gap:** family/node names/descriptions, eligibility, prerequisites, summon explanation and copy/status strings. Numeric values/preset IDs must stay unchanged. |
| `tbg-client/assets/arcana/editor.js:59` | **Gap:** max-rank/not-enough notices, rank actions and aria labels, Points left, storage/default/loading/error states and raw content strings. |
| `tbg-client/assets/arcana/preset.js:15` | **Gap:** prerequisite explanation joins raw node name with `rank` and `now`; validation rules and ranks are language-independent. |
| `tbg-client/src/ui/shell/arcanas.rs:100` | **Gap, native:** raw family/node descriptions and requirements; translating headings alone leaves the page mixed. Lines 166–167 build `Energy?` then replace it: replace this workaround with a full parameterized points message. |
| `tbg-core/src/arcana.rs` | **Support:** node IDs, costs, prerequisites and eligibility are mechanics; map them at the client edge. |

### D. Strongholds, Fortification and arenas

| File | Evidence / correction responsibility |
|---|---|
| `tbg-client/assets/stronghold/strongholds.json` | **Gap:** specialty, summary, passive, advantages/disadvantages, branch and tier names, instructions and copy templates. |
| `tbg-client/assets/stronghold/describe.js` | **Gap:** tiles, HP/damage/range/attack interval, Spoils, War Banner and Energy-effect sentence composition. |
| `tbg-client/assets/stronghold/picker.js:40` | **Gap:** storage/default/selection states, labels, aria/alt, passive/advantages/disadvantages/fortification, raw JSON display text. |
| `tbg-client/src/ui/fortify.rs:90` | **Gap:** raw kind/step/branch names, Max/Fortify/Foe and costs. `record()` carries a canonical kind: keep it stable. |
| `tbg-client/src/ui/shell/strongholds.rs:48` | **Gap, native:** raw name/specialty/summary/passive and bullet text; lines 147/181 onward add English units/delta labels. |
| `tbg-client/assets/arenas/describe.js` | **Partial/support:** already resolves names/effects by i18n keys, but `signed` and Energy conversion use raw number strings. Verify localized numeric formatting. |
| `tbg-client/assets/arenas/hub.js`, `banner.js`, `arenas.json` | **Coverage/support:** already-keyed arena page/banner and numeric source; check no raw fallback and switching while open. Do not count these whole files as untranslated. |
| `tbg-client/src/ui/shell/arenas.rs`, `src/ui/native_battle/arena.rs` | **Coverage/support:** keyed native names/effects; verify numeric/context parity and refresh. |

### E. Ranking, player profiles and API-produced values

| File | Evidence / correction responsibility |
|---|---|
| `tbg-client/assets/ranking/panel.html` | **Gap:** ranking/profile headings, columns, actions, placeholders and accessible labels. |
| `tbg-client/assets/ranking/controller.js:20` | **Gap:** Global Rankings, Try again/loading/errors; matches English error prefixes and messages rather than typed failure categories. |
| `tbg-client/assets/ranking/render.js:42` | **Gap:** explicit English date locale, ranking/page/profile/empty text, aria labels, Win/Loss/Draw and match plurals. |
| `tbg-client/assets/ranking/data.js` | **Gap/support:** errors are English messages surfaced by the controller; percent formatter forces `en`. Keep parsing/validation independent of language. |
| `tbg-client/ranking/index.html` | **Gap:** standalone page has `lang=en`, English title/metadata/nav and no common resolver bootstrap; preserve CSP. |
| `tbg-client/ranking/player.html` | **Gap:** standalone profile headings/table/loading/title/metadata; separate entry uses `ranking.js`, not the modern panel alone. |
| `tbg-client/ranking/ranking.js:7` | **Gap:** REASONS/RESULTS, raw Faction display at line 68, raw reason fallback at 94, errors and profile statuses. |
| `tbg-client/src/ui/ranking_ui.rs:97` | **Gap:** fallback ranking loading/empty/row/footer text. |
| `tbg-client/src/ui/ranking.rs` | **Support:** network/state producer; classify failure semantics and locale invalidation before changing its presentation consumers. |
| `tbg-client/src/ui/native_ranking/mod.rs:23`, `table.rs`, `podium.rs`, `profile.rs` | **Architectural gap, native:** `words(lang,en,pt)` already translates many labels, but bypasses catalog checks. Move these pairs into keys; do not report them all as English-only. Still raw W/L/D result codes (`profile.rs:177–184`), `Ranked 1v1` and fixed decimal percentages remain. |
| `tbg-server/src/api.rs:138` | **Support:** profile returns canonical Faction names/keys, favorite_faction, recent result/reason; live/online return status/mode slugs and player names. Map at the client; don't translate JSON field names or change API cache variants. |
| `tbg-server/src/store/ranked.rs:62`, `store/matches.rs` | **Support:** historical result/reason codes remain stable in storage/API; no migration to Portuguese values. |
| `tbg-server/src/live.rs`, `tbg-core/src/protocol.rs`, `account.rs` | **Support:** stable mode/status/error codes and guest/profile identity. Account names are not localization IDs. |
| `tbg-client/src/ui/shell/data.rs`, `assets/lobby/live-data.js`, `live.js` | **Coverage/support:** API parsing and mostly-keyed live view; verify status/mode/guest presentation, error states and language refresh without altering IDs or usernames. |

### F. Account, networking, common actions, queue and results

| File | Evidence / correction responsibility |
|---|---|
| `tbg-client/index.html:189` | **Gap:** battle/inspect/Shop/progress aria labels; account form title/help at 297; mixed dialogs outside the already-translated lobby. Keep language-tagged English fallback text where appropriate. |
| `tbg-client/assets/lobby/account.js:13` | **Gap:** login/register form headings, helper/submit/status/error text and account-key interactions. |
| `tbg-client/src/ui/account.rs:51` | **Gap:** AccountError presentation, login/logout/saved-key toasts. Keep keys/tokens and downloaded credentials unchanged. |
| `tbg-client/src/ui/account_ui.rs` | **Gap:** copy/paste/key notices, account fields/buttons/status for alternate UI. |
| `tbg-client/src/ui/textform.rs`, `textform_bevy.rs` | **Coverage/support:** form model and native rendering; ensure localized labels/errors update without dropping entered values/focus. |
| `tbg-client/src/ui/menu.rs`, `start.rs` | **Gap/coverage:** fallback menu and account/ranked prompts; initial Bevy message keys exist, inspect all branches rather than duplicating them. |
| `tbg-client/src/ui/queue.rs:49` | **Gap:** Connecting, Cancel, challenge created/joining, waiting/searching, Copy link, practice action and countdown composition. |
| `tbg-client/src/ui/surrender.rs:93` | **Gap:** confirmation/sent/loss warnings. Keep online surrender semantics untouched. |
| `tbg-client/src/ui/result.rs:39` | **Gap:** result reasons, duration/summary and rematch/retry/menu states; existing result keys do not localize their current hardcoded consumers. |
| `tbg-client/src/ui/actions.rs` | **Gap:** invalid challenge and share-invitation prose; intention/action identifiers stay stable. |
| `tbg-client/src/sync/messages.rs` | **Gap:** challenge unavailable and network-failure presentation; raw Failed strings can reach toasts. |
| `tbg-client/src/net/socket.rs:94` | **Gap:** connection-lost and version/update notices; resolve to client message IDs while preserving protocol/version logic. |
| `tbg-client/src/local.rs:65` | **Gap:** You/Training Bot are built-in identity labels, not ordinary player names; use explicit local identity semantics rather than translating any username equal to these words. |
| `tbg-client/src/ui/shell/mod.rs:134` | **Gap, native:** room-code validation error. |
| `tbg-client/src/ui/shell/settings.rs` | **Coverage/terminology:** keyed settings coexist with VSync backend value and a deliberate glyph/glossary specimen. Classify technical values/specimens rather than blindly translating Debug names. |
| `tbg-client/src/platform/http.rs` | **Conditional support, native:** URL/response errors need typed/safe UI mapping if surfaced; internal logs remain English. |
| `tbg-client/src/platform/window.rs:115`, `storage.rs`, `mod.rs` | **Conditional support, native:** distinguish player-visible crash/update/storage notices from diagnostics. Localize the user message if presented, retain technical log/path details as data. |

## 4. Cross-cutting files and omissions to prevent

- `assets/i18n/en.json`, `pt-BR.json`: expand content and UI families together;
  equality/parity alone does not prove coverage or translation quality.
- `src/i18n/mod.rs`: shared `Lang`, formatting/plural helpers and Localized
  updates. Parameterized content and dynamic reports also need invalidation.
- `assets/i18n/i18n.js`: DOM/attribute application and async switch propagation.
  `formats[tag]` caches NumberFormat by language only, although precision is
  an argument; cache by locale and precision/options. Guard out-of-order
  loads, and apply translations to dynamically inserted nodes.
- `build.rs`, `src/i18n/check.rs`, `tests.rs`: extend existing guards to
  referenced keys and content coverage; preserve glyph/placeholder validation.
- `ui/desktop.rs`, `desktop_actions.rs`, `battle_hud.rs`, `assets/lobby/lobby.js`:
  language bridge and cached projections must include language in their
  invalidation, even if no new view/API response arrives. No gameplay reset.
- Existing `ui/{augment,stronghold,arena}_data_tests.rs` and
  `tests/{arcana,art,factions,stronghold,arenas,ranking,account}.test.js`:
  preserve mechanics/identity checks while adding localized presenter tests.
  Locate exact current filenames from the manifest before running commands.
- `src/ui/glyphs.rs`, loaded fonts and `index.html` font loading: verify the
  actual supported Portuguese characters. English-only literal guards must
  allow translation catalogs without relaxing simulation correctness.
- `assets/lobby/presentation.json`, `assets/ranking/ranking.json`, art SVG
  metadata and developer-gallery strings are not automatically missing UI
  translations. The first two are predominantly asset manifests. Review
  any visible text embedded in images; do not translate file paths/IDs.

## 5. What must not be translated

Keep player names, account keys, URLs, API property names, wire enums,
serialized result/reason codes, storage keys, CSS classes, selectors,
asset filenames and canonical core terminology/identities stable. Translate
their display labels. Logs, assertions, test descriptions, build diagnostics,
ECS debug names and developer-only gallery/overlay text remain English unless
they are actually presented as normal gameplay UI. The source manifest keeps
these files visible so exclusions can be justified, not hidden.

No browser runtime, native GUI, API request, Cargo build or gameplay test was
run during this audit. The executor must confirm mounted routes and all
states on the integrated tree. Future P60/P61/P62/P63 UI is a coordination
requirement, not audited implemented code in these snapshots.

## 6. Planning validation report

Passed static documentation checks: nine local links/anchors, balanced
fenced blocks, shell-example syntax via `bash -n` without executing the
example, 437 unique manifest paths with valid hash fields and existing
baseline files, L01–L07 requirements and T01–T08 acceptance cases. Both
catalog pairs parsed successfully and have equal key sets as reported.
Audited client hashes were unchanged when rechecked after inspection.
`git diff --check` passed; source diff against origin/main is empty in this
planning branch. Final staged documentation is checked before publication.

Deferred: implementation, translation authoring/review, compiled guards,
runtime route/state checks, browser/native language switching, API fixtures,
font/readability evidence and PR of the future code correction. The owner
asked for investigation/planning and an execution prompt, not implementation
in this work front. No full-suite or release-readiness claim is made.
