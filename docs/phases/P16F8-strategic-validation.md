# P16-F8 — Strategic validation: lab, load, long matches, playtest, release

> **Owner revision 2026-10-29 (D-41).** Not executed. Every gate here is
> still open for the active economy and Augments; Siege/Collapse rows no
> longer apply (D-36 superseded).

**Depends on:** P16-F7 reviewed, canon switched (every P16-F mechanic on).
**Branch:** `phase/p16f8-strategic-validation`.
**Difficulty:** medium in code (mostly running and reading), high in
judgment. Only numbers change in this phase, never mechanics.
**Owner decisions:** the playtest (step 6) and the release decision
(step 8).

## Goal

Decide, with evidence, whether the expanded game is ready for the public
server: no dominant strategy the lab can find, matches that end on time,
server and client costs within budget, and humans who understand and enjoy
it. Bot results are necessary, never sufficient.

## Starting state (verify first)

Record HEAD, `PROTOCOL_VERSION`, migration level, `ruleset_fp`, the last
gate of every P16-F phase, and the owner decisions O-1–O-12 with their
answers. Any mechanic still disabled in canon is out of scope here.

## Rules

GAME-DESIGN §6 (fun targets), §6 "Balance lab", §10.9 (expansion targets),
§10 rules as switched on; D-33 (lab method and immutability of contract
tests), D-34–D-38.

## Technical decisions

- Reuse `tbg-balance` (no new crate, no dependency). Add only what a gate
  row needs: a `full` pool definition (strategic pool × 4 kinds × Augment
  policies × reaction 1/5/10), a "long-match" report, and a combined
  `search` over kind/Fortification/Augment knobs (numbers only, ≤ 8
  changes per proposal).
- Statistics: paired seeds, both seat orders, cluster bootstrap, held-out
  confirmation seeds never used for tuning (P16-C).
- Load: the existing `tbg-bots` and `scripts/capacity.sh`; bot clients
  play with kinds, Fortifications and Augments enabled.
- Release criteria are a checklist in this file, decided by the owner.

## Read first (only these)

`AGENTS.md`; GAME-DESIGN §6, §10; DECISIONS D-33–D-38; ARCHITECTURE §9,
§10, §13; reports of P16-C, P16-D and P16-F1–F7; lab `gate.rs`,
`meta.rs`, `search.rs`, `policy.rs`; server `bin/tbg-bots.rs`,
`botclient.rs`, `scripts/capacity.sh`.

## Files

Lab `policy.rs`, `gate.rs`, `report.rs`, `commands.rs` (only what the
gate needs); `content.rs` and GAME-DESIGN §5/§10 for accepted numbers;
`botclient.rs` (choose a kind, Fortify, Augment greedily); evidence
`docs/media/p16f8/`.

## Do NOT

- Add or change a mechanic (that needs a DECISIONS entry and its own phase).
- Tune on confirmation seeds or report a number without its data.
- Lower a gate threshold to pass (a threshold change is a product decision).
- Deploy to the public server before step 8's explicit owner yes.

## Steps (in order; check and commit each)

### 1. Full gate on canon

`gate` with the full pool on 20 000 confirmation pairs; every §6 and
§10.9 row with intervals; Nash support and largest weight; payoff matrix.
Save `gate-canon.txt`, `payoff.csv`.
**Done when:** committed (`docs(p16f8): full canon gate`).

### 2. Strategy families

Win rates (with intervals) of aggressive (`rush`, `heavy`), economic
(`eco`, `banker-n`), defensive (`turtle`, Walls), Evolution (`evolver`,
`legacy`/`veterans` forced), Augment-exploiting (adversarial triples) and
hybrid policies; per kind; per Augment and per Augment pair (only pairs
seen ≥ 500 times); Interest use (share of income, % of payments at max
band); Forgo rate per round; comebacks; reversals (lead changes in HP
fraction after 2:00). Save `families.txt`.
**Done when:** committed (`docs(p16f8): strategy families`).

### 3. Long matches and termination

Distribution of running time (p50/p90/p99/max), Siege share, Collapse
share, longest no-damage Siege stretch, matches within 10 s of the bound;
an adversarial search that **maximizes** match length (two defensive
policies, Redoubt + Walls, healing Augments, Last Bastion, Stone Mend,
War Leech) must stay under the BR-M10 bound; `ServerLimit` count must be 0.
Save `long-matches.txt`.
**Done when:** committed (`test(p16f8): termination under adversarial play`).

### 4. Adversarial search and tuning

`search` against the full pool for the strongest (kind, Fortification
branch, policy, Augment preference) combination; if any combination
exceeds 60 % against the pool or the best Augment triple exceeds 65 %,
propose ≤ 8 number changes, `confirm` on fresh seeds, apply only on the
owner's yes (in step 7's commit). Save `search.md`, `proposal.md`,
`confirm.txt`.
**Done when:** committed (`docs(p16f8): adversarial search and proposal`).

### 5. Server and client costs

