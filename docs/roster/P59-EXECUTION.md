# P59 — Complete character delivery plan

Owner authorization: 2026-10-31. Deliver all 21 character kits and catalog
abilities, with one commit per character, one real local screenshot when each
character finishes, and one new PR. The owner clarified screenshots, not prompts.
Base: PR #20 at 3287410. Branch: phase/p59-complete-roster, stacked on
phase/roster-expansion for owner consolidation. Preserve the active native front.

## Delivery contract

- GAME-DESIGN 13 owns all Proposed values; P59-NAMES owns English names;
  P58-CATALOG owns the visual/motion brief. Do not invent balance numbers.
- Every character owns model, five forms, motion and ability cues under its own
  art/kits directory. Reuse the shared palette, rig and materials, not a generic
  recolored character. Growth is 1.00 / 1.08 / 1.16, with major surface changes,
  silhouette attachments and the final branch artifact.
- Server-authoritative mechanics preserve targeting, windup, status caps,
  shields, DoTs and reconnect facts. Cosmetic movement never decides damage.
- Add meaningful regression scenarios with exact deferred commands. Never weaken
  contracts or gates. Any wire change carries its decision and protocol bump.
- Each character commit includes model/forms/motion/mechanics/cues, its status
  and actual desktop screenshot under docs/media/p59/<kind>/lineup.jpg.
  Never present AI concept art or a fallback figure as the finished model.
- R21 includes Microdrill, Duodrill and Megadrill. Public Shop/lab pools remain
  unchanged until W11 approval; no VPS deployment is requested for this PR.

## Sequence

Record planning and narrowly necessary shared art-review plumbing first.
Then finish R08, R09, R10, R11 (the existing Monsters pilot), followed by R01–R07,
R12–R19, R20, R21. Commit each character and show its screenshot before moving on. Push commits
to the draft PR at useful milestones without duplicating its entire CI workload. Consolidate affected tests once after integration; full suites and
balance matrices require an explicit owner request.

## Validation and resources

The updated AGENTS.md and docs/VALIDATION.md in the active checkout govern this
work even while uncommitted there. Use minimal scoped checks and existing caches.
Every compiling validation/preview uses the nonblocking shared host lock
/tmp/tbg-validation-$(id -u).lock, CARGO_BUILD_JOBS=2, and --test-threads=1 for
any authorized tests. Exit 75 is deferred; no retry/polling, new target directory,
cache deletion or interruption of another build. The requested screenshots
permit necessary preview builds, not unrelated suites or benchmarks.

## Character briefs

### R01 — Bulwark

Machines / Tank. **Ability:** Living Wall, using GAME-DESIGN 13.2–13.3.
Own library: `crates/tbg-client/src/art/kits/bulwark/`.

broad stone dome over compact humanoid legs, visible face/tool, iron bands, oak straps and brass Machines fasteners. Use a large team-colored cloth panel on the shell. A2 adds shell buttresses and warmer stone; A3 adds a crenellated back crest. B2 exposes iron forearms and a larger tool; B3 adds a flat stone impact shoe. Reuse Large anatomy, stone/metal palette and weapon socket; no walkable Stronghold on its back.

Status: implemented; targeted consolidation passed and actual desktop five-form
lineup observed. Evidence: `docs/media/p59/bulwark/lineup.jpg`.

### R02 — Demolisher

Machines / Tank. **Ability:** Area Crush, using GAME-DESIGN 13.2–13.3.
Own library: `crates/tbg-client/src/art/kits/demolisher/`.

squat broad humanoid, fists almost ground-sized, two piston cuffs and a compact pressure tank; iron/brass/rust, team cloth at waist/back. A2 thickens fist plates, A3 adds broad hammer palms; B2 exposes cyan-free forge-orange valve guards, B3 adds asymmetric shock coils. Reuse Large limbs and mechanical mesh parts; valves are rigid props.

