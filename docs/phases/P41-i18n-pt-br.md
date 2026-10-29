# P41 — Full client i18n: English and Brazilian Portuguese

**Status:** planned (spec written 2026-10-29 against `origin/main`
`1bf8761`, P40 and P16-F Interest D-43 included). Writing this spec
changes no rule and enables nothing.
**Depends on:** nothing competitive. Coordinate with A-1 (see
*Coordination*).
**Branch:** `phase/p41-i18n`, created from `main` — or from
`docs/p41-i18n-spec` while this spec commit is not merged yet (step 0).
**Difficulty:** medium (wide but shallow: many strings, two text layers,
two fonts). One commit per step.
**Scope:** `tbg-client` only, desktop browsers only (AGENTS.md platform
focus). Zero lines change in `tbg-core`, `tbg-server`, `tbg-balance`.

## Goal

A player picks **English** or **Português (Brasil)** in the lobby and every
word the client shows — lobby, Arcana editor, battle HUD, War Council,
unit inspect, toasts, results, nameplates, account and ranking pages —
appears in that language, with correct accents, and the choice is
remembered. A first visit from a Portuguese browser opens in Portuguese.

**Performance is a hard requirement, not a nice-to-have:** the language
system adds no per-frame work, no per-frame allocation, no frame-rate
loss, and no visible delay to the first menu.

## Invariants (absolute, checked in step 12)

1. **Client only.** The server and `tbg-core` never see a language. The
   protocol is unchanged; `PlayerIntention`s carry no text. Both players in
   one match may use different languages.
2. **Canon stays English.** `tbg-core` content names (`UnitDef::name`,
   `AugmentDef::name/text`, …) stay the canonical English source and the
   last-resort fallback. Glossary terms keep their English spelling in
   code, docs, and commits (AGENTS.md non-negotiables 1 and 7). The only
   non-English files are the translation catalogs.
3. **Zero per-frame cost.** A static label is translated when it is
   spawned and again only when the language changes. A lookup is an array
   index (Rust) or a property read (JS) — never a hash, parse, or fetch on
   the hot path. Existing compare-before-write patterns stay.
4. **No layout flash.** The lobby stays hidden (`data-ready="false"`) until
   its text is in the chosen language.
5. **Every key exists in every language** — enforced by the build, not by
   review.

## Audit findings (2026-10-29, from the current tree)

| Finding | Where | Consequence |
|---|---|---|
| Two text layers | HTML/CSS/JS overlays (`index.html`, `assets/lobby/lobby.js`, `assets/battle/hud.js`, `assets/arcana/editor.js`, `ranking/*`) and Bevy UI (`src/ui/*.rs`, `src/visuals/names.rs`, `src/overlay.rs`) | One catalog must feed both |
| Rust composes text sent to the DOM | `ui/battle_hud.rs` (`data-offer-N`, `data-inspect`, `data-council-options`, `data-synergy`), `ui/desktop.rs` | Rust localizes what it composes; JS localizes what it composes |
| Bevy default font is ASCII-only | `bevy_text` `FiraMono-subset.ttf` (95 glyphs, max U+007E); guarded by `ui/glyphs.rs` | `ã ç é` render as boxes; needs a Latin-1 font |
| DOM fonts are ASCII-only too | `assets/fonts/Battleground*.woff2` (99 glyphs each, Noto subsets; P19 meant Latin-1) | Accents fall back to a system font mid-word; re-subset needed |
| Content text lives in `tbg-core` | `content.rs`: 11 `UnitKind`s with forms, 40 `AUGMENTS` with numeric `text`, factions, archetypes, synergies | Client-side keyed by stable id; English must stay equal to core |
| Arcana copy lives in data | `assets/arcana/arcana.json` (`name`, `description` per family/node) | Text moves to catalogs; JSON keeps ids and numbers |
| Protocol carries no prose | `tbg-core/src/protocol.rs` strings are player names and keys only; rejections are `Rejection` codes | Server needs no change (verify in step 1) |
| Portrait slug uses the kind id | `hud.js` `portrait(image, kind)` gets `{:?}` of `UnitKind` | Must stay the id when names are translated |
| Decimal formatting is hard-coded | `×1.00` Interest multiplier and its `title` tooltip, inspect values | pt-BR uses a comma: `×1,25` |

## Coordination (read before step 0)

A-1 edits `tbg-client` art and possibly HUD files. Never run both phases on
the same files at once (AGENTS.md). If A-1 is in progress, run steps 0–5
(new files, fonts, `index.html` head) and wait for A-1 to merge before
steps 6–10. Rebase decisions belong to the reviewer.

## Read first (only these)

