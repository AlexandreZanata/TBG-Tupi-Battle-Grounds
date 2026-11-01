# P70 visual execution review

**Status: 46 individual source candidates implemented; phase incomplete.**
All champion kits have model/color and authored Idle, Walk, primary action
or support/production performance, and Victory changes. Candidate compilation,
after images, temporal/live acceptance and the real Trident range increase
remain pending. This is a local work-front handoff, not approved final art.

Integrated source revision: `45ddd2438901b86a68f358ce461fa3fa63343708` (before this report/index commit).
Execution: `phase/p70-champion-visual-polish`, isolated in
`.local/p70-champion-visual-polish`. The owner's original worktree and active
builds were preserved. No push, merge, deployment or packaging occurred.

## Scope and implementation index

The required base is `55510c5363e3f1e555fa5133208a5faf20c5825b`; planning
parent is `60416ae`. Commit `3dc532b` preserves the newer owner validation
policy on this branch. The fixed cohort remains **46 champions / 230 forms**.
No auxiliary entity is counted as a champion.

The [manifest](P70-VISUAL-MANIFEST.json), [dossiers](P70-DOSSIERS.json) and
[evidence index](P70-VISUAL-EVIDENCE.json) link each champion to its local
art commit, changed kit files, implementation report and existing before
originals. Every after list and owner approval remains pending. Original
identity briefs remain Proposed, not new lore or accepted designs.

- [Trident](P70-TRIDENT.md), [Pyre Warden](P70-PYRE-WARDEN.md) and
  [Old Finn](P70-OLD-FINN.md): the three mandatory pilot candidates.
- [Humans and Undead](P70-NEXT-HUMANS-UNDEAD.md): the other 14 kits.
- [Machines and Plants](P70-NEXT-MACHINES-PLANTS.md): the other 17 kits.
- [Monsters, Celestials and Insects](P70-NEXT-MONSTERS-CELESTIALS-INSECTS.md):
  the other 12 kits.

These are per-kit changes to the existing procedural figures and pose seams:
shield salutes, revised gun/shaft grip targets, pressure mechanisms, leaf/staff
performances, multiple-support animal motion and distinct creature displays.
New semantic palette roles are centralized and documented as Proposed in
ART-DIRECTION. Existing A/B equipment progression, shared IK, meshes, event
timings and P63 ownership remain. No replacement animation framework,
dependency, per-frame mesh creation or platform-specific generic character
was added. Core/server/protocol/balance sources and contract tests are unchanged.

## Authentic evidence: before only

Content is frozen at the required base; tooling is
`27f98acd98b51fa69eb09f4703bd2ca59a33ed41`. Its private executable receipt
records SHA-256
`1883b601100297d8cf890d9a1a90658fee55713d485e64ef7678aff07675fc52`.
The unmodified art remains reconstructable in `.local/p70-frozen-baseline`.

The index links four hashed capture manifests and **630 immutable renderer
originals**, each fully decoded and hashed at capture time:

- 460 champion sheets: all 46, five forms, all ten clip IDs sampled.
- 120 enemy-team diagnostic sheets: Archer, Bone Brute and Trident,
  three-quarter/front/profile/back views.
- 50 auxiliary sheets: Beetling, Crate, Microdrill, Duodrill and Megadrill.
  Non-Base preview columns do not imply playable auxiliary Evolution.

The champion matrix has 2,300 clip/form slots per revision: 1,835 applicable
and 465 explicitly inapplicable. All applicable before pose slots are recorded;
**zero after slots and zero temporal recordings are recorded**. Specials
are independent event channels and cannot be proven by a generic Cast pose.

Capture conditions: Linux, Vulkan, NVIDIA GeForce RTX 4060 Laptop GPU,
NVIDIA 595.104.02; 1920 × 2160, standard quality, fixed camera, isolated
preferences. Static attachments are labeled rest-equilibrium samples.
These images do not establish live effects, loop continuity, match-distance
readability, browser/Windows compatibility or improved final appearance.

[Compact review sheets](p70-review/README.md) are lossless WebP derivatives
of genuine before crops, with original/crop hashes and resizing/caption
provenance. They are explicitly labeled BEFORE; no painted or generated
image substitutes for evidence. Bulk originals/binaries remain ignored.
No shipping Shop portraits were overwritten.

The gallery now selects the correct A/B branch and actual kit/P63 duration,
including strike/release and final poses. The new live recorder uses ordinary
training actions, captures original video and authoritative event/HP journals,
and distinguishes the subject's events from other units. It covers the fixed
roster without changing combat speed. Its candidate runtime is unverified;
a generic engagement does not prove conditional abilities or every clip.

## Blue artifact and mandatory pilots

The [artifact investigation](P70-BLUE-ARTIFACT.md) reproduced one cause:
an opaque team tabard attached at the hips intersected independently moving
chest geometry. Changing teams changed blue to red while the blue background
remained unchanged. Closed geometry/opaque materials did not support a
universal hole or culling explanation. Four callers now have fitted chest
heraldry and a short split hem. Team identification remains; distinct rags,
ribbons and skeletal surfaces are separate cases. Final regression is pending.

Trident has larger legs/torso, a shark jaw/gills/dorsal fin, slate/ivory blocks,
a longer three-tine weapon and a dedicated two-hand frontal load/thrust/withdraw
at the unchanged 0.30 s Strike. The weapon has one chest-relative frame and
wrists solved to its shaft. Nominal construction length is 1.52 versus 1.21
figure units (about +25.6%); this is source arithmetic, not executed mesh or
standing-height measurement. Written diagnostics cover the five forms,
contact direction, grip, sampled shaft/tine clearance and life-clip blends.

**Trident's authoritative range is still 1100.** No proposed integer has
been selected, no active value/version/contract was changed, and this requested
competitive change is not implemented. A longer-looking weapon does not
satisfy it. Real opponent spacing, cosmetic lunge and posed geometry must
calibrate the value first; GAME-DESIGN Proposed entry, range-only content
change, ruleset/contracts and both-seat/form boundary tests then belong in
one separate logical change. D-33 evidence and owner balance acceptance stay
pending; damage, cadence and Bloody Rend radius/bleed remain unchanged.

Pyre Warden has an open visible nozzle and a bound green/white flame jet with
readable essential geometry in Low quality. Existing ProfaneFlame events
remain the only trigger. The jet follows the actual displayed form/socket
after transform propagation, before visibility checks; its endpoint and exact
ground danger segment remain authoritative. No particle decides damage.

Old Finn has a readable oilskin/fishing silhouette and a fish with distinct
back/belly, forked tail, gills, eyes and fins. Fish scales are 1.0/1.35/1.70
across Base/tier1/tier2 on both branches. Existing release 0.22 s, flight
0.08 s, hit 0.30 s, clearance trajectory and Echo path are retained. Written
fish and flame endpoint regressions are queued, not passed.

## Source review findings and remaining known defects

Independent source audits corrected newly introduced Longaxe Victory grip,
Drumguard loop seam, Smuggler barrel yaw at release, Bloomkeeper sash
occlusion and Shatterback node matching. They also reviewed
socket/event contracts and identified short-arm reach cases in Oracle,
Solar Sentinel, Hornet, Farwatch, Bombard and several held props. Source
reach bounds are not rendered clearance or continuous collision proofs.

Two inherited defects remain explicit acceptance blockers in the
[Humans/Undead report](P70-NEXT-HUMANS-UNDEAD.md): Longaxe's ready/primary
support target exceeds its arm span, and Bone Brute B2's scythe support target
does too. Their new Victory poses avoid those helper calls; their existing
primary/ready weapon paths were preserved pending a calibrated local repair.
No claim of complete grip safety is made.

All champions still require all-form geometry/transition review, nameless
lineup at normal match distance, real primary/special cues, P62 Troops and
training, P63 Victory and owner look/fun approval. Parent-linked auxiliary
review must include Nester/Beetling, Smuggler/Crate, Cargo Crab passengers and
Drillnest production/fusion. Static sheets cannot replace these observations.

## Validation actually completed

Level: **work front**, including the explicitly requested baseline campaign.
The only successful compile was the S0 capture tooling against unchanged art.
The first attempt could not find `alsa.pc`; reusing the already installed
local pkg-config directory resolved it, with no package installation:

```sh
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" \
  env CARGO_BUILD_JOBS=2 \
  CARGO_TARGET_DIR=/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-Tupi-Battle-Grounds/target \
  PKG_CONFIG_PATH=/home/iiii/PESSOAL-PROJETOS-ALEXANDRE/TBG-Tupi-Battle-Grounds/.local/native-deps/root/usr/lib/x86_64-linux-gnu/pkgconfig \
  cargo build --offline --locked -p tbg-client --bin tbg-client
```

Result at S0: exit 0, `Finished dev profile [unoptimized + debuginfo] ...
in 56.63s`. Each baseline capture campaign subsequently completed with exit 0.
The initial restricted Xvfb launch could not open its display; the same isolated
display workflow ran successfully with the required local display permission.

Final source-only checks, separately from compilation:

- Per-front `rustfmt --edition 2021` on owned files: exit 0.
- `cargo fmt -p tbg-client -- --check`: exit 0, no output.
- `git diff --check`: exit 0, no output.
- Python AST parsing and both capture scripts' `--help`: exit 0.
- P70 Markdown local links: present.
- `python3 scripts/p70/validate_evidence.py --metadata-only --json`:
  bookkeeping valid; phase incomplete; 46/230 cohort; 1,835 before / 0 after
  applicable recorded slots; 630 locally available originals; 2,380 pending
  acceptance items. Original decode/hash checks were not repeated; capture-time
  checks were reused. This does not claim candidate visual validation.
- Source coverage: all 46 kit directories change model and pose sources;
  no core/server/balance/dependency/contract diff against the required base.
  A source-audit assertion initially included GAME-DESIGN against the pre-plan
  base and flagged the existing Proposed P70 exception. Comparing that document
  with planning parent `60416ae` confirms execution did not change it.

## Blocked compilation and exact continuation

> blocked: The scoped pilot development build and later combined pilot
> consolidation both returned exit 75 (shared validation slot busy), before
> Cargo started. The latter selected client filters `p70_`, `fish`, `pyre_jet`
> followed by the capture executable. No candidate test/build/after capture
> ran. The suspected cause is another validation owning the host slot.
> Compilation attempts stopped after the repeated result. No lock polling,
> bypass, cache deletion, fresh target, raised parallelism or terminated
> owner session/build was used.

Sequence deviation: independent source work on the remaining 43 continued
using already captured baselines while S2 runtime acceptance was deferred.
This follows the owner instruction to continue independent art; it does not
turn the pilot gate into a pass. No full suite, balance gate, release build,
performance campaign or other heavy acceptance was run.

The designated reviewer should resume the following only in a later authorized
validation round; do not automatically retry the contested slot in this run:

1. Compile the consolidated client under the same nonblocking host lock,
   two build jobs and existing target/pkg-config paths above. Run the union
   of client regressions once, using the same wrapper and
   `cargo test --offline --locked -p tbg-client --bin tbg-client <FILTER> --
   --nocapture --test-threads=1`. Filters: `p70_`, `fish`, `pyre_jet`,
   `kits_keep_the_simulation_event_times`, `art::kits::archer::tests`,
   `art::celebration::tests`, `custom_ring_follows_its_later_weapon_parent`.
   Read each result; formatting is not a type check and written tests may fail.
2. Freeze that exact successful binary and receipt before after capture.
   `scripts/p70/capture_roster.py --help` and `record_trial.py --help`
   define the reproducible CLI. Preserve originals, source/tooling hashes,
   viewport/quality and event journals; never label the frozen old binary
   as an after candidate. Reconstruct temporal before only from frozen art
   with reviewed tooling-only additions, not an edited champion.
3. Resolve the two known grip defects, run affected diagnostics and capture
   final five-form/static, temporal, live, Low/both-seat and auxiliary evidence.
   Populate every champion's paired links and review requirements truthfully.
   Check the normal gallery's Victory-to-Idle/Walk ownership during temporal
   review; static presentation does not establish this transition.
4. Measure/calibrate Trident, implement the narrow Proposed range change with
   its contracts and boundary tests, then queue D-33 baseline/candidate and
   held-out evidence. Heavy lab/gate commands still need the owner's separate
   scope authorization; thresholds are not test fixes.
5. Verify desktop browser/WASM and Windows against the shared assets in their
   relevant final environments. Native success alone proves neither. Obtain
   owner visual/fun and balance approval only with concrete final evidence.

Individual art commit hashes are recorded per champion in the linked machine-
readable files. All changes remain local and reviewable; **P70 is not complete**.
