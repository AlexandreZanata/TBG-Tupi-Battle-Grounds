# P55 — Champion kits: every unrefined champion on the P54 standard

**Status:** implemented on `phase/p55-champion-kits` (not pushed); owner
look-and-fun review pending; workspace-wide checks and an online match
still to run before merging.
**Standard:** the P54 art kits (D-51) and the Stronghold defenders (D-52)
are the visual and motion standard. This phase inherits the fifteen new
champions (and the Smuggler's Crate) from `release/roster-online` and
rebrands every champion that still used the shared library, except the
three P54 champions and the towers.

## Goal

Every champion gets its own art kit: its own figure with five distinct
forms (base, A1, A2, B1, B2 — no recolors), its own idle, walk, attack or
cast, hit, death, spawn, evolve and victory motion, and its own projectile
or melee impact, in the P54 low-poly vertex-color style, inside the
doubled triangle budget. No game rule or number changes.

## Read first (only these)

- `docs/art/UNIT-ART-KITS.md` (the P54 process guide; §5 checklist)
- `docs/phases/P54-unit-art-kits.md` (kit format, timing, performance rules)
- `docs/ROSTER-EXPANSION.md` §C (roles and evolution branches)
- `crates/tbg-client/src/art/kits/` (`mod.rs`, the three P54 kits)

## Do NOT

- Touch the Siege Walker, Bone Brute or Rootcaller kits, or the towers.
- Change `tbg-core`, rule values, event times or the protocol.
- Replace the owner-approved Archer v2 figure (see *Deviations*).

## Design

### Shared kit library (`art/kits/common/`)

Twenty-three kits would repeat the same anatomy and motion blocks, so the
reusable parts live in one library; each kit keeps only what makes it
itself.

| Module | Holds |
|--------|-------|
| `common/mod.rs` | `Form { a, b, t2 }` from (tier, branch) |
| `common/body.rs` | `plan` (Humanoid/Large × size × 1.2 × Evolution growth), `humanoid` and `skeleton` anatomies, dress pieces (`tunic`, `tabard`, `cape`, `hood`, `helmet` with four `Helm` styles, `pauldrons`, `beard`, `glowing_eyes`, `bracers`, `greaves`) |
| `common/gear.rs` | weapons and props: `sword`, `hammer`, `kite_shield`, `staff`, `quiver`, `flask`, `sack`, `crate_box`, `lantern`, `bomb`, `strut` (a rod between two points: legs, pistons, strings); a piece is added when a kit first needs it |
| `common/moves.rs` | clip lengths (`DURATIONS`) and attack tables (`CHOP`, `SLASH`, `THRUST`, `THROW`, `CAST`, `POINT`) on the fixed event times; life blocks with styles (`idle`, `walk`, `hit`, `death` × `Fall`, `spawn` × `Entry`, `evolve` × `Power`, `victory` × `Cheer`); grips (`pole`, `shoulder_arms`, `staff`, `aim_level`); `toss` (THROW retimed to the cast Release) and `staff_cast` |

### Event times (unchanged, ART-DIRECTION §4)

Every kit uses the common 0.62 s attack length: Strike 0.30 s
(k 0.484), ranged Release 0.18 s (k 0.29), cast Release 0.22 s
(k 0.355). `kits_keep_the_simulation_event_times` checks every kit.

### Support actions

The Antidote Brewer, Bloodweaver and Puppetmaster act through the normal
`AttackStart` (cleanse, transfer and confuse are their attacks), so they
are shot kits: the flask flies to the cleansed ally, the blood thread to
the donor, the hex to the victim. Three actions have no `AttackStart` and
are cued in `sync/actions.rs`:
- a Crate `Spawn` → the seat's nearest Smuggler plays its toss;
- `Transfer { from, to }` → a blood stream from donor to receiver;
- `PlateBroken` → stone shards on the Shatterback.

## The kits

| Champion | Figure (base) | A branch | B branch | Attack | Shot / impact | Life clips (death · spawn · evolve · victory) |
|----------|---------------|----------|----------|--------|---------------|-----------------------------|
| Longaxe | halberdier, kettle hat, poleaxe | plate, plumed helm, gold crescent, red cape | bardiche → glowing halberd, horned helm | over-the-line chop (B alternates a sweep) | Cleave / Crush | Forward · March · Raise · Raise |
| Farwatch | slate-coat watchman, arbalest | scope → gilded arbalest, feathered hat, team cape | barbed bolts, scarf mask → glowing wound bolt, red eyes | kneel, aim, recoil, crank | Arrow | Back · March · Raise · Raise |
| Old Finn | oilskin fisherman, rod, creel | fuller creel → giant catfish, gold hook | net, second rod → net of fish, lantern | whip-cast of the rod | Fish / Splash | Kneel · March · Raise · Dance |
| Drumguard | red coat, white straps, shako, hip drum | plume, bigger drum → grand gold drum, standard | steel helm, plate → glowing drum runes | no attack; drums at a tempo | — | Back · March · Spin · Dance |
| Smuggler | tricorn, eye patch, crate frame, lantern | coin pouches, stacked crates → gilded chest, plumed tricorn, team cape | satchel, bandana → two crates, lantern pole | no attack; tosses each crate | — | Forward · Pop · Spin · Dance |
| Crate | banded crate, rope, team mark | coin sack → gilded corners, glowing lock | second crate → lantern | — | — | tips over · drops and bounces · — · stirs |
| Bombard | riveted ogre gunner, hip mortar, goggles | flared bell, bomb rack → great mortar, furnace pack | heavy plate, hull shield → twin barrel, ammo drum | brace, lob, recoil, reload | Bomb / Blast (Bombard muzzle) | Back · Drop · Burst · Dance |
| Oilpress | walking boiler press, piston arms, press plates | hull plate → bulwark, spiked anchor feet, lit furnace | oil barrels, drips → tar tank, tar-coated plates | two-plate slam | Crush (A) / Oil (B) | Kneel · Drop · Burst · Both |
| Cargo Crab | brass crab, passenger saddle, eye stalks, six legs | shell plates, spikes → shield dome, gilded saddle | racing stripe, leg pistons → afterburners, fin | pincer lunge | Bite | Back (belly up) · Pop · Spin · Both |
| Rotcap | mushroom-folk, spotted cap, ring collar | dark cap, toxin spots → venom cap, drips, green eyes | shoulder caps → cap cluster, spore pods | rear back and spit | Glob / Poison (from the mouth) | Sink · Rise · Burst · Dance |
| Leechroot | hunched root beast, leech maw, whip roots | wider maw → blood sacs | glowing veins → blood-bulb heart, tip maws | whip-root lash | Drain (B) / Bite (A) | Sink · Rise · Burst · Both |
| Antidote Brewer | sprout herbalist, leaf hood, apron, cure flask | gilded flask, herb crown → glowing still | flask bandolier → two flasks, cauldron | flask toss on the cast Release | Flask / Cleanse | Kneel · Pop · Raise · Raise |
| Shatterback | stone-bark beast, four mossy shell plates | thicker thorned plates → amber thorn tips | cracked plates on a core → shed plates, blazing core, spiked fists | two-fist pound | Shards (A) / Crush | Kneel · Rise · Burst · Both |
| Blood Furnace | skeleton knight, ribcage furnace, heated hammer | bigger fire, flame eyes → blazing sun, flame crown | furnace door, bellows → ember lanterns | two-handed chop | Burn | Sink · Rise · Raise · Raise |
| Bloodweaver | hooded skeleton priestess, spindle staff | chalice → heart reliquary | threaded arms → two spindles, orbiting drops | both hands raised, thread to donor | Blood | Sink · Rise · Spin · Both |
| Puppetmaster | top-hatted skeleton showman, marionette | hooked strings, armed puppet → glowing strings, mask | second puppet → chorus on the coat | points the control | Puppet | Back · Rise · Spin · Dance |
| Shieldbearer | man-at-arms, mail, tabard, kite shield, sword | bigger shield, plate → castle tower shield, great helm | longer blade, plume, cape → winged helm, glowing sword | slash round the shield | Crush (A) / Cleave (B) | Back · March · Raise · Raise |
| Hexer | hunched skeleton witch, pointed hat, skull staff | hex runes → doom crown, bigger skull | plague censer → plague beak, greater censer | staff thrust | Hex | Sink · Rise · Spin · Dance |
| Sawbot | one-eyed automaton, spinning saw hand | hot vents and teeth → twin saws, exhausts | plates → horned juggernaut, great saw | slash with the saw spinning faster | Sparks | Forward · Drop · Burst · Dance |
| Bloomkeeper | dryad, petal skirt, blossom crown, flower staff | shoulder blossoms, lit flower → blossoming tree | bark armor, thorns → thorn crown, roots, amber staff | staff thrust at the healed ally | Petal / Bloom | Kneel · Rise · Spin · Dance |
| Nester | plump moss mother, nest of eggs | bigger clutch → amber hive | green venom eggs → dark spiked blight eggs | overhand seed toss | Seed | Sink · Rise · Burst · Both |
| Beetling | small beetle, split shell, mandibles | amber bands (Brood) → honey shell (Hive) | venom spots (Venom) → spiked blight shell (Blight) | rear and bite | Bite | Back (flips) · Pop · Spin · Dance |
| Rampart | plated ogre with a mobile oak-and-iron wall | bigger wall → merlons, team banner | spiked wall → hot battering ram | punch round the wall | Crush | Kneel · Drop · Burst · Both |

## Phase report

### What was built

See *Design* and *The kits*. Commits (one champion or small group each):
the library and Longaxe; Farwatch, Old Finn and Drumguard; Smuggler and
Crate; Bombard, Oilpress and Cargo Crab; Rotcap, Leechroot, Antidote
Brewer and Shatterback; Blood Furnace, Bloodweaver and Puppetmaster;
the seven original champions and the support action cues; this report.

### Deviations

- **Archer kept on its owner-approved v2 figure.** It was already redone
  and approved; moving it onto a kit means replacing its bow-string and
  `arrows::release` path, which would double-fire if done halfway. Left
  for a dedicated follow-up.
- **Cargo Crab passenger is not drawn on the saddle.** The view does not
  pack the carried bit yet (`tbg-core/src/view.rs`, "P43: affliction and
  carrier bits are not packed yet"), so the client cannot tell who rides;
  the saddle stays empty. Needs a core view change.
- **Shatterback plates do not drop off one by one** for the same reason
  (plate count not in the view); `PlateBroken` shows stone shards instead.
- **Beetling forms** follow its Nester's tier and branch (summon.rs), so
  its five gallery forms are the Brood, Hive, Venom and Blight shells.
- **Separate build folder.** The worktree had shared the main checkout's
  `target/`; both checkouts produce the same build hashes for `tbg-core`,
  so another session's build on a different branch made this one compile
  against the wrong `tbg-core` (and could do the reverse to them). The
  worktree now builds in its own `target/` (dependencies already build
  without debug info, so it stays small).

### Verification (real output)

Run in this worktree with its own `target/` (see *Deviations*), at
`nice -n 19` and two jobs to keep the machine light:

| Check | Result |
|-------|--------|
| `cargo fmt --all -- --check` | clean |
| `cargo clippy -p tbg-client -- -D warnings` | clean (the first run flagged ten unused library items; they were removed, not silenced) |
| `cargo test -p tbg-client` | `test result: ok. 62 passed; 0 failed` (includes `every_kit_has_five_distinct_forms`, `kits_keep_the_simulation_event_times`, `every_kit_form_builds_within_the_doubled_budget`, `art/roster_tests.rs`) |
| `trunk build` (dev, wasm32) | `✅ success` — compiles `tbg-client` and `tbg-core` for the browser |
| `?gallery=<unit>&lineup` at 1600×900 | looked at for all twenty-three kits (headless Chrome, ANGLE GL); Bombard also in `&range` (bomb lob, smoke, blast) |
| `tbg-core`, `tbg-server` | no changes since the roster merge (`git diff d06279a HEAD -- crates/tbg-core crates/tbg-server` is empty) |

Not run: the workspace-wide clippy and tests (this phase changes only the
client, and the disk was at 99 % with 5.5 GB free, too little for the
server and balance builds), a release build, and a two-client online
match. Run them before merging.

### Triangles per form (`every_kit_form_builds_within_the_doubled_budget`)

Budget: 6 000 per form (3 000 doubled by the owner).

| Kit | Min | Max |
|-----|-----|-----|
| Longaxe | 818 | 1 116 |
| Farwatch | 880 | 988 |
| Old Finn | 1 280 | 2 014 |
| Drumguard | 952 | 1 292 |
| Smuggler | 876 | 1 468 |
| Crate | 204 | 476 |
| Bombard | 1 694 | 2 326 |
| Oilpress | 1 560 | 2 072 |
| Cargo Crab | 2 264 | 2 564 |
| Rotcap | 1 132 | 1 828 |
| Leechroot | 1 282 | 1 782 |
| Antidote Brewer | 1 400 | 1 956 |
| Shatterback | 1 320 | 2 488 |
| Blood Furnace | 2 418 | 3 186 |
| Bloodweaver | 2 782 | 3 740 |
| Puppetmaster | 2 596 | 3 528 |
| Shieldbearer | 926 | 962 |
| Hexer | 2 680 | 3 002 |
| Sawbot | 948 | 1 232 |
| Bloomkeeper | 2 210 | 5 412 |
| Nester | 1 036 | 1 276 |
| Beetling | 844 | 1 324 |
| Rampart | 876 | 1 016 |

### Evidence (`docs/media/p55/`)

One contact sheet per faction, each kit's five forms × idle, walk and
attack: `humans.jpg` (Longaxe, Farwatch, Old Finn, Drumguard, Smuggler,
Crate, Shieldbearer), `machines.jpg` (Bombard, Oilpress, Cargo Crab,
Sawbot, Rampart), `plants.jpg` (Rotcap, Leechroot, Antidote Brewer,
Shatterback, Bloomkeeper, Nester, Beetling), `undead.jpg` (Blood Furnace,
Bloodweaver, Puppetmaster, Hexer).

### Review notes (what changed after looking)

- Smuggler: bigger tricorn, lighter coat, a lantern you can see.
- Oilpress and Cargo Crab were too big for the lane; the crab's legs ride
  the thighs, so its feet are planted upright outside the walk.
- Leechroot read as a wooden robot; it got the long leech maw, coils,
  moss and thick whip roots.
- Shatterback's plates did not show from the front, then looked like a
  box; they are now rounded mossy shell plates rising behind the head.
- Rampart's wall lay along the forearm like a plank; it now stands
  upright on the chest and the left hand holds its grip.

### Still open

- Owner look-and-fun review of all twenty-three kits.
- Archer kit migration; Cargo Crab passenger and Shatterback plate loss
  (both need the view's affliction and carrier bits).