Status: implemented; directed core consolidation passed and real desktop lineup
observed. Evidence: `docs/media/p59/demolisher/lineup.jpg`.

### R03 — Crowcaller

Undead / Ranged. **Ability:** Crow's Mark, using GAME-DESIGN 13.2–13.3.
Own library: `crates/tbg-client/src/art/kits/crowcaller/`.

narrow plague-mask silhouette, torn coat, short ritual crossbow, one shoulder raven and relic pouch; charcoal, bone, tarnished bronze and muted plague green with team lining. A2 adds feather mantle and green vial panel, A3 a longer beak/censer. B2 adds rigid crossbow arms and pale sleeves, B3 a large ruined sight. Raven is a shoulder/back prop, not Unit.

Status: implemented; seven R03 scenarios passed in the 58-test combined block
and actual desktop lineup observed. Evidence: `docs/media/p59/crowcaller/lineup.jpg`.

### R04 — Trigger

Humans / Ranged. **Ability:** Crossfire, using GAME-DESIGN 13.2–13.3.
Own library: `crates/tbg-client/src/art/kits/trigger/`.

small orange feline, huge hat, two revolvers, booted legs and curved tail; leather/oak/iron and Human gold, team neckerchief. A2 raises hat brim and adds blue/red long coat, A3 swept tail guard/spurs. B2 adds wide brass barrel collars and dark vest, B3 twin angular sights. Keep weapons in this supplied mercenary fantasy; no imported protected asset.

Status: implemented; targeted consolidation passed and actual desktop lineup
observed. Evidence: `docs/media/p59/trigger/lineup.jpg`.

### R05 — Rattle & Tick

Humans / Tank. **Ability:** Double Mayhem, using GAME-DESIGN 13.2–13.3.
Own library: `crates/tbg-client/src/art/kits/rattle_tick/`.

square heavy torso, oversized nailed club, small thin brother above one shoulder, bomb backpack; leather/linen/iron with broad Human team sash. A2 expands chest straps and club head, A3 iron shoulder guard. B2 enlarges Tico's satchel and striped coat, B3 adds visible bomb rack. Do not reuse Cargo Crab boarding or add another Unit.

Status: implemented; targeted consolidation passed and actual desktop lineup
observed. Evidence: `docs/media/p59/rattle_tick/lineup.jpg`.

### R06 — Capybara

Plants / Tank. **Ability:** Swamp Belly Flop, using GAME-DESIGN 13.2–13.3.
Own library: `crates/tbg-client/src/art/kits/capybara/`.

huge rounded belly, short thick limbs, capybara muzzle/ears, leaf necklace and fiber skirt; warm ochre hide, bark/moss, team cloth on pack. A2 stone-like belly guard and darker flanks, A3 broad leaf gorget. B2 visible belly bands and pale fiber skirt, B3 a wide bark waist disk. Rounded low-poly volumes can reuse Large torso/limb primitives.

Status: implemented; targeted consolidation passed and actual desktop lineup
observed. Evidence: `docs/media/p59/capybara/lineup.jpg`.

### R07 — Trident

Plants / Tank. **Ability:** Bloody Rend, using GAME-DESIGN 13.2–13.3.
Own library: `crates/tbg-client/src/art/kits/trident/`.

broad shark head, dorsal fin, muscular legs, rusted trident and large chain shapes; slate hide, bone teeth, rust/bark and team sash. A2 shell cuirass and pale belly, A3 large dorsal armor; B2 wider trident and red-brown wrap, B3 crescent side tines. Chains merge into torso mesh.

Status: pending. Evidence: `docs/media/p59/trident/lineup.jpg`.

### R08 — Bonehorn

Monsters / Tank. **Ability:** Primordial Carapace, using GAME-DESIGN 13.2–13.3.
Own library: `crates/tbg-client/src/art/kits/bonehorn/`.

