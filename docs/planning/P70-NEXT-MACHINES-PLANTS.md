# P70 — Machines and Plants work front

Status: all 17 champion implementations complete at source level; runtime,
after captures and owner visual approval remain pending. This report records
changes, not an aesthetic or gameplay verification pass.

## Baseline and scope

Inspected the actual `derived-before/Machines.png` and `Plants.png`
montages and `before-cohort/<ID>/ally-threequarter/00.png` originals for
all 17 champions below. Each original contains five forms and four Idle
samples. The local files under `.local/p70-evidence/` are recorded through
`P70-VISUAL-EVIDENCE.json` and linked capture manifests. Content revision:
`55510c5363e3f1e555fa5133208a5faf20c5825b`; tooling revision:
`27f98acd98b51fa69eb09f4703bd2ca59a33ed41`.

Read all 17 model and pose sources, relevant P70/art/kit conventions,
`visuals/shell.rs`, `sync/special_cues.rs`, and Beetling, cargo crate and
drill auxiliary sources. Idle sheets informed the silhouette/color work;
Walk, primary, Victory and live abilities were inspected in source only.
No new render campaign ran on this front.

Deviation: the S2 runtime gate remains pending. The coordinator's second
consolidation attempt returned lock-busy 75. The coordinator explicitly
authorized continued independent art; compilation, runtime and captures
remain deferred. This is not a pilot runtime pass.

Edits are confined to these 17 kit directories and this report. Existing
central palette colors replace local color literals. No shared helpers,
shared palette, effects, authoritative rules or protocol were changed.
Existing form branches, attack declarations, duration/contact/release
contracts and special-event channels remain. Model proportions changed
locally where required for silhouette or authored hand reach.

## Implemented Machines

- **Sawbot:** baseline gray box body and small eye were weak at lineup
  scale. Added a cylindrical engine casing, brass eye socket/rim and dark
  brow; A rust and B iron masses retain existing saw radii and tips. Idle
  settles the clamp, Walk counterbalances the saw, primary uses a loaded
  low crosscut at 0.30 s, Victory presents the saw with a clamp salute.
  Existing spin axis, A2 twin saw and B2 horned hull remain.
- **SiegeWalker:** baseline bores read as solid caps. Added dark bore
  insets, broader leather harness and round belly volume. Existing
  shoulder carry, two-hand aim/crank IK and form muzzle/breech functions
  remain. Idle adjusts the load; Walk compensates gun weight; primary
  adds shoulder bracing; Victory raises the gun with both hands attached.
- **Rampart:** baseline blue rectangle resembled a window. Replaced it
  with a brass wall boss, fitted diamond heraldry and large bolts; added
  ogre brow and light gauntlet cap. Idle peers from the wall, Walk keeps
  it steady, primary uses a right-wrist punch at 0.30 s, Victory salutes
  with the free gauntlet. Chest parenting and left wall grip remain.
- **Bombard:** baseline mortar was readable but dark torso and goggles
  lacked separation. Added warm armor/apron region, brass panel and
  larger goggles. Raised/centered authored carry for reachable two-hand
  support and extended local arm length to 0.35 for A2 mouth reload.
  Idle checks the load, Walk steadies the mortar, primary retains brace,
  recoil and reload phases, Victory lifts the gun with both hands.
- **Oilpress:** baseline presses were small against generic shoulders.
  Enlarged flat press faces around unchanged contact offsets, broadened
  boiler hoops and added a rust furnace mass. Idle settles paired
  pistons, Walk carries rigid plates, primary loads and drives both
  presses at 0.30 s, Victory alternates contained pressure cycles.
  Existing Oil/Crush selection and tar/pressure forms remain.
- **CargoCrab:** baseline saddle and eye stalks were less legible than
  its orange shell. Added steel lens surrounds, dark pupils, saddle trim
  and dark jaw caps, using shared forge orange. Idle checks alternating
  pincers, Walk counterbalances claws, primary leads with the right claw,
  Victory alternates raised claws with a stationary Root. Six existing
  legs, all seat dimensions and the Cast boarding sequence remain.
- **Bulwark:** baseline tool nearly merged with feet and side heraldry
  looked detached. Added masonry seams, fitted team panels and distinct
  dome/metal values. Added two-hand tool support for load adjustment,
  short-step Walk, a 0.30 s tool plant and grounded Victory salute.
  The support point lies within the authored shaft; brace/reduction
  remains independently driven by the original special channel.
- **Demolisher:** baseline flat blue chest dominated hydraulic identity.
  Added a brass team boss/diamond, rust cuffs and light piston rods.
  Idle settles opposite pistons, Walk carries fists outside knees,
  primary drives the right hydraulic punch, Victory alternates pistons.
  Existing double-fist special, impact and radius remain unchanged.
- **Drillnest:** baseline feed rack disappeared into its chassis.
  Warmed the feed housing, exposed forward honeycomb mouths/drill noses
  and added a dark barrel inset. Idle cycles the feed carriage, Walk
  compensates chassis load, the existing special production cue tucks
  the head/chassis, Victory acknowledges with rack/barrel movement.
  Custom anatomy, track groups, disabled springs and `Attack::None`
  remain; no cosmetic basic shot or extra drill spawn was introduced.

## Implemented Plants

- **Bloomkeeper:** baseline leaf dress was rectangular and crown/staff
  flowers were small. Added rounded leaf torso/shoulders, hip petals
  and larger crown/staff flowers. Replaced the old sash, which intersected
  the new torso, with a fitted team leaf brooch. Idle offers a blessing
  palm, Walk counter-sways the offhand, Cast gathers/offers at 0.22 s,
  Victory keeps the staff upright while opening the free palm.
