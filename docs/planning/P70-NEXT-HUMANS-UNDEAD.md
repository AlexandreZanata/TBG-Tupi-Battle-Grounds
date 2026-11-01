# P70 Humans and Undead continuation

**Scope:** Shieldbearer, Archer, Longaxe, Farwatch, Drumguard, Smuggler,
Trigger, RattleTick, BoneBrute, Hexer, BloodFurnace, Bloodweaver,
Puppetmaster and Crowcaller. This front owns only those kits and this report.

**Sequence deviation:** the immutable 46-champion static originals exist.
The coordinator authorized independent S3 art after two consolidation lock
attempts returned 75. S2 pilot runtime acceptance remains pending. This is
not a waiver of pilot acceptance or evidence, and no pilot/core/shared file
is edited here. No compilation, tests, rendering or commits are authorized
on this front. Proposed visual decisions are not owner approval.

## Evidence and preserved contracts

Inspected `.local/p70-evidence/derived-before/{Humans,Undead}.png` plus
original `before-cohort/<id>/ally-threequarter` sheets: 02 for Shieldbearer,
Longaxe, RattleTick, BoneBrute and BloodFurnace; 03 for Archer, Farwatch,
Trigger and Crowcaller; 04 for Smuggler, Hexer, Bloodweaver and Puppetmaster;
01 for Drumguard; and 09 for Trigger, Crowcaller, Puppetmaster and
BloodFurnace. Each original primary sheet includes Base/A1/A2/B1/B2.
These static poses establish silhouettes and authored snapshots, not live
contact, continuous clearance, both seats or performance.

Read each named kit's model and pose code, BoneBrute motion, Archer's
existing bow regressions, shared rig/pose/gear conventions, the manifest
briefs, content Archetypes and P70 timing requirements. Retain melee Strike
0.30 s, ranged Release 0.18 s, Cast Release 0.22 s, existing flight and special
channels. Retain the five-form scale progression and all existing form
mechanics. Drumguard remains `Attack::None`. Smuggler's primary is its Cast
pistol release; rear crate placement is its independent `gesture` channel.
P63 continues to use each kit's Victory through the existing ownership.

## Individual dossiers

### Shieldbearer — `shieldbearer/{model,mod}.rs`

Observed: readable kite/tower shield, but the original broad chest cloth
competes with it and the raised shield nearly covers the face during the cut.
The S1 fitted-tabard correction is already integrated and must be retained.
Keep steel mail, linen and dark leather; introduce warm red cloth behind the
small team heraldry so the shield remains the principal team-colored shape.
A1/A2 keep heavier plate and tower merlons; B1/B2 retain the cape, plume/winged
helm and longer blade. Add an attached shield boss/rim or broad division,
not a floating rectangular chest panel. Compare Rampart and Bulwark.

Candidate motions: Idle checks the near shield edge with head/forearm motion;
Walk protects the torso with a lower shield and a restrained sword carry;
primary braces the shield before a sword cut around its outside at 0.30,
then returns the blade and shield together by 0.62. Victory brings the shield
to a salute, taps its outer rim with the sword and settles. Preserve the
shield arm's explicit pose; a sword event must not become a fabricated bash.
Review blade/shield/helmet overlap and all larger A2/B2 equipment.

### Archer — `archer/{model,mod,tests}.rs`

Observed: the longbow and quiver already read; full draw, upright bow,
string/nock connection and quiver recovery are authored. The original chest
panel and pale sleeves dominate the ranger's small head. Retain S1 cloth,
make Base's hood and shoulder cowl `HOOD_GREEN`, and keep linen torso,
warm timber bow, dark leather bracers. A uses greener ranger cloth plus
feather and gold greatbow; B uses a dark leather cowl and twin quivers.
Compare Farwatch and CelestialArcher; avoid metallic angel-like body armor.

Candidate motions: Idle briefly checks the string without adding a loaded
arrow; Walk keeps the bow upright while the head scouts ahead. Preserve
`aim`, `bow_rest`, `bow_string`, `bow_upright`, nock visibility and 0.18
Release. Refine the existing draw with a deliberate head/shoulder settle,
quiet anchor and restrained release follow-through, preserving bow alignment
and the quiver reach. Victory gives a small bow of the head/body with the
bow held beside the face, not both arms overhead. Existing tests stay intact.

### Longaxe — `longaxe/{model,mod}.rs`