- `AGENTS.md`, this file.
- `docs/ARCHITECTURE.md` §10 (budgets).
- `docs/GAME-DESIGN.md` §2 (glossary).
- `crates/tbg-client/index.html`, `src/ui/mod.rs`, `src/ui/desktop.rs`,
  `src/ui/desktop_actions.rs`, `src/ui/battle_hud.rs`, `src/ui/glyphs.rs`,
  `src/config.rs` (`url_param`, `storage`).
- `bevy_text-0.20.0/src/lib.rs` lines 145–152 (how the default font is
  inserted at `AssetId::default()`).

## Do NOT

- Add a runtime i18n framework (`fluent`, `rust-i18n`, `i18next`, …). The
  catalog is two flat JSON files; codegen is ~150 lines of `build.rs`.
- Send a language, locale, or translated string to the server.
- Translate inside a system that runs every frame without a change guard.
- Rename glossary terms, enum variants, `data-*` attribute names, or
  `UiAction`s.
- Touch phone layouts (`hud_top.rs` narrow branches etc.) beyond keeping
  them compiling; they get translated strings for free through the same
  calls, but are not verified.
- Machine-translate and ship without the owner's terminology decisions
  (O-P41-1, O-P41-2).

## Technical design (the contract for steps 2–10)

### D1. Catalogs (single source of truth)

```
crates/tbg-client/assets/i18n/en.json      reference, complete
crates/tbg-client/assets/i18n/pt-BR.json   same keys, same placeholders
```

- Flat object, keys `area.thing[.part]`, lowercase, dots and `_` only,
  sorted alphabetically (diff-friendly):
  `"lobby.mode.casual.title": "Casual"`.
- Placeholders are named: `"toast.need_energy": "Need {cost} Energy"`.
- Plurals: two keys `x.one` / `x.other`; the helper picks one. pt-BR rule:
  `n == 0 || n == 1` → `one` (CLDR `i = 0,1`); English: `n == 1` → `one`.
- Indexed families use the id in the key:
  `unit.archer.name`, `unit.archer.form.1`, `augment.12.name`,
  `augment.12.text`, `faction.humans`, `archetype.ranged`,
  `rejection.not_enough_energy`, `arcana.node.<id>.name`.
- Trunk already ships `assets/` (`copy-dir`), so the JSON is served as is.

### D2. Rust side: compiled tables (`build.rs` + `src/i18n/`)

`crates/tbg-client/build.rs` (build-dependency `serde_json`, already in
`Cargo.lock`; reason comment in `Cargo.toml`) reads both catalogs and
writes `$OUT_DIR/catalog.rs`:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum Msg { LobbyModeCasualTitle, /* … one per key, PascalCase */ }
pub(super) static EN: [&str; MSG_COUNT] = [ /* … */ ];
pub(super) static PT_BR: [&str; MSG_COUNT] = [ /* … */ ];
// Indexed families, e.g. "augment.{n}.name" → AUGMENT_NAME[n]
pub const AUGMENT_NAME: [Msg; 40] = [ /* … */ ];
```

The build **fails** (panics with the key name) when: a key is missing in
pt-BR or extra in it; placeholder sets differ for a key; a value contains
a character outside the font charset (D5); an indexed family has a gap.
`cargo:rerun-if-changed` on both files.

`src/i18n/mod.rs` (`//! Active display language and compiled string lookup.`):

```rust
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Lang { #[default] En, PtBr }

impl Lang {
    pub fn code(self) -> &'static str;               // "en" | "pt-BR"
    pub fn from_code(code: &str) -> Option<Self>;     // "pt*" → PtBr, "en*" → En
    pub fn t(self, msg: Msg) -> &'static str;         // table[msg as usize]
    pub fn tf(self, msg: Msg, args: &[(&str, &dyn std::fmt::Display)]) -> String;
    pub fn plural(self, one: Msg, other: Msg, n: i64) -> Msg;
    pub fn decimal(self, value: f64, places: usize) -> String; // "1.25" | "1,25"
}

/// Static Bevy label: re-texted only when `Lang` changes.
#[derive(Component, Clone, Copy)]
pub struct Localized(pub Msg);

fn retext(lang: Res<Lang>, mut labels: Query<(&Localized, &mut Text)>);
// registered with .run_if(resource_changed::<Lang>)
```

`src/i18n/content.rs` maps core ids to `Msg` with exhaustive `match`es
(`unit_name(UnitKind)`, `form_name(UnitKind, u8)`, `faction(Faction)`,
`archetype(Archetype)`, `rejection(Rejection)`, …) so a new enum variant
is a compile error, and `augment_text(lang, &AugmentDef)` fills the
placeholders from `AugmentEffect` numbers.

