# Patch <release label> — <date>

**Status:** Draft / Approved / Released (choose one).
**RulesetVersion:** <previous> -> <new>.
**Source hash / Git commit:** <hash> / <commit>.
**Collection window:** <UTC days and cutoff>.
**Evidence report:** <internal report location; publish only safe aggregates>.
**Owner approval / deployment record:** <decision and release reference>.

## Intent

<Describe the player experience to improve and the tradeoff being accepted.
Distinguish an investigation hypothesis from a confirmed change.>

## Balance changes

### <Unit / Evolution / Augment / Arcana / Stronghold / Arena>

- **Before -> after:** <exact attribute, units and values>.
- **Expected effect:** <what changes in a match and for whom>.
- **Online evidence:** <ruleset, cohort/definition, eligible matches/sides,
  concentration qualification, estimate and uncertainty; or insufficient data>.
- **Confounders:** <duration, skill, repeated opponents, simultaneous changes>.
- **D-33 confirmation:** <lab source hash, scenario, paired seeds/seat swap,
  held-out result, artifact and acceptance decision; or pending>.
- **Risk and follow-up:** <what would reject the hypothesis and when to review>.

Repeat for each coherent change family. Do not present an online correlation
as causal proof or publish a balance change while required evidence is pending.
Do not change unrelated number groups together merely to fill a patch.

## Interface and accessibility

<Player-visible desktop changes, supported languages and relevant verification.>

## Fixes

<Trigger, previous behavior and corrected behavior. A gameplay bug fix also
receives a new RulesetVersion, even inside the same marketing patch.>

## Competitive integrity and data notes

<Collection/eligibility changes, missing data, public suppression, rating
policy changes and whether the cohorts are comparable. No personal evidence,
account identifiers, keys, IPs or private adjudication details.>

## Release verification and rollback

- Actual focused checks and results: <commands/evidence>.
- Deferred checks and reasons: <do not mark deferred as passed>.
- Migration/backup compatibility: <schema and tested recovery path>.
- Feature flag rollback: <exact configuration/version; preserve newer results>.
- Separate deployment approval: <reference>.

## Next-revision comparison

<Compare the same eligible definitions after enough new-revision data exists.
Keep revisions separate; show changed population/concentration/coverage.
No fabricated comparison while the new revision has insufficient data.>