Observed: the long poleaxe is strongly recognizable; its upright head is
cropped in the baseline framing. This is not evidence that its canonical
long-reach weapon should be shortened. Keep HAFT 2.40, HEAD_Y, impact point
and the existing two-hand swing/tilt path. The inherited support-target defect
is recorded below as an acceptance blocker. Use a warm padded `TRIM`/`RUST` gambeson,
visible quilt divisions, dark gloves and a steel blade face with a dark neck.
A preserves broad armor/crescent; B preserves twin blade/hook and horned helm.
Keep S1 fitted heraldry. Compare Shieldbearer, not the Trident thrust.

Candidate motions: Idle balances the upright shaft with a small wrist/head
check; Walk uses the authored shoulder carry with a settling counterbalance;
primary retains the two-hand overhead cut and B alternate fan sweep, with
stronger preparatory torso counterpose and 0.30 contact/0.62 recovery.
Victory balances the pole diagonally at the side and gives a grounded nod,
using the existing hand solver. Check full uncropped views before changing
length, head position or any gameplay impression of reach.

### Farwatch — `farwatch/{model,mod}.rs`

Observed: the arbalest is distinct from a bow but its narrow front span can
read as a rifle; Base/A/B dark coats blend together. Use `FIELD_SLATE` for
the main coat, linen collar and warm timber stock with brass mechanism.
Give the limb ends a readable backward angle and preserve MUZZLE and FORE.
A retains scope and feathered hat; B retains the mask, red fletching and
wine/dark cloak. Compare Archer and Crowcaller.

Candidate motions: Idle glances along the sight then lowers the weapon;
Walk retains braced carry rather than allowing the free arm to swing through
the stock. Keep `shoulder_arms`, fore-stock anchor, kneel, aim elevation,
0.18 recoil and 0.62 recovery; use the recovery's left hand for a visible
crank check only when it is clear of the stock. Victory lowers the arbalest
to a formal two-handed rest and nods. Avoid generic overhead gun waving.

### Drumguard — `drumguard/{model,mod}.rs`

Observed: red coat, crossed cream straps and drum are already strong. Source
uses a full opaque rim cylinder over the linen drum-head cylinder at the
same height; a ring rim would expose the playable membrane. Keep red/linen,
brass hoops, team shell/shako, A bass drum/banner and B iron armor/runes.
Fit the straps to the torso rather than broadening them. Compare Shieldbearer.

Candidate motions: keep alternating `drum` IK and `Attack::None`. Idle gives
a distinctive grouped snare pattern with a pause; Walk uses its existing
cadence and opposing head/chest beat. Attack previews remain a support
performance, never emit a new attack. Victory plays a double roll, crosses
the sticks above the exposed head and finishes with a downbeat; preserve
stick-to-head contact for every drum radius/depth. No generic cheer after
IK that would detach the sticks.

### Smuggler — `smuggler/{model,mod}.rs`

Observed: dark coat and cargo read, but the pistol action changes little
between the early samples. Keep dark brown coat, large red sash, warm crate
wood, cream collar and amber lantern; give the shoulder frame a visible
strap/attachment. A preserves money bags/gilded chest; B preserves lighter
satchel/bandana and the B2 pole lantern. Compare Nester and Drillnest.

Candidate motions: Idle alternates a short side glance with a crate check;
Walk leans against cargo weight while the independent left-hand support
remains attached. Cast loads the pistol at the hip, extends by 0.22, recoils
and returns by 0.62. Preserve the current `gesture` carry/lower function and
its body-only rear-placement semantics. Victory gives a shoulder shrug and
small lantern-side step with pistol safely down. Do not replace the Cast
shot with a crate spawn, or claim crate movement through a hand when the
visible model remains attached to Back.

### Trigger — `trigger/{model,mod}.rs`

Observed: orange feline face/tail and two guns distinguish him, but the tall
brown hat and brown torso dominate; Victory keeps the guns low in all sampled
poses. Source's final `arms` IK overwrites shared cheer arm movement. Keep
orange fur, cream muzzle/tail tip, dark fitted coat and readable steel/brass
revolvers; expose the ears above a lower hat crown/brim. A keeps long coat,
wide hat and spurs; B keeps reinforced firearms and darker gear.

Candidate motions: Idle tail flick and one guarded gun check; Walk quiet
heel-toe upper-body counterbalance. Preserve `TwinShot`, variant hand choice,
0.18 recoil and fourth-hit independent offhand cue. Victory passes its own
hand targets through `arms` and rotates the gun sockets for a small outward
spin, then lowers them; no shared cheer followed by overwritten arms.
Review gun/hat/tail overlap and both hand variants. Compare Smuggler.