low, wide quadruped, heavy rhino muzzle, oversized tusks, separate bone dorsal plates; charcoal hide, ivory, turquoise eye grooves and team cloth across flank. A2 adds overlapping pale shoulder plates, A3 broad back ridge. B2 darkens neck and lengthens tusks, B3 adds an outer jaw guard. No direct one-to-one V08 panel match is asserted.

Status: implemented and desktop lineup observed. Evidence:
![Bonehorn five-form lineup](../media/p59/bonehorn/lineup.jpg)

R08 owns its quadruped layout, five meshes, diagonal gait, braced tusk shove,
shoulder hit compression, front/rear collapse, Evolution rise and reactive mantle.
Primordial Carapace uses positive HP receipts after damage; its absolute branch
parameters live in content. D-68 documents the corrected Evolution form order and
protocol 21 public reduction snapshot. Online pools remain unchanged.

Work-front checks: affected-package formatting and `git diff --check` passed.
The owner-requested preview passed (exit 0) with:
`flock -n -E 75 "/tmp/tbg-validation-$(id -u).lock" env CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR=/tmp/tbg-pr20-web-target PATH="/home/iiii/.cache/trunk/wasm-bindgen-0.2.129:/home/iiii/.cache/trunk/wasm-opt-version_131/bin:$PATH" trunk build --release --offline --locked`
in `crates/tbg-client`. The initial packaging attempt failed after Rust compilation
with `couldn't find application wasm-bindgen (version: 0.2.129), unable to download in offline mode`.
The matching existing cached binaries were then exposed through this command's
PATH; no download, dependency change, fresh target directory or cache deletion.
The successful retry reused the compiled WASM (`Finished release ... in 0.16s`).

Observed at 1280×720 in the actual local browser: all five forms, idle, diagonal
walk key pose and attack anticipation fit the lineup. A adds shoulder/back armor;
B lengthens tusks and adds the final jaw guard. Logged triangles, Base/A1/A2/B1/B2:
2,124 / 2,764 / 2,876 / 2,124 / 2,148. Screenshot is the actual rendered page,
not concept art. This is no FPS, balance or full-clip verification claim.

Deferred to the combined consolidation, not run per character:
`cargo test -p tbg-core --lib bonehorn::tests -- --test-threads=1` and
`cargo test -p tbg-core --test contract_roster --test contract_protocol -- --test-threads=1`,
inside the shared lock with two jobs. These cover HP-only stacking, first/subsequent
hits, cap refresh, expiration, shield overflow, global reduction cap, printed-range
bonus/Stronghold exclusion, A2 expiry/HP, and matching seat/public snapshots.
Pilot consolidation subsequently passed all five Bonehorn scenarios through the r0 filter.
Gameplay animation/VFX observation and live reconnect with this protocol remain
consolidation scenarios. No full suite or balance gate was run.

### R09 — Ravager

Monsters / Dps. **Ability:** Predatory Frenzy, using GAME-DESIGN 13.2–13.3.
Own library: `crates/tbg-client/src/art/kits/ravager/`.

low head, six planted limbs, massive anterior scythe limbs and lean abdomen; dark hide/bone with turquoise eyes and team dorsal sash. A2 swept lighter leg guards, A3 long trailing crest; B2 thick scythes and dark chest, B3 serrated jaw shield. No Sombrajaré image anatomy import.

Status: implemented; native scenarios passed in pilot consolidation and actual
desktop lineup observed. [Delivery and limits](p59/R09-ravager.md).
![Ravager five-form lineup](../media/p59/ravager/lineup.jpg)

### R10 — Acidmaw

Monsters / Ranged. **Ability:** Corrosive Spit, using GAME-DESIGN 13.2–13.3.
Own library: `crates/tbg-client/src/art/kits/acidmaw/`.

tall thin neck, four-part jaw, glowing sac at throat and slender legs; muted hide, bone braces, turquoise sacs, team shoulder cloth. A2 elongates throat guard and lightens neck, A3 long sight-like horn. B2 widens sac shell and darkens torso, B3 three-lobed throat collar.

