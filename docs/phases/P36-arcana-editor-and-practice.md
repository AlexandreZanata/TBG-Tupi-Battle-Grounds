# P36 — Arcana editor, accurate previews and practice

**Depends on:** P35 reviewed, including browser protocol regression.
**Branch:** `phase/p36-arcana-editor`.
**Status:** planned; follows the completed P19–P31 visual/UI system.
**Difficulty:** medium.

## Goal and rules

Make six choices understandable on desktop and phones without delaying a
match or requiring setup. Use the approved medieval UI kit and the same
core calculations as the authoritative server (BR-AR01–BR-AR06/08/09).
The editor is pre-match; there is no new preparation round or live respec.

## Read first (only these)

- `AGENTS.md`; GAME-DESIGN §9 and P34's approved values; P35 report/types.
- ART-DIRECTION UI/readability/device budgets; P19/P20/P30 UI contracts.
- Current client menu, UI kit/cards/input, config/local storage, link/net,
  sync, local.rs, tips, Evolution panel, HUD and result metadata handling.
- Core Arcana validation/Stats functions and MatchRules; current gallery.
Use Bevy examples on disk for the actual pinned version, never old APIs.

## Affected files and exclusions

Add `ui/arcanas.rs` with small focused helpers; modify menu/actions/state,
account-aware selection resource, config/link/sync/local, details/previews
and existing UI tests where meaningful. Adapt to post-P31 module locations.
No new renderer, UI framework, asset pipeline, paid art, tree graph,
drag-only input, supremes, account unlocks or public deployment.
One selected preset only; free-text names/multiple preset slots are deferred.
P37 supplies cloud persistence. Do not pretend a local save is an account save.

## Steps (in order; check and commit each)

### 1. Selection lifecycle and local storage

Use a bounded resource containing draft ranks, last server-accepted preset,
dirty/pending state and last MatchRules. Initialize with the complete default
without a forced tutorial. Store a guest preset under `tbg_arcanas_v1` with
schema and ruleset versions; parse bounded data and validate every load.
On missing/corrupt/unknown data, keep the default, explain the reset, and
offer redistributing points; never reinterpret an old schema silently.
On account changes, scope local data correctly; P37 will replace account
selection from its authoritative stored response. Never overwrite the
currently running match resource from an asynchronous preference update.
**Check:** reload, malformed storage, version mismatch, logout/account switch
and auto-resume preserve the intended draft/accepted/match boundaries.

### 2. Compact editor using the existing kit

Add an Arcanas menu action. Show War, Precision and Vitality as three
sections with their two nodes, 0/1/2 rank, description, affected roster
portraits/names, bonus and remaining points. Use stable schema order.
Provide labeled +/- buttons, Apply and Back. Back with an unsaved draft
returns to the accepted preset without changing it. No free-text fields.
Disable + at rank/budget cap and - at zero. Tree totals are visible but
need no separate prerequisite UI. Apply requires exactly the full budget.
Use tap/click/keyboard; essential information never relies on hover or
long-press. Preserve P30 press feedback and cancel accidental scroll taps.
**Check:** all 141 complete presets are representable, overspending is
impossible in the editor, and server rejection can still be handled.

### 3. Explain the actual resulting attributes

For a selected roster unit show base → with draft and the affected stat;
use core effective_stats_with_arcanas, never duplicate percentage formulas
in UI code. Identify whether preview includes current Evolution/Synergy.
Explain base-Archetype eligibility and that unchanged attributes remain
unchanged. Show integer values so small rounded gains are not overstated.
In-match unit/Evolution details use the frozen MatchRules and public Synergy,
not a menu draft. Opponent details use the opponent's applied preset.
Expose both players' selections read-only from match details and result.
**Check:** representative HP/damage/heal previews equal server/core outputs,
including Marksman, Grove, Bramble, zero attributes and changed Synergy.

### 4. Queue/room handoff and feedback

When a draft is applied, send SetArcanas and wait for ArcanasSelected.
Only then send the selected queue/room request; new users automatically
confirm the default without an extra decision. Never queue a partial draft.
While queued/in a room/in a match, editor is read-only; Cancel unlocks it.
Rejected/rate-limited/disconnected requests clear pending state, retain the
last accepted preset and give an actionable message. Version mismatch
requires reload, not guessing compatible ranks. Play again preserves the
accepted selection and MatchMode; changes require returning to the menu.
**Check:** delayed/rejected acknowledgements and rapid Apply/Play taps do
not queue a stale preset or send duplicate matchmaking requests.

### 5. Practice parity

Construct LocalMatch with the player's complete selected preset and the
complete default for the bot. Populate MatchRules exactly like online.
Keep the neutral constructor only for explicit harness/tests, not default
practice. No stronger private bot stats. Practice and online rendering share
the same message/view path; online outcomes remain entirely server-owned.
**Check:** a complete browser practice match exercises buy, Evolution,
Synergy, healing/status where available and correct result metadata.

### 6. Responsive and accessibility review

Desktop shows the three compact groups; portrait stacks them without a
large dependency graph. Test at 360×800, 390×844 and 1366×768, plus the
project's 320 px minimum if required by the P19 kit. Keep effective touch
targets ≥44 px, readable labels, visible points and no horizontal overflow.
Use rank numbers/text in addition to color, keyboard focus and kit contrast.
Do not redesign the existing lobby, cards, Strongholds or characters.
**Check:** capture editor, partial/full preset, queue lock and in-match
details at desktop and phone sizes; owner approves look and interaction.

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown
```

Run `trunk build --release` in crates/tbg-client and inspect real browser
behavior, console and rendered values. Native compilation is insufficient.
Run two browser clients to completion, including reconnect; preserve the
published preset display throughout. Use gallery regression screenshots
at desktop and phone size to check existing UI/visuals still fit.
Do not claim real Android/iPhone approval from a resized desktop viewport.

## Acceptance and next-phase gate

- [ ] Every user can play immediately with all points through the default.
- [ ] Editor validation, integer previews and lock/ack flow are correct.
- [ ] Practice and online use the same approved rules and frozen metadata.
- [ ] Unknown storage/version data is recoverable without silent power changes.
- [ ] Desktop/phone screenshots, browser matches and checks are recorded.
- [ ] Owner approves the editor; reviewer permits P37, not public release.

## Risks and reviewer checklist

Check stale acknowledgements, menu/account changes during resume, draft
values leaking into live details, and Support forms accidentally shown as
damage-only. A locally validated selection never replaces server validation.
Do not add a tutorial wall to explain choices that need simpler design.

## Phase report

_Not executed. Fill in HEAD, checks and actual browser observations, screenshot
paths/viewports, preview comparisons, request failure cases, practice/online/
reconnect results, accessibility and owner look decision, deviations/blockers._