- **Rootcaller:** baseline rectangular trunk and straight limbs weakened
  the treant. Added knotted root lobes and broad moss beard, using shared
  bark/moss colors. Idle flexes asymmetrical wrists; Walk carries them
  as roots; the authored ground-cast track gains wrist follow-through;
  Victory opens branches from a planted stance. Per-form tips, long A
  claws and B mushroom/spore features remain.
- **Nester:** baseline nest and eggs hid behind the belly. Added a raised
  straw rim and visible eggs while retaining the nest parent and distinct
  hive/venom/blight forms. Extended arms to 0.20 for belly/seed gestures.
  Idle checks over a shoulder and pats the belly, Walk guards the nest,
  Cast offers the seed at 0.22 s, Victory sways protectively. Existing
  growth/hatch cues and Beetling creation remain authoritative.
- **Rotcap:** baseline full-sphere cap covered the mouth and the stem
  read as a humanoid. Replaced the main cap with a flat-bottomed domed
  Lathe profile, rounded stem/feet and enlarged dark mouth/lip. Idle
  compresses beneath the cap, Walk counter-tilts it, the original 0.18 s
  spit gains throat/head follow-through, Victory uses a planted cap
  orbit. The mouth-origin socket, gills and form clusters remain.
- **AntidoteBrewer:** baseline herbs were small and apron sunk into the
  body. Added fitted forward apron/team leaf, rounded sleeves and broad
  herb fans. Idle swirls while the free hand rests at the apron, Walk
  steadies the flask, Cast measures/offers at 0.22 s, Victory toasts with
  the flask and pats the apron. Existing Flask socket and A/B gear remain.
- **Shatterback:** baseline forepaws were raised and team hoop vertical.
  Lowered the torso, lengthened forelimbs, added rounded forearm/neck
  volume and moved head geometry forward. Fitted the team ring along
  the shell; strengthened existing plate marks without merging plates.
  Idle/Walk/primary/Victory use local ground-targeted forepaw IK; primary
  shifts into the right paw at 0.30 s, Victory keeps Root stationary.
  Four detachable plate entities and their Chest-space geometry remain
  separate. This requires real four-foot and plate-transition inspection.
- **Leechroot:** baseline dark box torso and thin roots obscured the maw
  beast. Added rounded root torso/lobes, broad gum rim and thicker root
  segments without extending contact tips. Idle curls one wrist, Walk
  trails roots, primary loads and lashes at 0.30 s, Victory curls both
  roots in a low sway. Bite/Drain and branch-specific maws/sacs remain.
- **Capybara:** baseline round ears/muzzle looked bear-like and B bars
  resembled shelves. Added a broad blunt muzzle, smaller ears/high eyes,
  rounded paws and curved side supports fitted around the belly. Arms
  extend to 0.20 for palm gestures. Idle sniffs, Walk uses the travel
  phase and relaxed paws, primary gives a planted palm bump at 0.30 s,
  Victory pats the belly. Existing custom Head/Tabard mappings, belly
  plate and independent SwampBelly/reduction channel remain.

## Source review and corrections

The independent machine/plant source review identified Bombard A2 reload
reach and Bloomkeeper sash overlap; both were corrected. Root review
identified a `Node` equality expression unsupported by its derives;
Shatterback now uses `matches!`. Leechroot/Shatterback Victory waves use
the exact 0.8 s TAU phase rather than an approximate multiplier.

Bombard reach was inspected using a source-equation numerical sweep of
the actual brace/recoil/reload interpolation: 1 ms over the 0.62 s clip,
71 elevation values over 0.55–0.90, and all five forms. Maximum requested
left spans were Base/B1 0.584511, A1 0.600533, A2 0.650350 and B2 0.616884;
right maximum was 0.323304. Carry pitch/lift envelope maximum was 0.549320.
The old 0.33 arm allowed 0.627 total span, insufficient for A2. The new
0.35 arm allows 0.665, giving approximately 0.01465 sampled margin.
Uniform form scaling preserves these relative distances. This calculation
is source-only, sampled evidence; it does not prove engine IK, exact
continuous extrema, grip contact or rendered clearance.

## Auxiliaries and deferred evidence

Beetling source was reviewed with Nester; its six legs, split shell,
mandibles and inherited forms remain untouched in its separate folder.
No auxiliary render or visual approval is claimed. Microdrill, Duodrill
and Megadrill were reviewed in Drillnest's folder and left unchanged;
real production/fusion must show the parent relationship and counts.
Crate belongs to Smuggler, not CargoCrab. CargoCrab requires real Boarded
passenger placement at its retained seat height.

Shatterback's original raw figure sheets omit the four plates spawned by
`visuals/shell.rs`; this is not evidence of a production defect. Live
plate counts and detachment need separate capture. Other remaining
high-value checks are Bombard muzzle/reload grip, A2 Sawbot disk clearance,
Rampart B2 fist/wall clearance, SiegeWalker crank/head clearance,
Bloomkeeper back-tree/staff clearance and Capybara belly/special overlap.

Validation level: source inspection and the specifically requested
Bombard calculation only, followed by final owned-file formatting and
whitespace checks recorded below. No Cargo compilation, tests, browser
build or after render ran on this work front. Cosmetic-only changes add
no redundant implementation-mirroring tests; applicable existing focused
checks belong to the coordinator's one consolidation run.

Commands/results: `rustfmt --edition 2021` on the 34 changed owned Rust
files exited 0. `git diff --check` scoped to the 17 kit directories and
this report exited 0 with no diagnostics. No build or test was run.

Deferred: compilation and focused consolidation checks; all five-form
after pose comparisons, temporal loops, aim/grips and melee contact;
real projectiles, specials, shell plates, boarding and auxiliary
transitions; normal match/grayscale/nameless lineups; desktop web and
applicable platform checks; owner recognition and beauty approval.
Deferred checks are not passes.