Status: implemented; four native regression scenarios passed and desktop lineup
observed. [Delivery and limits](p59/R10-acidmaw.md).
![Acidmaw five-form lineup](../media/p59/acidmaw/lineup.jpg)

### R11 — Whisperer

Monsters / Control. **Ability:** Fear Pulse, using GAME-DESIGN 13.2–13.3.
Own library: `crates/tbg-client/src/art/kits/whisperer/`.

floating bell shape, one luminous eye/bone mask and short tapered tentacles; charcoal/ivory/turquoise, team ribbon underneath. A2 taller eye crest and pale bell, A3 wide mask rim; B2 dark segmented bell and paired forehead ridges, B3 prominent eye socket. Reuse primitive lathe.

Status: implemented; five timing/status scenarios passed, 34 targeted pilot tests
passed, and desktop lineup observed. [Delivery and limits](p59/R11-whisperer.md).
![Whisperer five-form lineup](../media/p59/whisperer/lineup.jpg)

### R12 — Solar Sentinel

Celestials / Tank. **Ability:** Solar Barrier, using GAME-DESIGN 13.2–13.3.
Own library: `crates/tbg-client/src/art/kits/solar_sentinel/`.

upright spear/shield guardian, oversized sun disk, ivory/gold plates, dark blue cloth and team tabard separate from gold. A2 broad pale breastplate, A3 taller disk shield crest. B2 paired shoulder disks/darker cloth, B3 wide halo with side panels. Reuse humanoid spear/shield parts.

Status: pending. Evidence: `docs/media/p59/solar_sentinel/lineup.jpg`.

### R13 — Luminous Oracle

Celestials / Control. **Ability:** Harmonic Pulse, using GAME-DESIGN 13.2–13.3.
Own library: `crates/tbg-client/src/art/kits/luminous_oracle/`.

long robe, staff sun-ring, hooded empty face and geometric cape; ivory/gold/navy, team lining. A2 wider pale stole, A3 tall staff ring. B2 short dark over-robe with side fins, B3 paired staff-ring struts.

Status: pending. Evidence: `docs/media/p59/luminous_oracle/lineup.jpg`.

### R14 — Celestial Archer

Celestials / Ranged. **Ability:** Arrow of Purification, using GAME-DESIGN 13.2–13.3.
Own library: `crates/tbg-client/src/art/kits/celestial_archer/`.

slim armored archer, angular gold bow, rigid feather wing motifs and narrow halo; ivory/navy, team quiver sash. A2 long bow/ivory sleeves, A3 large sight-halo. B2 pronged bow/dark cuirass, B3 three-point bow crown. Wings are short shoulder ornaments, not flight machinery.

Status: pending. Evidence: `docs/media/p59/celestial_archer/lineup.jpg`.

### R15 — Justiciar

Celestials / Control. **Ability:** Celestial Rain, using GAME-DESIGN 13.2–13.3.
Own library: `crates/tbg-client/src/art/kits/justiciar/`.

broad segmented construct, compact gold chest sun, floating-looking but rigid connected forearms, ivory stone/gold/navy and team shoulder panel. A2 wide bright chest plates, A3 large sun collar; B2 dark segmented arms, B3 angular side fins. Visual gaps use short hidden rigid supports.

Status: pending. Evidence: `docs/media/p59/justiciar/lineup.jpg`.

### R16 — Jawguard

Insects / Tank. **Ability:** Raised Carapace, using GAME-DESIGN 13.2–13.3.
Own library: `crates/tbg-client/src/art/kits/jawguard/`.

wide beetle shield shell, prominent mandibles and horn, stout limbs; moss/amber/black with team side bands. A2 pale layered carapace, A3 large back shield ridge. B2 darker head and longer mandibles, B3 outer amber jaw blades. Reuse Large stance with local mandible props.

