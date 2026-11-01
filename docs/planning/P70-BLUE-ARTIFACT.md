# P70 S1 — Blue rectangle diagnosis and fitted heraldry

**Status:** shared clothing correction implemented; final-renderer inspection,
physical secondary motion, transitions and owner aesthetic approval pending.
This is not a claim that every reported opening across the roster has been
resolved. The shared helper serves four champions, not all 46.

## Evidence inspected

The immutable static campaign uses content `55510c5` and tooling `27f98ac`.
It samples actual game meshes through `Animator::present`, with secondary
springs at rest equilibrium. Its images establish sampled poses, not live
cloth dynamics, ability effects or real-time transitions. Full capture
settings and hashes are in the original campaign manifests:
[cohort metadata](../../.local/p70-evidence/before-cohort/run.json) and
[enemy-view metadata](../../.local/p70-evidence/before-s1-enemy/run.json).
These local ignored artifacts require the execution worktree and are not
included in the repository. Do not substitute generated or edited pixels.

Direct observations:

- Archer's chest rectangle is blue on the
  [ally Idle sheet](../../.local/p70-evidence/before-cohort/Archer/ally-threequarter/00.png)
  and red on the same-camera
  [enemy Idle sheet](../../.local/p70-evidence/before-s1-enemy/Archer/enemy-threequarter/00.png).
  The backdrop stays blue. The gold hem and lozenge remain in the same
  places. This confirms opaque team-colored geometry, not a hole exposing
  the background, for that observed rectangle.
- The [Archer profile](../../.local/p70-evidence/before-s1-enemy/Archer/enemy-profile/00.png)
  shows the shallow front plate in profile. The
  [back view](../../.local/p70-evidence/before-s1-enemy/Archer/enemy-back/00.png)
  shows the torso and quivers, plus the separate tier-2 capes. The front
  rectangle is not a missing torso surface visible from every direction.
- Shieldbearer has the same hem/lozenge rectangle in
  [Idle](../../.local/p70-evidence/before-cohort/Shieldbearer/ally-threequarter/00.png).
  The [AttackMelee sheet](../../.local/p70-evidence/before-cohort/Shieldbearer/ally-threequarter/02.png)
  shows the chest/armor sweeping through its plane as the upper body turns.
- BoneBrute's [ally view](../../.local/p70-evidence/before-cohort/BoneBrute/ally-threequarter/00.png)
  shows an upper-arm team rag and a cape hem. The
  [enemy front view](../../.local/p70-evidence/before-s1-enemy/BoneBrute/enemy-front/00.png)
  shows the red arm rag and intentionally open ribcage. These are separate
  geometry from the shared tabard, and are not changed by this fix.
- Hexer's [Idle sheet](../../.local/p70-evidence/before-cohort/Hexer/ally-threequarter/00.png)
  shows a small independent hip ribbon, not the large shared rectangle.
- Trident's [enemy profile](../../.local/p70-evidence/before-s1-enemy/Trident/enemy-profile/00.png)
  shows red hip strips and a separate torso band. Its shaft visibly enters
  the body in these baseline poses; that belongs to the Trident pilot.

## Confirmed mechanism and limits

`common/body.rs::tabard` previously built a single thin, full-length cuboid
on `Secondary(Spring::Tabard)`, plus a gold hem and lozenge. The secondary
node is parented to **Hips**; the visible upper torso is parented to
**Chest**. A single rigid front card therefore could not follow upper-body
turns and breathing while also hanging from the hips. The plain rectangle
and contrasting flat blue surface made the separate clothing especially
conspicuous. It is the exact source geometry observed on Archer and
Shieldbearer.

Source inspection checked these possible alternatives:

- `Shape::Box` creates a closed Bevy Cuboid; no face is intentionally
  omitted by this helper.
- `mesh::paint` duplicates indexed vertices, recalculates flat normals and
  writes alpha 1.0. The shared material is opaque and retains normal
  culling behavior.
- `figure::build_layer` transforms and merges each part into its joint
  mesh. Team replacement changes `ALLY` to `ENEMY` before building enemy
  joint meshes; it does not delete faces or change transforms.
- The inspected part scales are positive. No negative `.scale([...])`
  literal was found in the kit/roster source search.

These checks and observations support a clothing geometry/parenting
correction. They do not prove that every other kit has correct normals,
closed geometry, depth clearance or team visibility. No global culling,
sky, camera, saturation or lighting change is part of the correction.
No hidden-node or filtered-part diagnostic capture has yet been made;
the enemy-color comparison provides the current direct material isolation.

## Implemented correction

The shared helper now receives the actual `Proportions`. A short tapered,
faceted heraldic panel follows **Chest**, with a gold edge, small lozenge
and shoulder loops. Its width is bounded to 60% of the torso width, and
its chest height fits the torso. The panel follows the torso's front
slope instead of spanning the chest and hips as a rigid rectangle.

Two short tapered cloth ends remain on the hip spring, below the waist.
This retains secondary movement without making the entire chest garment
move independently of the chest. Blue/red team heraldry remains visible
on the panel, shoulder accents and lower cloth. Materials use existing
central palette constants.

The four affected kits are Archer, Shieldbearer, Longaxe and Solar Sentinel,
including Base/A1/A2/B1/B2. Their individual weapons, other clothing and
Evolution pieces are retained. Solar Sentinel explicitly supplies its
breastplate front at local z=0.17; the existing sun disk is shifted forward
to z=0.215 so it remains in front of the fitted heraldry. This is an art
surface clearance, not a gameplay size or hitbox change.

The helper improvement alone does not satisfy those four champions'
individual P70 model/color/motion review. Their bespoke identity and
animation requirements remain separate.

## Validation and handoff

Validation level: **work front**.

Completed on the working-tree changes:

```bash
rustfmt --edition 2021 crates/tbg-client/src/art/kits/common/body.rs crates/tbg-client/src/art/kits/archer/model.rs crates/tbg-client/src/art/kits/shieldbearer/model.rs crates/tbg-client/src/art/kits/longaxe/model.rs crates/tbg-client/src/art/kits/solar_sentinel/model.rs
git diff --check -- crates/tbg-client/src/art/kits/common/body.rs crates/tbg-client/src/art/kits/archer/model.rs crates/tbg-client/src/art/kits/shieldbearer/model.rs crates/tbg-client/src/art/kits/longaxe/model.rs crates/tbg-client/src/art/kits/solar_sentinel/model.rs docs/planning/P70-BLUE-ARTIFACT.md
```

Both completed successfully. No Cargo build/test, new screenshot campaign,
triangle-count measurement or performance run was executed by this front.
The integrated client build belongs to the coordinating reviewer.

Deferred observations: final matching static sheets for all 20 affected
forms; both-team front/profile/back views; live spring behavior and
chest/hip transitions in Idle, Walk, actual attacks, Hit, Death, Spawn,
Evolve and Victory; team readability from the desktop match camera;
Solar Sentinel's sun-disk/armor overlap; final mesh budgets and draw calls;
desktop web and Windows observation. Static before evidence alone never
establishes that the new design passed these checks.
