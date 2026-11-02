# Ogre faction and Machine replacement

Status: implementation and focused validation in progress on `codex/ogre-faction`.
Owner-authorized scope and initial numeric proposals: GAME-DESIGN OG-01 and
DECISIONS OG-01. Nine new champions, 45 new forms, existing Ogre Archer transfer
and range equality, equipment clearance, landing catalog, GitHub PR. The
seventeen previous preview champions, support-facing fix and PR44 laboratory
are retained. Original checkout/user edits are preserved.

Read: AGENTS, VALIDATION, ART-DIRECTION sections 2–5, DESKTOP-UI-CONTRACT,
existing completion hooks, SiegeWalker kit/crowd/overhead pipeline and web
content sync. No new dependency. No mobile, public deployment or merge.

Image generation used the built-in image_gen tool, one five-form sheet per
new champion. Prompt set: docs/ogre-faction/image-prompts.json. All nine full
PNG references are saved outside git in `.local/ogre-faction-2026-11-01/concepts`
under the original checkout. SHA manifests distinguish concept images from
actual native captures. Character body equipment is seat-independent, without
body affiliation plaques; blue/red foot rings supply identity.

Validation level: scoped consolidation. First cached client check found a
Node equality expression in the new Hook Brawler model; replaced it with a
variant match. Final results and commands will be appended after consolidation.
Windows/desktop-web visual motion, full competitive balance and look/fun
acceptance remain pending. No full workspace suite is authorized or run.

## Initial scoped consolidation (ruleset 8)

Core: 29 targeted tests passed (22 completion scenarios, range/roster,
Synergy, source classification, revision identity and frozen bot/trace checks).
Client: 8 targeted tests passed in 0.88 s, including 45 distinct forms below
6000 triangles, finite clip transforms, actual bazooka/arm vertex clearance,
220 support cases, shell flight alignment and ring-only bodies on all 385
registered forms. The test name still says 340; its loop uses ALL_KINDS.
The first test build revealed an omitted Ogres arm in the existing death-profile
test; the explicit case was added. Cached server check passed in 3.46 s.

D-33 bounded screen: `cargo run --offline --locked -p tbg-balance --example
ogre_screen`, 148 seat-swapped Energy-equal duels at budget 20 (five Ogre
Archer forms against five references, before/requested range, plus eight new
non-healers against three references). Raw initial results:
[balance-initial.csv](../ogre-faction/balance-initial.csv). This is not the full
release gate. Ogre Archer's mean score in this bounded panel is 0.68 before
and after; the eight new solo kits lose all three initial reference matchups.
Those weak starting values require a proposed tuning pass before visual delivery;
the isolated control/healing roles are not judged by solo-duel wins alone.

Five canonical trace seeds were recorded; seed 65535 ends at tick 19759 with seat A winning, versus the earlier
8613-tick trace (requested range/Faction change). Earlier fixtures
remain intact; OG-01 has an explicit appended source identity and baseline.
All compiling commands used the nonblocking shared host lock and two jobs.
No full suite ran. Native installation and actual captures remain next.

## Candidate 9 scoped consolidation

Protocol 34 / ruleset 9 / source hash `89698dcd4bcb5ddd`.
**59 core and 8 client targeted tests passed.** The final core selection adds
the authoritative earth-center scenario; the new test caught a Prismancer-only
center initializer before commit. The initializer now follows the typed Beam
ability; public earth telegraphs use the same locked point. Client checks
cover Ogre special anticipation and retained support/weapon/body behavior.
Server check passed in 1.12 s. The existing unrelated `Layout::clear` warning
remains. No full workspace or release checks ran.

Commands, under `/tmp/tbg-validation-1000.lock`, `CARGO_BUILD_JOBS=2`, existing
root target cache, offline/locked:

```bash
cargo test --offline --locked -p tbg-core --lib --test ogre_roster --test contract_constants --test contract_ruleset_version --test contract_determinism --test contract_lab -- ogre_ completion_ synergies every_core_source current_hash bot_matches c2_ --test-threads=1
cargo test --offline --locked -p tbg-client --bin tbg-client -- ogre_ support_ the_bombard_points ownership --test-threads=1
cargo check --offline --locked -p tbg-server --bin tbg-server
cargo run --offline --locked -p tbg-balance --example ogre_screen
cargo test --offline --locked -p tbg-core --test contract_protocol --test contract_protocol_server --test contract_roster --test contract_release_identity -- --test-threads=1
```

The last four named contract targets add 29 checks; they pin the appended
Faction/kind discriminants, stable message tags/bytes, exact range/roster/forms
and computed release identity. Earlier fixture tables and canonical traces
are preserved with explicit OG-01 amendments. One approval review expired
before starting the contract command; the single retry completed normally.

Final D-33 bounded screen: [balance-final.csv](../ogre-faction/balance-final.csv),
148 seat-order duels. Basalt wins Shieldbearer/Longaxe and loses Archer;
Cinder wins Shieldbearer; Pebble Slinger wins Shieldbearer/Longaxe; Rivet wins
all three references. Hook, Flint and the two control specialists lose these
three solo matchups. This small panel does **not** establish competitive
balance or counters for every form; team compositions, special-ability
opponents, other Energy budgets and the complete gate remain pending.
Hide Mender is excluded from solo-duel screening. Values remain Proposed and
new kinds stay private-preview only; no threshold was weakened or gate claimed.

Local review: `http://localhost:8964/`. Its nine concept sheets are available;
actual engine portraits and five-form lineups are being added next. Native
figures share one vertex-color rig pipeline across desktop targets. Linux
native observation is next; Windows/game-web visual motion remains pending.


### Native silhouette refinement

The first source-stamped native portraits revealed thin human-like arms and
insufficiently visible tusks on the new Ogres. The shared Ogre-only anatomy
now has broad beveled forearms/fists, a larger jaw/nose, sloping brows and
forward lower tusks. Basalt uses intrinsic Ogre skin and irregular stacked
stone faces; Flint uses aged pale skin. The Machine replacement and existing
Ogre Archer keep their own anatomy. This is presentation-only. Initial images
retain their provenance; affected portraits/lineups are replaced from a new
committed renderer, with unchanged kinds reused only through verified manifests.

## Refined native assets and landing

The Ogre anatomy refinement passed the three affected geometry/clearance/
ring-only tests in 0.99 s. Native renderer build passed in 14.79 s from
`03b9b2de158c2a6d5e75873c814e501beec7fbd8` (same shared lock, two jobs).
All 16 affected Ogre portrait/lineup captures passed readiness and content
hash checks. The four unchanged Ogre Archer/Rivet captures are reused with
their original `abe7893` provenance. Final review manifest has 20 captures;
all ten portrait WebPs are pixel-identical to their PNGs and below 40 KiB.
Ten committed five-form sheets use WebP quality 90 and are below 400 KiB each; provenance
is in docs/ogre-faction/native-previews.json. The full 1600×1000 originals and
logs remain outside git. No loading screen is accepted as a capture.

`python3 web/scripts/check.py` passed: landing 146 files / 3815 KiB,
admin-public 16 / 240 KiB, admin-preview 141 / 2088 KiB; generated content
freshness passed. The real desktop browser showed 72 cards, eight 9-count
filters and 26 preview badges; the Ogre filter and bilingual names/counts
were observed. Its first image observation found the dev server's startup
manifest had been created before the newly tracked portrait files; only that
owned local server was restarted. Final image loading and install witness
are recorded below. This static landing check does not verify game-web parity.