`ui::label`/`ui::button` gain a `Msg` variant (`label_t(lang, msg, size)`
adds `Localized(msg)`); dynamic text keeps building its `String` where it
already does, through `lang.tf(...)`, and keeps writing `Text` only when
the value changed.

Lookup cost: one bounds-checked index. Memory: ~2 × catalog size of
`&'static str` in `.rodata`. No startup parsing.

### D3. JS side: one tiny runtime (`assets/i18n/i18n.js`)

- Loaded with `<script data-trunk defer src="assets/i18n/i18n.js">`
  **before** `lobby.js`, `hud.js`, `editor.js` (Trunk hashes it).
- Exposes `window.TBG_I18N = { lang, t(key, args), plural(base, n),
  number(value, places), apply(root), ready }` where `ready` is a Promise.
- Loads `assets/i18n/<lang>.json` with `fetch(url, { cache: 'no-cache' })`
  (ETag revalidation; no stale catalog after a deploy). On failure it falls
  back to `en.json`, then to the key itself — never throws.
- `apply(root)` sets text for `[data-i18n]`, and attributes for
  `[data-i18n-aria-label]`, `[data-i18n-title]`, `[data-i18n-placeholder]`,
  `[data-i18n-alt]`. One pass over ~200 nodes; runs on load and on
  language change only.
- `number()` uses one cached `Intl.NumberFormat` per language.
- `lobby.js`, `hud.js`, `editor.js`, `ranking.js` replace literals with
  `TBG_I18N.t(...)` and gate their `data-ready="true"` on
  `TBG_I18N.ready` (invariant 4). A `tbg:lang` DOM event tells them to
  re-render.

### D4. Choosing and switching the language

- **One resolver**, an inline non-deferred `<script>` at the top of
  `<head>` in `index.html` and the ranking pages, so it runs before
  anything else: `?lang=` (tests) → `localStorage['tbg.lang']` →
  `navigator.languages` (first `pt*` → `pt-BR`, else first `en*` → `en`)
  → `en`. It writes `document.documentElement.lang` and starts the
  catalog `fetch` early (`<link rel="preload" as="fetch" crossorigin>`
  equivalent in JS). Both JS and Rust read `document.documentElement.lang`
  — never re-resolve.
- **Rust at startup:** `i18n::plugin` reads `<html lang>` (wasm32) via the
  `Document` feature already enabled; native and tests use `Lang::En` or
  `?lang` through `config::url_param`.
- **Selector:** a two-option control (`English` / `Português (Brasil)`,
  each written in its own language, never translated) in the lobby
  Settings dialog. Click → `i18n.js` saves `tbg.lang`, sets `<html lang>`,
  re-applies the DOM, fires `tbg:lang`, and sets
  `data-action="lang:pt-BR"` on `#tbg-lobby` — the existing action
  channel polled by `desktop_actions::poll`. New
  `UiAction::SetLang(Lang)` sets the `Lang` resource; `retext` and the
  bridge's compare-before-write `attr` calls do the rest. Switching is
  allowed in the menu only (the Settings dialog is there).
- `<title>` and `<html lang>` follow the language.

### D5. Fonts with Portuguese glyphs

Charset (one constant, shared by `build.rs` check and the subset command):
U+0020–007E, U+00A0–00FF, U+2013, U+2014, U+2018, U+2019, U+201C, U+201D,
U+2022, U+2026, U+2192, U+25C6.

- **DOM:** re-subset Battleground Display / Sans / Sans Bold from the
  same Noto sources (`OFL.txt`) into **two files each**: the existing
  ASCII file (unchanged bytes, so English first load is identical) plus a
  `*-ext.woff2` with the remaining charset, declared as a second
  `@font-face` with `unicode-range`. The browser downloads the extension
  only when an accented glyph is on screen.
- **Bevy:** drop the `default_font` feature; embed one Latin-1 TTF
  subset (`include_bytes!`) and insert it at `AssetId::default()` in
  `i18n::plugin`, exactly as `bevy_text` does. Every existing `TextFont`
  keeps working without a handle. Which face is owner decision O-P41-3.
- Command (record the exact one in the report):

```bash
pyftsubset NotoSans-Regular.ttf --unicodes="U+0020-007E,U+00A0-00FF,U+2013-2014,U+2018-2019,U+201C-201D,U+2022,U+2026,U+2192,U+25C6" --layout-features='kern,liga' --output-file=assets/fonts/BattlegroundSansUi.ttf
```

- `ui/glyphs.rs` changes from "ASCII only" to "every catalog value and
  every remaining client string literal is inside the charset".