- `scripts/capacity.sh` at 250/500/1 000 concurrent matches with bot
  clients using every P16-F feature: CPU, RSS, bytes/s per player, late
  ticks; compare with the P10/P16-B tables (ARCHITECTURE §10).
- `examples/army_load.rs` at 10/40/100 units a side with three Augments,
  Fortification tier 2 and Siege active: step mean/p99, view bytes.
- C4b (`lab_equivalence --ignored`, real clock) with the full rules.
- Browser: desktop FPS in a busy match with the War Council open; phone
  viewport FPS in the browser pane; WASM brotli size; first-load bytes.
  Real phones are the owner's (step 6).
Save `costs.txt`. Budgets: ARCHITECTURE §10 rows unchanged; bandwidth per
player ≤ 6 KB/s at typical army sizes (if exceeded, report and recommend
delta views — P30-B — rather than cutting features).
**Done when:** committed (`docs(p16f8): server and client costs`).

### 6. Human playtest (owner)

At least 8 participants and 24 completed online matches (P38's bar), on
the test VPS, with a short form per match: which kind, Fortification and
Augments each chose and why; why they won or lost; was the War Council
readable while fighting; did any match feel stuck; would they play again.
Collect `ruleset_fp`-tagged records. Report: §6 targets from records
(duration, Stronghold kills vs Collapse, comebacks, replay rate,
readability ≥ 4 of 5), plus kind and Augment pick and win shares (small
samples: report counts, not percentages, below 30).
**Done when:** the owner's notes and the record summary are committed
(`docs(p16f8): playtest results`).

### 7. Owner balance decision

Present steps 1–6 together. The owner accepts, rejects or narrows the
step-4 proposal and any change suggested by the playtest (numbers only).
Apply accepted numbers to `content.rs`, GAME-DESIGN §5/§10 (Proposed,
"P16-F8 evidence, <date>"), contract tests in the same commit; re-run the
full gate on confirmation seeds.
**Done when:** committed (`balance(p16f8): <summary>`).

### 8. Release decision and report

Fill in the release checklist below and the report. The owner decides:
release to the public server, release with listed mechanics disabled in
canon (each disabled mechanic stays in code only if a later phase re-enables
it; otherwise remove it — no orphaned code), or hold. Record the answer in
ROADMAP and, if any mechanic is dropped, a DECISIONS entry.
**Done when:** committed (`docs(p16f8): release decision`).

## Release checklist

- [ ] Every §10.9 row marked required in its phase passes on confirmation
      seeds; no unexplained FAIL in §6 "Balance lab" required rows
- [ ] No combination beats the full pool by more than 60 %; Nash support ≥ 3
- [ ] `ServerLimit` never reached; p99 running time ≤ 390 s
- [ ] Capacity and bandwidth within ARCHITECTURE §10 (or the owner accepts
      the measured cost)
- [ ] C4b passes; verification block passes
- [ ] Playtest: ≥ 8 people, ≥ 24 matches, ≥ 4 of 5 can explain a defeat,
      no "stuck" match reported more than once
- [ ] Owner release decision recorded

## Acceptance tests

Steps 1–5 are measurement steps; the tests are the gate rows and:
`termination_under_adversarial_defense` (step 3, release build, `#[ignore]`
with reason, run by reviewers), C1–C12 (P16-C), and every P16-F contract
test unchanged except in commits that name their canon change.

## Verification

```bash
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown
cargo test -p tbg-server --test lab_equivalence -- --ignored
cargo run --release -p tbg-balance -- gate --pairs 20000 --threads 16
bash scripts/capacity.sh
cd crates/tbg-client && trunk build --release
```

## Performance

Step 5 is the performance step; every number is measured in this phase
and compared with its earlier measurement (never with a hypothesis).

## Acceptance checklist

- [ ] Steps 1–8 committed with evidence files
- [ ] Release checklist filled in; owner decision recorded
- [ ] GAME-DESIGN §5/§6/§10, ARCHITECTURE §10, ROADMAP updated

## Risks and what to do

| Problem | Action |
|---------|--------|
| Lab passes, humans find one strategy | Playtest notes beat bot rows; owner may disable a mechanic in canon |
| Small playtest samples | Report counts, not percentages; widen intervals; plan a second round |
| Costs exceed budget with big armies | Delta views (P30-B) before any gameplay limit (BR-U10) |
| Too many failing rows | Release with fewer mechanics on; never lower gates |

## Reviewer checklist

- Re-run step 1 and C4b; spot-check three rows of `families.txt`.
- Confirmation seeds absent from every search manifest.
- Records sampled from the playtest match the reported summaries.

## Owner checkpoint

Steps 6, 7 and 8.

## Phase report

Owner revision 2026-10-29: see the [combined report](P16F1-economy-and-siege.md#phase-report). The original checklist below is kept for the gated work still open.

_Not executed._ Fill in: starting state and decisions O-1–O-12; gate;
families; long matches; search and proposal; costs; playtest; accepted
changes; release checklist; deviations; blocked items; limitations.