### RattleTick — `rattle_tick/{model,mod}.rs`

Observed: the shoulder brother is legible but both figures use similar
brown/cream clothing; the attack raises Rattle's club while Tick remains
mostly upright. Keep Rattle's broad linen shirt/dark leather vest, give Tick
a red shirt/cap with small team bands, and brighten the club's metal bands.
A retains the big-brother vest/armor and larger club; B retains Tick's striped
shirt and explosive satchel. Do not erase the paired silhouette.

Candidate motions: Idle Rattle looks up as Tick points ahead; Walk keeps Tick
counterbalanced against the body roll. Primary keeps the club's 0.30 impact
and authored throw channel, adding a preparatory shoulder brace and Tick's
duck during the club passage. Victory alternates the brothers' waves, keeps
the club outside the shoulder seat and returns both to balance. Preserve
custom parents for head, two arms and carried bomb. Inspect bomb/hand
attachment and carrier/summon effects separately; never invent another hit.

### BoneBrute — `bone_brute/{model,motion}.rs`

Observed: large open ribcage, ivory skull and form-specific shield/cleavers/
scythe are strong. Preserve the exposed bone and soul rather than filling
the chest. Increase readable aged-bone/ivory separation at joints, darken
cleaver flats with a warm rust region and keep the small soul focal point.
A keeps armor and skull tower shield; B1 keeps stained apron/twin cleavers;
B2 keeps violet hood/scythe. Team armband remains secondary.

Candidate motions: retain the bespoke SLAM/SWEEP/BASH/DOUBLE/reap tables,
0.30 event, 0.62 duration, aim and planted feet. Idle gets a brief jaw/skull
rattle after the ribcage heave; Walk a delayed heavy head response. Refine
primary anticipation/head follow-through within the existing tracks instead
of replacing them. Add a dedicated Victory: lowered cleaver, ribcage roar,
then a deliberate weapon salute; B2 must keep `scythe` two-hand grip.
Check skull/shoulder plate/cleaver and every A2/B1/B2 alternate attack.

### Hexer — `hexer/{model,mod}.rs`

Observed: pointed hat and skull staff distinguish the witch, but Base/A/B
robes remain similar and the purple skull and robe merge at small scale.
Use violet robe, dark rag folds, pale skull and clear ritual focal point;
retain purple A hex runes and green B plague censer. Give B a moss-toned
shoulder mantle and retain B2 beak; A2 keeps crowned hat/larger skull.
Compare Bloodweaver and Puppetmaster. Avoid recoloring every effect green.

Candidate motions: Idle makes the lantern skull trace a small external circle
while the free hand beckons; Walk keeps the crooked hunch/staff support with
asymmetric head movement. Keep staff-based IK and Cast's 0.22 Release; author
a brief staff preparation then directed skull-first curse and return by0.62.
Victory turns the staff once through a safe shallow arc beside the body,
with a crooked bow; B's censer follows the free hand without crossing the
robe. Keep the long shaft outside the hat/face throughout blends.

### BloodFurnace — `blood_furnace/{model,mod,motion}.rs`

Observed: an iron-grated orange chest and tiny flame crest; exposed limbs
look more skeletal than armored. The primary/Victory sheets show the free
left hand separately raised, although the brief calls for two-handed hammer
handling. Source uses generic CHOP and a 0.52-duration kit over a shared
0.62-motion table: contact timing is near0.30, but recovery is truncated.
Keep aged bone, large dark-iron shoulder/hip masses, orange hot center and
small gold core. A progresses to sun rays/crown; B to closed door/bellows/
ember lanterns. Compare Pyre Warden's open brazier, not its pole-like form.

Candidate motions: Idle furnace exhale, Walk hammer-weighted chest sway,
primary two hands on one measured haft with a braced downward blow at0.30
and complete recovery at0.52. Victory braces the hammer beside the furnace
and lifts the head with one chest flare. If existing short arms cannot reach
the second grip, adjust coherent model/anchors together; do not request
unreachable IK. Preserve Burn impact, tip, duration, fire mechanics and core.

### Bloodweaver — `bloodweaver/{model,mod}.rs`

Observed: wine robe and spindle are distinct; the generic Cast raises both
arms then holds them straight forward, including the chalice/second spindle.
Use wine cloth, darker folds, pale bone and a thin bright thread region;
make the spindle's winding and pale end caps legible. A keeps gold chalice/
heart reliquary; B keeps forearm threads and B2 second spindle/orbiting drops.
Compare Hexer's crooked hat and Puppetmaster's divided tails.