### D6. Who translates what (the bridge rule)

**Whoever composes a string localizes it, from the same catalog.**

| Text | Composed by | Notes |
|---|---|---|
| Static HTML chrome | `index.html` + `i18n.js apply` | `data-i18n` keys |
| Lobby news, stats status, Arcana editor, HUD chrome, War Council chrome | JS | `TBG_I18N.t` |
| Offer cards, inspect record, council options, synergy line, augment chips | Rust (`battle_hud.rs`, `inspect.rs`) | localized before `attr`; field layout unchanged; ids (`UnitKind`, augment id) stay in their fields for portraits and actions |
| Bevy screens, toasts, rejections, nameplates, countdown | Rust | `Localized` or `lang.tf` |
| `#tbg-form` labels and errors | Rust (`textform.rs`) | already set from Rust |
| Diagnostics overlay (`overlay.rs`, `?measure`) | — | developer-only, stays English |
| Player names, room codes, keys | — | user data, never translated |

## Steps (one commit each: `feat(p41): <step title>`)

### 0. Preflight

Branch from `main`. Run the four AGENTS.md checks. Record baselines in the
report: brotli size of the `.wasm` and of first-load assets (`trunk build
--release`), time to menu, and FPS/frame-time p95 with `?stress=200` and in
a practice match at the 3-minute mark (`?measure`), desktop 1920×1080.

### 1. Audit (no code)

Inventory every user-visible string by layer and file (counts per file;
list composed strings with numbers). Confirm the server sends no prose.
List every fixed-width container that will hold translated text (Bevy
`button(.., width)`, CSS `white-space:nowrap`). Commit the inventory as
the report's appendix.

### 2. Catalog format and compiled lookup

`build.rs`, `src/i18n/{mod,content}.rs`, `Lang`, `Msg`, `Localized`,
`retext`, `tf`, `plural`, `decimal`. Start with ~10 keys in both files.
Tests (`src/i18n/tests.rs`): parity failure messages (feed `build.rs`'s
check function bad input), `tf` placeholder fill, pt-BR plural of 0/1/2,
`decimal` for both languages, `retext` runs only when `Lang` changes
(two `app.update()` calls, count changed `Text`s).

### 3. Fonts

D5 for DOM and Bevy. Gallery screenshot of a test string
`"Ação Évolução ÁÂÃÀÇÉÊÍÓÔÕÚÜ ãõç"` in Bevy and in the lobby. Record font
byte sizes. Update `glyphs.rs`.

### 4. Resolver, selector, and action

D4 end to end with the ~10 keys: inline resolver, `i18n.js`, Settings
control, `UiAction::SetLang`, `<html lang>`, `<title>`. Check: switch in the
lobby, reload, still pt-BR; `?lang=en` overrides; a `pt-PT` browser gets
pt-BR.

### 5. Migrate `index.html` and `lobby.js`

Every visible text node and `aria-label`/`title`/`alt`/`placeholder` gets a
key. Lobby readiness waits on `TBG_I18N.ready`.

### 6. Migrate battle HUD and War Council (`hud.js`, `battle_hud.rs`)

D6 rows; `Ranked`/`Casual`, `Yours`/`Enemy`, `tiles`, `tiles/s`, the
surrender states, Interest multiplier (`×1.00`) and its tooltip via `number()`/`t`.

### 7. Migrate Arcana editor

Move `name`/`description` from `arcana.json` to catalog keys
(`arcana.family.<id>.*`, `arcana.node.<id>.*`); `editor.js` reads them via
`t`. `arcana.json` keeps ids, numbers, art.

### 8. Migrate Bevy UI

`ui/mod.rs` (`rejection_text` → `i18n::content::rejection`), `result.rs`,
`account_ui.rs`, `account.rs`, `menu.rs`, `ranking_ui.rs`, `queue.rs`,
`tips.rs`, `surrender.rs`, `toast.rs`, `actions.rs`, `evolve.rs`,
`countdown.rs`, `hud*.rs`, `textform.rs`, `visuals/names.rs`.
Add a guard test: no `Text::new("`, `label("`, `button("` with a string
literal outside `src/i18n/` and `overlay.rs`.

### 9. Content names

Units, forms, factions, archetypes, synergies, Strongholds, augments
(templated text). **Contract test** `i18n_matches_core_canon`: for every
`UnitKind`, form, and all 40 `AUGMENTS`, the English catalog rendering
equals the `tbg-core` string. When balance changes an augment number, this
test fails and names the key to update — the catalog can never silently
drift from the rules.

### 10. Portuguese translation pass

