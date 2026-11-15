# Stronghold power and current Blender previews — 2026-11-15

Status: owner-authorized production preview (D-112); technical CI, artifact publication and attended rollout in progress. Competitive acceptance remains pending.

## Owner scope

The owner confirmed approximately 40% less Stronghold HP and 30% less damage,
with the relative kind identities preserved. Refresh every champion portrait,
all eight Arena previews, the Laboratory imagery and the public site's gameplay
captures using the current Blender models. Publish the resulting code and update
the live game. The previous native Ranked test was excluded by the owner's
browser-only choice; this new request explicitly includes native image captures.

## Proposed TW-02 values

Reference 6750/159 becomes 4050/111 HP/damage. Existing kind coefficients give
Classic 4050/111, Treasury 3766/101, Bastion 4657/98, Artillery 3442/122 and
Command 3888/101. Fortification branches, prices and passive effects stay as
specified in TW-01. The proposed identity is R17/3df52af730a4e914; all five natural-end traces are registered;
release acceptance remains pending D-33 evidence. Confirmation seeds must not be used for tuning.

## Browser findings on published R16

Two ordinary Ranked accounts were tested with a visible browser and the existing
`tbg-bots` opponent utility. One browser profile could not provide two independent
same-origin account sessions, so the second account used the normal authenticated
headless client. Linux native Ranked UI was not tested, as requested.

- Reloading resumed the same live match and its players. A later local opponent
  transport reset ended the first test by disconnect, not Stronghold destruction.
  The persisted result was unique: 1000→1020 and 1000→980, ruleset 16.
- The second opponent ran from the VPS through the public WSS endpoint. It stayed
  connected for its entire 900-second observation budget (RTT p50 7 ms, p95 9 ms)
  and the browser resumed after reload. Its timeout eventually caused a second
  disconnect result: 1020→1039 and 1000→981, persisted duration 1049 seconds.
- Neither result proves a complete match ending by Stronghold destruction.
- Manual purchases after autoplay/reload exposed separate, stale sequence counters.
  Commit 420c68e1 shares the counter and advances its floor from authoritative
  acknowledgements, retaining higher in-flight numbers. The corrected live
  purchase/reconnect scenario remains pending.

## Image findings and fixes

An official, provenance-verified .8 Linux capture build was identified as
b9041af8bcb982a94d72d8c1607ed25c1ef73e77, with `ui-acceptance`. Its current
Blender champion and Arena sources match this work's model sources. The first
Archer portrait was rejected: asynchronous scenery appeared behind the isolated
figure and lossless WebP was 115710 bytes, above the 40 KiB budget. No rejected
image was copied into the delivery tree.

Commit cdb1dd70 hides newly loaded Arena meshes before Gallery visibility
propagation. Commit 96f7dfb7 identifies capture versions/commits, retains ready
scene evidence, isolates the server endpoint and adds Arena preview captures.
All 88 portraits were captured from the verified PR native artifact at source
36b49b229b66e1e3afada11c5696655e2ef40564. The original PNG hashes,
binary identity and shipped WebP hashes are in
`docs/media/blender-refresh-20261115/portraits.json`. Lossless pixel comparison
and the 40 KiB budget passed for every image. A Linux contact sheet was inspected.
One Bonehorn startup timed out once; its isolated retry passed in 10.08 seconds.
The helper now terminates only its own renderer process group on timeout.
All eight initial Arena frames loaded successfully, but were rejected visually
for an unrelated recovery toolbar in the corner. The shared preview route now
suppresses that toolbar; replacement frames remain pending.

## Validation

The independent R16 release-recovery checkout completed the explicitly requested
full suite: formatting and Clippy passed, 1052 tests passed with five ignored,
WASM check passed and Trunk release passed. Its own unoptimized C8 run was
interrupted, then only unresolved tests resumed using the repository's optimized
CI test profile. Logs: `/tmp/tbg-live-validation-20261115/full-suite.log` and
`full-suite-resume.log`. These results belong to source 363e50db/.8, not R17.

For this front: package formatting, `git diff --check` and the desktop source
routing check passed. The new capture guard correctly rejected a mismatching .8
binary before rendering in the .10 source tree. The matching .8 portrait sample
exposed the Gallery bug described above. Source routing is not visual parity.

Pending release work: full competitive gate and 1x server/lab equivalence;
corrected live browser purchase/reconnect/natural Ranked completion; final
GitHub checks and publication approval. Windows and desktop web in-game
observations remain pending; shared source and Linux photographs do not prove
visual correspondence.

## R17 focused consolidation and discovery

The seven affected core test binaries passed: 31 tests, zero failures/ignored.
Command: `cargo --config .github/cargo-ci.toml test --locked --offline -p tbg-core
--features lab,training --test contract_constants --test contract_strongholds
--test contract_determinism --test contract_lab --test contract_ruleset_version
--test contract_release_identity --test champion_preview_legacy -- --test-threads=1`.
The shared nonblocking host lock and two-job limit were used. The first sandboxed
attempt could not write the existing /data cache; the authorized retry passed.

Five measured natural-end traces (seeds 1, 7, 42, 1000, 65535) update only the
new R17 arms; all historical baselines remain. Stronghold destruction occurred
at ticks 41676, 10203, 25488, 9671 and 18720 respectively.

