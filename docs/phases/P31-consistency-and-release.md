# P31 — Visual consistency review, playtest, and release

**Depends on:** P30 reviewed.
**Branch:** `phase/p31-release`.
**Difficulty:** medium. The reviewer leads; the owner signs off.

## Goal

Ship the medieval, competitive TBG: every character, Stronghold, card,
and screen looks like one family; online, ranked, and ranking still work
at the P14 standard; real players confirm it is fun and readable.

## Deliverable

- Final lineup and consistency fixes.
- Full regression of the competitive suite on the real server.
- Playtest round with fun metrics; balance untouched unless the owner
  approves a change (GAME-DESIGN process).
- Release tag `v0.2.0` and updated README screenshots.

## Read first (only these)

- `AGENTS.md`
- `docs/ART-DIRECTION.md` (all)
- `docs/GAME-DESIGN.md` §6 (fun metrics)
- Reports of P14 and P15–P30

## Do NOT

- Start new features; only fixes found by this review.
- Change balance values without the owner's decision recorded in
  GAME-DESIGN as **Proposed** with the playtest evidence.

## Steps

### 1. Consistency review
`?gallery=lineup&all`: all 8 units × 5 forms in one lineup, plus the three
Strongholds, at desktop and phone size. Check against ART-DIRECTION:
proportions, palette slots, team-color placement, detail density, motion
energy (contact sheets side by side), card portraits uniform. File every
outlier as a small fix commit (`fix(art): …`).

### 2. Readability test
Five people who did not build the game watch 30 s of a busy match on a
phone and answer: which side is yours, which unit is each, which units are
evolved, who is winning. Target ≥ 4 of 5 correct on each question.

### 3. Competitive regression on the real server
Deploy; repeat the P14 drop-and-resume matrix and a 50-match `--ranked`
bot run; check ranking pages and the in-game panel; confirm the Low tier
still meets P14's latency numbers (rendering must not delay input).

### 4. Playtest round
≥ 6 people, ≥ 20 online matches (mix of casual and ranked), on phones and
desktops. Collect GAME-DESIGN §6 metrics from match records and a short
survey (fun 1–5, readability 1–5, would play again). Record results.

### 5. Release
Update README (new screenshots, features, ranking link), THIRD_PARTY_NOTICES,
ROADMAP statuses; tag `v0.2.0` (`git tag -a v0.2.0 -m "Medieval competitive release"`);
the owner pushes the tag and announces.

## Acceptance checklist

- [ ] All verification commands from `AGENTS.md` pass; `trunk build --release` passes
- [ ] Lineup screenshots approved by the owner
- [ ] Readability test ≥ 4/5 on every question
- [ ] P14 matrix passes again on the release build
- [ ] Playtest metrics recorded; owner decision on balance recorded
- [ ] Tag created

## Reviewer checklist

- Every fix commit references the outlier it fixes.
- No host details committed.

## Phase report

_Reviewer fills this in: lineup notes, readability results, regression
results, playtest metrics, owner sign-off._