Fill `pt-BR.json` using the owner's glossary (O-P41-1, O-P41-2). Then a
layout pass at 1920×1080 and 1366×768: Portuguese runs ~25% longer; fix
overflow with existing ellipsis/`min-width:0`/wrapping, never by
shrinking fonts below current sizes. Screenshot every screen in both
languages.

### 11. Ranking pages

`ranking/index.html`, `player.html`, `ranking.js` load the same
resolver and `i18n.js`.

### 12. Performance and regression

Repeat step 0's measurements in both languages and report the deltas.
Also: a language switch in the lobby measured with the Performance panel
(longest frame), and a 30-second Bevy trace in battle showing `retext`
never runs. Two clients finish an online match, one in `en` and one in
`pt-BR`, including a reconnect.

### 13. Documentation and report

ARCHITECTURE: new "Client text and languages" subsection (D1–D6 in five
lines) and the font/WASM numbers in §10. DECISIONS: **D-45 (Proposed)**
"Client-only i18n with compiled catalogs". GAME-DESIGN §2: a pt-BR display
column for glossary terms, marked **Proposed** until the owner signs off.
AGENTS.md non-negotiable 7: "…except `assets/i18n/*.json` translation
values". Fill the report.

## Acceptance checklist

- [ ] Four AGENTS.md checks green; `trunk build --release` green.
- [ ] `git diff main -- crates/tbg-core crates/tbg-server crates/tbg-balance` is empty.
- [ ] Build fails on a missing pt-BR key (demonstrated in the report with the exact error).
- [ ] Every screen listed in step 10 screenshotted in `en` and `pt-BR`, no boxes, no system-font fallback, no clipped text.
- [ ] First visit with `navigator.language = pt-BR` shows Portuguese with no English flash; choice persists across reloads.
- [ ] Frame-time p95 in `?stress=200` and in battle within ±2% of baseline in both languages (or within run-to-run noise, measured).
- [ ] Language switch: longest frame ≤ 16 ms; `retext` runs zero times in battle.
- [ ] `.wasm` brotli delta ≤ +60 KB; English first-load asset delta ≤ +15 KB (i18n.js + en.json); pt-BR adds the `*-ext.woff2` files and pt-BR.json only.
- [ ] Time to menu unchanged within noise.
- [ ] Mixed-language online match with reconnect completes.
- [ ] `i18n_matches_core_canon` passes.

## Owner decisions

| ID | Question | Proposed default |
|---|---|---|
| O-P41-1 | Portuguese display names for glossary terms | BattleLine → Linha de Batalha · Energy → Energia · Shop → Loja · Evolution → Evolução · Synergy → Sinergia · Stronghold → Fortaleza · Faction → Facção · Archetype → Arquétipo · Augment → Aprimoramento · War Council → Conselho de Guerra · Arcanas → Arcanos · Interest → Juros · Ranked → Ranqueada · Casual → Casual |
| O-P41-2 | Translate unit and form names? | Yes (Archer → Arqueiro, Shieldbearer → Escudeiro, …), owner reviews the list |
| O-P41-3 | Bevy UI face | Noto Sans (same family as the lobby's `Camp`) instead of today's Fira Mono — one look across both layers |
| O-P41-4 | Where the selector lives | Lobby Settings dialog only |
| O-P41-5 | More languages later (es?) | Out of scope; D1 makes a third language one JSON file plus one `Lang` variant |

## Risks and what to do

| Risk | Mitigation |
|---|---|
| Portuguese text overflows fixed widths | Step 1 lists them; step 10 fixes layout, not font size |
| Catalog drifts from balance numbers | Step 9 contract test |
| Stale catalog after deploy | `cache: 'no-cache'` revalidation; build-time parity makes WASM/JSON keys agree |
| Font license | Noto is OFL; keep `OFL.txt`, the renamed family names, and record the subset command |
| A system re-translates every frame | `Localized` + `run_if(resource_changed::<Lang>)`; step 12 trace |
| Executor machine-translates game terms inconsistently | Glossary decisions first; catalog reviewed by the owner |

## Reviewer checklist

- Re-run the acceptance list; open both languages in a browser yourself.
- Grep the diff for `format!` / `Text::new` inside per-frame systems
  without a change guard.
- Read `pt-BR.json` against O-P41-1 for term consistency.
- Confirm English screenshots match `main` pixel-for-pixel apart from the
  Bevy font change (O-P41-3).

## Phase report

> Fill in with real command output and measurements (AGENTS.md step 7).

- Baselines (step 0):
- String inventory (step 1):
- Font sizes (step 3):
- Measurements before/after (step 12):
- Deviations:
- Limitations:
