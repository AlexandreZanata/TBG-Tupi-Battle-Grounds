# P70 execution prompt — all 46 champion identities and animations

Copy the following prompt to the executor. Repository documentation and
commits remain in English; communicate with the owner in Portuguese.

---

Execute **P70 — Champion identity, bespoke animation and 3D polish** in
`docs/phases/P70-champion-visual-identity-polish.md`, using
`docs/planning/P70-VISUAL-MANIFEST.json` as the fixed 46-champion cohort.
Read the phase and its Read first list before changing files.

Start from the 46-champion `codex/consolidated-linux-release` base
`55510c5363e3f1e555fa5133208a5faf20c5825b`, or a reviewed descendant which
preserves it, and include these planning files. Create the isolated local
execution branch `phase/p70-champion-visual-polish`. Never replace this
cohort with the older 25. Do not modify other agents' active worktrees,
overwrite owner edits or reset another front.

The goal is that a player recognizes each champion without reading its
name. Give **every one of the 46** its own recognizable silhouette,
equipment, deliberate color blocking and character-specific performance.
Polish every model; author personalized Idle, locomotion, primary
attack/ability and Victory, and review all other applicable clips and
transitions. Preserve the stylized medieval world. Merely recoloring a
generic body or changing the shared class animation's speed is insufficient.
Shared IK, meshes, timing and motion helpers should remain shared; the
visible performances must be individual.

Follow the phase in order:

1. Freeze source/tooling revisions and capture settings. Validate the 46
   IDs, 230 champion/form combinations and applicable animation matrix.
   Prepare reproducible evidence capture with correct branch selection
   and actual per-kit clip durations. Capture all baselines before shared
   art changes; never fabricate a before image from an already-edited kit.
2. Reproduce and diagnose the blue square/hole reported across characters.
   A shared blue tabard/patch is only a lead. Check geometry, transforms,
   normals, culling and overlap; fix the actual cause without removing
   team identification or hiding it through camera tricks.
3. Complete the three mandatory pilots:
   - **Trident:** make the shark taller, lengthen its trident and increase
     its real authoritative basic attack range above the current 1.1
     tiles. Replace the axe-like motion with a two-handed **frontal spear
     thrust**: load, brace, extend the points, contact and withdraw. The
     primary action must not be an overhead chop or sideways axe swing.
     Fix the longer trident entering its body. Check the grip, weapon socket,
     torso/head/fins, all five forms, basic/special attacks, walking,
     Victory, Hit/Death/Evolve and blends from several views.
     Record measured height/weapon length and a proposed integer range
     after calibrating visual reach. Update core content and GAME-DESIGN's
     Proposed value together; add both-seat boundary/form-inheritance
     regressions and queue required D-33 evidence. The exact balance value
     is pending evidence/owner acceptance, not implicitly approved by a
     visual screenshot. Keep damage, cadence, other stats and Bloody Rend
     radius/bleed unchanged. Continue independent art if balance evidence
     is deferred; do not claim the candidate release-ready.
   - **Pyre Warden:** make the nozzle visibly emit a recognizable flame
     jet synchronized with the existing ProfaneFlame event. Keep the exact
     directional ground segment, friendly-fire feedback, both seats and
     Low-quality readability. Particles never decide hits or damage.
   - **Old Finn:** make the projectile visibly a fish, with a readable
     silhouette and greater size in tier 1 and tier 2 on both branches.
     Preserve the existing release/flight/impact timing, trajectory
     clearance and Echo behavior. Do not slow combat for the screenshot.
4. Work through the remaining 43 individually. For each: existing identity
   brief -> unique model/colors -> bespoke poses/motion -> weapon/effect
   inspection -> five-form/all-applicable-clips review -> paired evidence
   -> logical local commit. Compare similar champions side by side.
   Review summons with their parent kits; the five auxiliaries do not
   inflate the 46-champion count.
5. Produce the consolidated final before/after index and per-champion
   contact sheets/temporal evidence, plus a nameless lineup at real match
   distance. Verify actual combat cues, Troops/training previews and P63's
   existing personalized Victory ownership, rather than only a studio pose.

Use meaningful individual colors with clear large regions. Preserve faction
identity while differentiating champions within the same faction. Add named
semantic colors centrally when needed and document the palette refinement
in ART-DIRECTION; do not use a global saturation change or turn every body
into team-colored/emissive armor. Proposed briefs guide implementation and
remain subject to owner visual approval; do not invent gameplay or lore.

All screenshots must come from the real game renderer. Save immutable
originals and metadata; label crops, annotations and comparison sheets as
derivatives. Include Base, A1, A2, B1, B2, applicable clips, actual
release/strike and temporal effect sequences. Account for all ten clip IDs;
explain real inapplicability instead of creating fake coverage. Store bulk
raw images/videos in ignored or established external artifact storage,
keeping compact review sheets and a complete linked index in the repository.

Keep all combat/server/protocol behavior unchanged, including damage,
cadence, target rules, friendly fire, hitboxes and event times, with
**one explicitly authorized exception: increase
Trident's basic attack range following the phase's calibration and D-33
acceptance process**. A longer-looking weapon alone does not meet that
request. Reuse the existing procedural Bevy 0.20 kits. Avoid new dependencies, per-frame
mesh/material creation, a giant per-character switch or a replacement
animation framework. Keep Linux, Windows and web on the shared artistic
assets; do not introduce platform-specific generic fallback characters.
Android/iOS and packaging/auth/deployment are outside this phase.

Apply the owner's current minimal-validation policy even if the selected
base still contains older broad acceptance commands. Formatting and one
cheap cached focused client check per final relevant front; queue named
regressions for one targeted consolidation. All compiling commands use
`flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock"` and
`CARGO_BUILD_JOBS=2`. A busy lock/cold cache is deferred without polling,
cache deletion or bypass. The requested baseline/final screenshot campaign
does not authorize a full suite, release build, packaging, balance gate or
unrelated performance campaign. Reuse evidence and recapture only affected
entries after relevant changes. Native success does not prove web/Windows.

Finish `docs/planning/P70-VISUAL-REVIEW.md`, the machine-readable evidence
index and roadmap status with actual coverage, paired before/after links
for all 46, confirmed root causes, Trident size/thrust/range proposal and
its balance status, decisions, local commits, commands run,
deferred checks, limitations and pending owner visual approvals. Never
claim the phase complete with missing champion evidence or unapproved
subjective results. Do all routine work before requesting aesthetic review.

Commit logical batches and stop for review. The current owner-supplied
AGENTS.md says never push; do not publish, force-push, merge or deploy.
If the owner separately authorizes GitHub publication, send this execution
branch as a PR targeting `main`, with the complete evidence/report and no
automatic merge.

---