Candidate motions: keep existing circular hand IK for Idle, add opposed
spindle turn; Walk carries spindle upright and free hand close to the robe.
Cast gathers between both hands, leads from `spindle_top` at0.22 and follows
with an outward free-hand weave, recovering by0.62. A must keep its chalice
upright; B2 uses two supported staff poses rather than a generic palms-out
push. Victory winds an external figure-eight around the spindle and bows.
Preserve BloodBolt, Siphon target semantics and ally transfer channel.

### Puppetmaster — `puppetmaster/{model,mod}.rs`

Observed: violet tails/top hat and marionette clearly identify the showman.
The Cast lifts the puppet upside down; Victory tilts its whole rigid control/
strings/doll assembly sideways. Use violet coat, ivory waistcoat/bone hands,
dark hat and warm wooden cross with light strings and red doll. A retains
armed puppet/half-mask/magic strings; B retains second doll and B2 chorus.
Compare Hexer and Bloodweaver without adding a wizard staff.

Candidate motions: preserve `work` control IK and B's offbeat second control.
Idle makes a brief puppet greeting through controlled height/roll; Walk keeps
the cross approximately horizontal. Cast prepares by raising the control
slightly, points the cross toward target at0.22, then settles; counterrotate
the wrist so the doll hangs beneath instead of inverting. Victory is a
curtain-call bow with both controls held forward. Dolls remain rigid mesh
assemblies; do not claim articulated limbs or create a new simulation.
Check doll/strings against coat, knees and the second puppet.

### Crowcaller — `crowcaller/{model,mod}.rs`

Observed: pale beak and compact ritual crossbow distinguish him, but the
raven merges with the dark shoulder and Victory keeps the same low grip.
Use charcoal coat with separated slate folds, pale mask, warm brass latch
and cool raven feather edges; retain small green ritual glow. Make the
raven's body/beak/wing silhouette legible without enlarging it into another
champion. A preserves feather mantle/long beak/censer; B bone arm armor and
reinforced bow. Compare Farwatch and B2 Hexer.

Candidate motions: Idle turns the mask toward the raven, which looks back;
Walk counterbalances raven head against the stalking trunk. Preserve
`loaded`, custom Nock parent, two-hand grip, 0.18 Release and independent
marked-shot wing channel. Refine the existing latch preparation/recoil and
return instead of a generic ranged gesture. Victory supplies low, secure
crossbow targets, a shallow bow and raven wing salute; it must survive the
final grip pass. Keep the raven's feet on the shoulder and mask off the stock.

## Implementation and validation ledger

All fourteen candidates are implemented in source. The earlier dossiers record
intent and baseline observations; the entries below describe the actual result.
No after image, temporal review, compilation or runtime test exists for this
front yet. These candidates are not accepted art.

- **Shieldbearer:** steel mail, red tunic, linen hem and a brass kite-shield boss
  frame the retained fitted team tabard. Idle inspects the shield, Walk keeps a
  low protective carry, the existing sword cut gains shield bracing, and Victory
  uses an authored shield-and-sword salute. The intended rim tap still needs
  contact review; it is not claimed as measured contact. A2 retains its tower
  construction and all branches retain their existing equipment and Strike.
- **Archer:** a green Base hood and fitted shoulder mantle separate the face,
  linen and leather; B keeps a darker cowl. Idle checks the string, Walk scouts,
  the existing primary draw gets a head anchor, and Victory bows with an upright
  side-held bow. Existing arm IK, string/nock handling, draw, release, recovery,
  socket logic and regression tests are preserved.
- **Longaxe:** a rust-and-linen padded gambeson, shoulder padding and existing
  fitted heraldry distinguish the infantry silhouette. Idle balances the haft,
  Walk retains shoulder carry with a free-arm counterbalance, primary retains
  the authored chop/B sweep with added anticipation, and Victory is deliberately
  a one-hand pole salute with a separate free-hand nod. HAFT 2.40, HEAD_Y and
  impact anchor are unchanged. Its inherited ready/primary offhand defect is
  an explicit acceptance blocker below.