Status: pending. Evidence: `docs/media/p59/jawguard/lineup.jpg`.

### R17 — Mantis

Insects / Dps. **Ability:** Precise Strike, using GAME-DESIGN 13.2–13.3.
Own library: `crates/tbg-client/src/art/kits/mantis/`.

tall narrow mantis, folded scythe forearms, triangular face and antennae; moss/amber/black, team abdomen band. A2 swept pale leg guards, A3 long antenna crest; B2 dark thick blades, B3 broad serrated elbows. Humanoid joints can drive bent insect legs without adding six limbs.

Status: pending. Evidence: `docs/media/p59/mantis/lineup.jpg`.

### R18 — Hornet

Insects / Ranged. **Ability:** Venomous Sting, using GAME-DESIGN 13.2–13.3.
Own library: `crates/tbg-client/src/art/kits/hornet/`.

slender wasp abdomen, stinger-like ranged weapon, opaque faceted wing pairs and antennae; amber/black/moss with team abdomen plate. A2 green venom sacs and pale thorax, A3 long stinger casing. B2 swept wings/ dark thorax, B3 tall fin pair. Use painted thin geometry, no transparency or wing shadow light.

Status: pending. Evidence: `docs/media/p59/hornet/lineup.jpg`.

### R19 — Weaver

Insects / Control. **Ability:** Sticky Web, using GAME-DESIGN 13.2–13.3.
Own library: `crates/tbg-client/src/art/kits/weaver/`.

large abdomen, eight clearly grouped legs, low head with green eyes, web-spinner nozzle; moss/amber/dark chitin, team back plate. A2 pale abdomen bands and wider spinner, A3 broad web-shaped back crest. B2 dark body/longer forelegs, B3 angular nozzle fork. No simulated web geometry.

Status: pending. Evidence: `docs/media/p59/weaver/lineup.jpg`.

### R20 — Pyre Warden

Undead / Tank. **Ability:** Profane Fire, using GAME-DESIGN 13.2–13.3.
Own library: `crates/tbg-client/src/art/kits/pyre_warden/`.

Follow this character's catalog sheet.

Status: pending. Evidence: `docs/media/p59/pyre_warden/lineup.jpg`.

### R21 — Drillnest

Machines / Ranged. **Ability:** Drill Nest, using GAME-DESIGN 13.2–13.3.
Own library: `crates/tbg-client/src/art/kits/drillnest/`.

low tracked hull, industrial honeycomb launcher, thick treads, one cannon/nozzle and forge vents; iron/brass/oak and team hull panels. A2 widens hatch rack and lightens side armor, A3 prominent twin feed housing. B2 thickens drill barrel/dark armor, B3 broad siege shroud. Microbroca is a small pointed mechanical tick, Duobroca two fused shells, Megabroca broad multi-bladed body; rank reads by silhouette, not scale alone.

Status: pending. Evidence: `docs/media/p59/drillnest/lineup.jpg`.

## Deferred acceptance

Full workspace suite, C7, balance matrices/gates, load/soak, additional platforms
and phone layouts are not requested. Record their exact commands when relevant.
Owner look/fun sign-off and D-33 evidence remain prerequisites for W11 activation.
Remote CI and final PR consolidation belong to the owner; do not wait for or
reproduce their complete workload locally.


## Shared art plumbing — D-67

Implemented optional per-kit anatomy over the existing node slots, preserving
existing kits with anatomy=None. The named gallery and figure cache accept only
registered staged kinds with real kits. No Shop/lab pool was changed.

Work-front check: `flock -n -E 75 /tmp/tbg-validation-$(id -u).lock env
CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR=/tmp/tbg-pr20-target cargo check -p tbg-client
--bin tbg-client --target wasm32-unknown-unknown`: exit 0, 3.49 s. Package format
and `git diff --check` pass. Existing kit regression cases are deferred to the
single consolidation run; no tests, release build or full suite ran here.
