# Stronghold power and current Blender previews — 2026-11-15

Status: work in progress; no balance acceptance or updated production release claimed.

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

Pending consolidation: deterministic update probe and append-only R17 trace
registration; Stronghold balance evidence and the applicable D-33 gate; focused
client sequence regressions; fresh identified native capture; all shared imagery
and generated website hashes; final CI and packaged/runtime checks. Windows and
desktop web visual observations remain pending and are not inferred from shared
source, compilation or Linux images.

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