The discovery comparison ran `tbg-balance strongholds --pairs 32 --threads 2
--seed-start 0` twice, first with `--apply stronghold_hp=6750,stronghold_damage=159`
and then with the proposed canon. Both complete 25-cell reports were written
(1600 games each); wall times were 210.79 and 98.34 seconds. The orchestration
session subsequently returned 143, so the completed manifest/report files are
the evidence, not a successful wrapper exit. Discovery seeds are not confirmation.
The binary was compiled against the recorded uncommitted numeric candidate;
the reports' git_head 5e3f6a7c alone does not identify that working-tree diff.
The raw reports and exact candidate patch remain under
`/tmp/tbg-blender-refresh-20261115/`. No passing release gate is claimed.

## Held-out confirmation (frozen candidate, no tuning)

The 256-pair matrix used seeds 1,000,000..1,000,256, both seat orders for
every kind pairing: 12,800 games in 756.48 seconds on two workers. Among the
four selectable kinds, observed matchup scores ranged from 38.7% to 59.4%;
Classic remains the laboratory reference. The matrix is not a full metagame gate.

The 1000-pair held-out policy probes used seeds 1,000,000..1,001,000:
Bot vs Bot seat A scored 48.9% (required band 48.5–51.5), with 100% natural
Stronghold destruction. Median duration was 829 combat seconds (outside the
non-release fun target 150–240 seconds). Heavy vs Bot scored 86.0% and Cheap
vs Bot 84.6%, both above the required <=55% threshold. With the old frozen
6750/159 values on the same seed block, the corresponding scores were 85.6%
and 85.0%. These required failures predate the change. This comparison does not
prove statistical equivalence and the confirmation block was not used to tune.

The planned confirmation reports all completed; the first silent wrapper
returned 143 after its final report, so wrapper success is not claimed. The
old-value diagnostic batch, streamed through tee, completed with exit 0.
Raw manifests and reports are retained in `docs/media/blender-refresh-20261115/balance/`.
No thresholds or test assertions were weakened. Evidence is not accepted for
a competitive release: the required spam rows fail; the full Nash/duel gate and
1x server/lab equivalence are pending. Publishing a preview with these known
limits requires an explicit owner decision.

## Current interface and capture corrections

The Stronghold picker and Fortification description JSON still contained old
HP/damage values even though the authoritative HUD was already reduced. All
four base and sixteen Fortification blocks now match R17. The existing two
core-data contract tests caught an unsupported Unicode escape introduced by
the first JSON serialization. UTF-8 text was restored; both tests passed.
The two shared intention-sequence regression tests also passed. Total focused
consolidation: 31 core tests plus four client tests, zero failures on the fixed
tree. The identified native .10/R17 build at 28f94749 compiled successfully.

An initial site-capture script incorrectly supplied the Arena-export flag to
ordinary menu captures, hiding their interface. Those frames were rejected.
The corrected profiles select the Arena through normal preferences, use a
private D-Bus with an empty disposable vault and never access the desktop vault.
A one-shot champion selection could die before the requested frame; the
capture helper now selects a current unit at capture time. Troops exports also
honor the requested delay. All nine corrected frames are consolidated. The final source
28f94749 passed both Stronghold catalog tests, both sequence tests and the
native build. The earlier failed test wrapper returned 143 after recording its
parser failure; the fixed targeted batch completed with exit 0. Native capture
2250 completed with exit 0. The preceding eight-frame capture wrote all its
records but its wrapper returned 143; only the hash-verified frames are reused.
See `docs/evidence/blender-captures-20261115.md` and the three per-image manifests.

## Static consolidation and production boundary

`web/scripts/check.py` passed: landing 208 files/7146 KiB, all admin variants
checked, generated champion/Arena content current. `check-native-inputs.py`,
package formatting and `git diff --check` passed. The mandatory patch-note
guard rejected the unaccepted numeric candidate with `Numeric balance notes
require accepted held-out evidence`. This is an intentional release block,
not a passing CI result; the balance note stays `kind=balance`, accepted=false.

The production host remains on the latest published .8 / R16 generation.
The browser test account returned from its old Result screen to Menu through
the ordinary control; the slot now reports zero running matches and zero
pending results. No player data was deleted. An attempted PR #83 merge was
rejected by automatic approval review for missing explicit merge authorization;
the owner question is pending. No alternate merge or deployment bypass was used.

The local desktop browser loaded all nine updated gallery images and displayed
the Portuguese Laboratory lightbox. Screenshot:
`/tmp/tbg-blender-refresh-20261115/site-preview-browser.jpg`. The loopback
preview remains available at port 8601 for owner review; it changes no live site.

## Owner authorization (D-112)

The owner answered both pending approval questions: authorize merge #83 and
authorize merges #83/#84 plus the .10 preview with the disclosed competitive
and visual limits. The owner then explicitly requested publication of all
changes in production. PR #83 merged as c527c0f5. The balance note remains
`kind=balance`; `accepted=true` now records owner acceptance for the preview,
with explicit `competitive_accepted=false`. No failing row is reported as passed.
The previous quality rejection remains historical evidence of the enforced gate.

## Final CI fixture consolidation

CI on 9792a4e3 exposed two TW-01 assumptions outside the initial focused set.
The Artillery target/Fury fixture used 50 HP, above R17 collateral damage;
its enemies now have one HP to isolate the unchanged three-extra-target limit
and per-kill attribution. All four-kill, ally-safety and Fury assertions remain.
The historical TW-01 boost test retains its exact assertions against a frozen
6750/159 reference; current R17 bases remain pinned by contract_strongholds.
No production mechanics or source fingerprint changed. Both focused module
tests and both fortification_power tests passed (four tests, exit 0) under
the shared lock/two-job limit. Total focused Rust checks: 39 passing tests.
After the explicit D-112 decision, all five patch-note guard regression tests
and the actual current-note guard passed, with no guard/test modifications.