- **Farwatch:** slate coat, cream collar and outward crossbow limb tips strengthen
  the arbalest silhouette. Forestock support is shortened to 0.14 local Z and
  arm segments grow from 0.20 to 0.23, with centered two-hand wrist targets.
  Idle inspects the sight, Walk supports the crossbow with both hands, primary
  retains its kneel/aim/recoil, and Victory makes a restrained low-stock bow.
  The muzzle and shot/arc contract remain unchanged. Source reach estimates do
  not establish live muzzle trajectory or stock/head clearance.
- **Drumguard:** cream cuffs and exposed linen membranes sit between thin brass
  ring rims. Arms grow from 0.17 to 0.21. Existing form-specific drum dimensions
  now drive tip contacts in hip space, transformed into chest space before IK;
  stick tip length is 0.23. Idle marks the beat, Walk marches, and Victory adds
  a paired lift and finishing roll. Victory uses four cycles per 0.8-second loop
  (five for A), avoiding the reviewed wrapped-clock seam. It remains
  `Attack::None`: drumming is visible performance, not a fabricated attack.
- **Smuggler:** rust coat, charcoal hat and a cream cargo strap/buckle support the
  existing crate identity. Arms grow from 0.17 to 0.20. Idle checks cargo, Walk
  carries the pistol low, primary draws then releases at 0.22 before recoil,
  and Victory gives a cargo-weighted shrug. The chest coil is zero at release,
  avoiding an added barrel yaw. The separate `gesture` still lowers the crate
  arm; its target excursion is shortened to stay within the intended reach.
- **Trigger:** slate/charcoal coats, smaller hat dimensions and exposed ears
  separate the rat from other shooters. Idle inspects one pistol, Walk retains
  heel-toe swagger, primary adds a timed draw before the existing alternating
  shot, and Victory performs two staggered outboard pistol rolls. Final arm IK
  explicitly includes the Victory targets. TwinShot variants and the independent
  special cross-fire channel remain in their original ownership.
- **RattleTick:** Tick gains red cloth, a dark seat belt and linen strap; club bands
  read as steel. Idle exchanges a look, Walk gives Tick a synchronized seated
  counterbalance, the existing club primary adds Tick's brace/duck, and Victory
  alternates the brothers' waves. The existing secondary rig and special throw
  channel remain intact; Tick's wave is not an additional projectile event.
- **BoneBrute:** alternating pale ribs, dark cleaver faces, larger rust regions and
  jaw gaps reinforce the skeleton/weapon separation. Idle adds a skull rattle,
  Walk delays the head counterturn, and the existing SLAM/SWEEP/BASH/DOUBLE/reap
  tracks receive a small head brace. Victory opens the ribs for a short roar,
  preserving shield/dual-cleaver forms. B2 Victory deliberately holds the scythe
  in one hand. Its older two-hand rest/reap defect is an acceptance blocker below.
- **Hexer:** violet robes and dark folds, a moss B mantle and small bright eyes in
  dark skull-staff sockets preserve the witch identity. Idle circles the skull,
  Walk supports the hunch, primary authors a skull-first curse at 0.22 with full
  0.62 recovery, and Victory makes a crooked staff arc and bow. Hex, its muzzle,
  A runes, B plague censer and all life clips retain their existing contracts.
- **BloodFurnace:** dark pauldrons/cuffs, iron greaves, aged bone and a bright orange
  center separate armored mass from heat. Arms grow from 0.17 to 0.25; two wraps
  mark actual 0.00/0.14 positions on the unchanged +Z hammer haft. A separate
  `motion.rs` authors furnace breathing, weighted Walk, a two-handed strike at
  0.30 with recovery ending at the existing 0.52, and a braced furnace Victory.
  Burn impact, tip, duration and authoritative mechanics remain unchanged.
- **Bloodweaver:** wine cloth, darker B folds, a cream A collar and pale spindle
  winding separate cloth from the bright blood thread. Idle retains circular IK,
  Walk carries the spindle upright, primary gathers and sends from the existing
  spindle at 0.22, and Victory winds a small external figure-eight and bows.
  A's chalice receives counterrotation; B2 keeps its second spindle. The Siphon
  transfer channel and BloodBolt are unchanged.
- **Puppetmaster:** showman violet, ivory lapels/strings and a brass cross center
  improve the rigid puppet assembly's separation. Idle adds a puppet greeting,
  Walk reduces control tilt, primary leads with the cross at 0.22, and Victory
  counterrotates the control through a curtain-call bow. A and B equipment remain
  distinct. Strings/dolls remain existing rigid meshes; no articulated puppet or
  new simulation is claimed.
