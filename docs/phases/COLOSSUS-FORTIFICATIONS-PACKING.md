# TW-01 / PK-01 / GD-01 — implementation and validation

Owner request, 2026-11-02: triple every Stronghold's damage, add 50% HP,
make Fortification more expensive and powerful, implement width-limited
side-by-side capacity, create Gods with the 18-Energy Colossus, and send the
changes to GitHub. Linux is the visual reference. Continue the existing
`codex/ogre-faction` draft PR; do not merge or deploy publicly.

All new values and the activation boundary are Proposed in
[GAME-DESIGN](../GAME-DESIGN.md#tw-01--pk-01--gd-01--proposed-owner-revision-2026-11-02).
See [design and art brief](../colossus-fortifications/README.md) and the
[visual contract](../ui/DESKTOP-UI-CONTRACT.md#tw-01--pk-01--gd-01--gods-and-authoritative-passage-2026-11-02).
Original checkout edits were not used as an implementation workspace.

## Delivered source

- Exact base Stronghold damage x3 and HP x1.5 with integer truncation;
  Fortification 6/10 Energy. Treasury stronger spoils, Bastion armor before
  shields, capped Artillery collateral with Fury attribution, stronger/wider
  Command aura. All picker/upgrade numbers come from core-pinned JSON.
- Gods / Deuses, currently one Tank: Colossus / Colosso, 18 Energy, 2400 HP,
  170 damage, 110-tick interval, Ogre Archer reach in every corresponding
  form, frontal area hit. Five distinct forms, one actual back-carried obelisk,
  sculpted hands, authored draw/slam, heavy walk, stone impacts, death and salute.
- Private roster 82 / 10 Factions (nine with nine; Gods with one), no Gods
  Synergy benefit. Online FULL_POOL remains 46. Ring-only identity is retained.
- PK-01 candidate: 50 integer columns across five usable tiles, real body/arm/
  weapon profiles for 87 kinds and 435 forms, gradual bypass and persistent
  slots, paid-unit/summon waiting queue, private owner count, no-charge LaneFull
  Evolution rejection. Births may use the nearest free footprint in the own
  backfield third, never beyond a hostile front. No active unit teleports to
  bypass a blocker. Casks use launch origin/target and exact burst lane.
- Private Practice/Training and explicit preview lab experiments enable physical lanes.
  Canonical competitive matches keep legacy movement/crowd presentation until
  long-run acceptance. Both views carry the authoritative flag. Preview games
  interpolate lateral roots without a second cosmetic repulsion solver.
- Wire 36, ruleset 11, `w36-r11-a0a1d508c59e5841`. Historical enum indices and
  fixture rows remain intact. Stored four/seven/eight/nine-Faction summaries
  stay readable. Public landing catalog derives 82 entries and ten filters.

## Targeted consolidation evidence

Every compiling command used nonblocking `/tmp/tbg-validation-1000.lock`,
`CARGO_BUILD_JOBS=2` and the existing root target cache. No full workspace
suite, release build, full D-33 gate or extra-platform build was requested.
Commands below use `--offline --locked`; test execution uses `--test-threads=1`.

- `cargo check -p tbg-core --lib --features lab,training`: passed on the work
  front. Formatting and `git diff --check` passed on the integrated source.
- `cargo test -p tbg-core --features lab,training --lib -- pk01_ gd01_`:
  final 12 passed. Includes mirrored giant blocking, ranged row capacity,
  stable recomputation, mixed-form conservation, queue release/replay/privacy,
  gradual bypass, failed Evolution without spending, protected backfield birth,
  no ranged reach extension, bot preference and real frontal Colossus damage.
- Seven rolling-cask `og02_` cases passed with the new presentation facts;
  direct launch/replay/interception/heal behavior is unchanged by PK activation.
- `cargo test -p tbg-core --features lab,training --test fortification_power
  --test ogre_roster --test abyssal_roster`: five passed. Two direct `tw01_`
  shot/armor cases also passed. Those unchanged results were reused.
- `cargo test -p tbg-core --features lab,training --test contract_protocol
  --test contract_public_view --test contract_ruleset_version
  --test contract_determinism --test contract_lab`: 31 passed. After the private
  Colossus tuning, the seven ruleset/determinism/lab identity cases passed again
  for the final source hash. Twenty-four wire/privacy cases retain their valid evidence.
- Five canonical bot traces finish by Stronghold destruction: seeds 1/7/42/
  1000/65535 end at 16625/14265/23274/58307/24291 ticks. The raw
  [pre-tuning trace](../colossus-fortifications/traces-canon-final.txt) has source
  `2eea56326efbb607`; final identity tests confirm identical state/trace checksums
  after tuning an unused private kind, under source `a0a1d508c59e5841`.
- `cargo test -p tbg-client --bin tbg-client -- gd01_ stronghold_data_tests
  ring_only_ support_ sync::timeline::tests`: 18 passed. Includes finite
  Colossus life/action poses, distinct forms, geometry budget, every kind/form's
  ring-only body materials, all Stronghold facts, support posture and timelines.
- `cargo check -p tbg-server --lib`: passed, 3.01 s (cached native target).
- Cached native `cargo build -p tbg-client --bin tbg-client`: passed; exact
  committed build/capture/install receipt is appended after visual observation.
- `python3 web/scripts/check.py`: passed, generated 82-entry catalog current,
  156 landing files checked. No published-site change is claimed.

## Bounded balance measurements

[80 defensive cases](../colossus-fortifications/balance-final.csv): four tower
kinds x five forms x two seats x six Archer or six Longaxe attackers, at most
1000 ticks each. Physical lanes are false in this TW-01 canonical screen.
All 80 eliminate their six attackers, final tower HP 3377..10736, and all 40
seat pairs match exactly. This is a bounded defensive screen, not a full gate.

Initial Colossus 3600 HP / 70 ticks won all eight Energy-18 base references and
all eight Energy-24 evolved references. At 2400 HP / 70 ticks the evolved
panel still all loses. Final proposal 2400 HP / 110 ticks keeps owner-fixed
cost/damage/reach and yields:

- [Eight Energy-18 duels](../colossus-fortifications/duels-final.csv): beats
  Archer and Longaxe; loses to Ogre Archer and Trident, both seats.
- [Eight Energy-24 base counters](../colossus-fortifications/counters-base-final.csv):
  loses to Sawbot, Farwatch and Mantis; beats Ravager, both seats.
- [Eight evolved references](../colossus-fortifications/counters-2400-slow.csv):
  still wins all four pairings on both seats. Evolution cost is included.

Commands: `cargo run -p tbg-balance --example colossus_towers_screen`, final
private-kind-only repeat with `-- --duels-only`, and
`cargo run -p tbg-balance --example colossus_counters [-- --base]`.
No counter, average win-rate or competitive-acceptance claim extends beyond
these exact forms, budgets and compositions. The final 104 scenarios reuse
unchanged tower evidence and retest only the tuned private champion.

## Known issues and pending acceptance

> blocked: repeated preview bot probes reached 120 s wall time with both
> Strongholds intact and persistent front/queue formations. Initial three-tile
> geometry, later five-tile/root-range/birth/bot candidates were inspected;
> full preview bot end-to-end balance is unresolved. Stop repeating this probe.
> PK-01 stays private rather than entering competitive canonical play.

Extreme forced displacement can retain an active unit targetable when no legal
projection exists; zero overlap for every teleport/congestion case is not
claimed. Existing combat/area rules remain longitudinal. Five-tile floor and
meadow dressing are shared; themed scenery clearance still needs observation.

Native Linux actual captures, a short packing witness and the installation
receipt are pending below. Windows/desktop game-web comparison, both-seat
continuous crowd motion, dense mixed-team fun review and full D-33 acceptance
remain pending. No shared-code or native-only parity claim is permitted.

Actual landing browser inspection remains blocked: fresh browser inventory
worked, but selecting localhost returned `browser.user is unavailable`.
Static/link/HTTP checks do not substitute for browser appearance. Old preview
sessions and installations remain available; the new slot is separate.
