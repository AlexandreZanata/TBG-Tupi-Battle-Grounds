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

Five canonical trace seeds were recorded; seed 65535 changes end tick/winner
from the previous baseline (requested range/Faction change). Earlier fixtures
remain intact; OG-01 has an explicit appended source identity and baseline.
All compiling commands used the nonblocking shared host lock and two jobs.
No full suite ran. Native installation and actual captures remain next.