- **Crowcaller:** slate shoulder and raven feather planes, a narrower team tail
  and pale beak preserve the ritual crossbow identity. Arms grow from 0.20 to
  0.21; support/latch targets now share the stock axis. Idle exchanges a raven
  glance, Walk counterbalances its perch, primary keeps the latch/recoil timing,
  and Victory gives a low crossbow bow and wing salute. Custom Nock parenting,
  0.18 release/0.52 loaded restoration and marked-shot special wing channel stay
  unchanged.

### Known acceptance blockers from source geometry

The requested target arithmetic below uses the existing `tools::reach` contract:
shoulder at `(±(torso.x/2+0.05), torso.y*0.9, 0)` and maximum arm reach
`arm + 0.9*arm`. It evaluates the source formulas, not actual `pose_world` output
or runtime mesh clearance. Uniform form scale does not fix an unreachable local
anchor.

1. **Longaxe ready/primary support is inherited and unreachable.**
   `longaxe/mod.rs::pose` calls `common/moves.rs::pole` with swing -15 degrees,
   tilt 20 degrees and support span 0.32 at full ready. This requests left wrist
   `(-0.21556, -0.20070, 0.11954)` from shoulder `(0.23, 0.306, 0)`: distance
   **0.68524**, versus available **0.361**. The primary uses the same helper
   with span 0.34. Per coordinator instruction, its existing weapon path remains
   unchanged in this candidate batch. The new Victory uses reachable one-hand
   targets and does not call the helper. Acceptance requires a local grip/path
   correction followed by all-form `pose_world` and temporal shaft/body review.
2. **BoneBrute B2 rest/reap has the same inherited pattern.**
   `bone_brute/motion.rs::scythe`, called at rest with swing -20 degrees,
   tilt 18 degrees and span 0.26, requests left wrist
   `(-0.28789, -0.16727, 0.16537)` from shoulder `(0.30, 0.342, 0)`: distance
   **0.79519**, versus available **0.475**. Rest/reap trajectories remain authored
   as before. New Victory uses a deliberate one-hand salute. Acceptance requires
   measuring and correcting B2's older two-hand path without flattening its
   reaping motion or changing the existing Strike event.

Both are concrete known defects, not a generic unverified-clearance note. The
batch does not claim all-clip grip safety. Their future corrections must include
actual shoulder/wrist/shaft transforms and the affected meshes across the full
clip, because a reachable endpoint alone does not prove body clearance.

### Validation and consolidation handoff

Validation level: source review and formatting only. The independent Human
review caught the new Longaxe Victory offhand target, Drumguard Victory loop
seam and Smuggler release yaw; all three have been corrected. The subsequent
source pass identified the two inherited blockers above and removed the scythe
helper from the new BoneBrute Victory. Existing Archer regressions were not edited.

Final formatting command: `rustfmt --edition 2021 --config skip_children=true`
over the 29 changed/new Rust files in this front only. Actual output:
`rustfmt owned Rust files: 29; exit 0`. No shared modules or tests were formatted.
Scoped whitespace results are recorded at the end of this report.
No Cargo command, rendering, browser check, benchmark or test was run on this
front. Compilation remains deferred under the coordinator's host-lock policy
after two exit-75 attempts; this front did not retry or poll that lock.

The minimum future automated checks belong to the designated consolidation
reviewer, once on the integrated tree. Reuse its single targeted client compile;
queue only the existing bow regression subset for the changed Archer boundary:

```sh
flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" \
  env CARGO_BUILD_JOBS=2 cargo test -p tbg-client --bin tbg-client \
  art::kits::archer::tests -- --test-threads=1
```

No broad suite is requested. In addition to resolving the two known grip defects,
focused after/temporal review must cover the fourteen champions' five forms and
Idle/Walk/primary/Victory, with the critical event neighborhoods (0.18 ranged,
0.22 Cast, 0.30 melee, and Furnace recovery at 0.52), support wrists, stock/face,
shaft/body, drum contacts, chalice orientation, puppet hanging direction and the
secondary/special channels called out above. All life-clip transitions, both
seats, Low/reduced-motion browser, Troops/training/Result, nameless/grayscale
lineup and performance evidence remain pending in the coordinator's acceptance
backlog. Static originals and source arithmetic do not satisfy those checks.

Final whitespace output: scoped `git diff --check` returned 0 with no findings.
`git diff --no-index --check /dev/null` on the new Furnace motion and this report
printed no whitespace findings; each returned 1 because the new file differs
from `/dev/null`. No compile/test or rendered validation is implied by these
source-only results.
