# P16-A — Desktop lobby and battle arena (owner references)

**Depends on:** P15 signed off, the Archer v2 candidate approved by the owner.
**Runs before:** P16 (P16 now depends on P16-A reviewed).
**Branch:** `phase/p16a-desktop-lobby-arena`.
**Difficulty:** high (art quality); the wiring is medium.

## Goal

On **desktop only** (landscape, ≥ 1280×720), the lobby and the battle arena
look like the owner's two reference images, built with the current code
pipeline (D-22, approach A: vertex-colored meshes made in code, no
Blender). "Like the reference" means the same layout, composition, palette,
and density of detail; the references are the **minimum** quality bar.
Phone and tablet layouts stay as they are and must not regress; they get
their redesign in P17 (arena, every ratio) and P20 (lobby, every ratio).

## References (local only — never commit them)

- `.local/references/p16a/lobby-desktop-reference.webp` (1672×941)
- `.local/references/p16a/arena-desktop-reference.webp` (1672×941)

`.local/` is gitignored. Comparison images that contain the references stay
in `.local/references/p16a/compare/`; only our own screenshots go to
`docs/media/p16a/`.

## Deliverable

- Archer v2 (base, Longbow, Marksman) is the default Archer (step 1).
- A rebuilt desktop arena matching the arena reference (steps 2–5).
- A desktop lobby matching the lobby reference, over the live arena (steps 6–8).
- The asset and triangle budgets of ARCHITECTURE §10 as amended (D-26).

## Read first (only these)

- `AGENTS.md`
- `docs/ART-DIRECTION.md` §1–§3, §7, §9
- `docs/DECISIONS.md` D-20, D-22, D-26, D-27
- `docs/phases/P17-arena-and-background.md` and
  `docs/phases/P20-lobby-and-ranking-page.md` (what later phases extend)
- Client: `src/arena.rs`, `src/arena/props.rs`, `src/camera.rs`,
  `src/ui/menu.rs`, `src/ui/mod.rs`, `src/ui/textform.rs` (HTML overlay
  bridge), `src/ui/ranking.rs`, `index.html`
- Archer v2: `src/art/archer_v2*.rs`, `src/art/roster/humans/archer_v2*.rs`
- Bevy examples: `3d/vertex_colors.rs`, `3d/lighting.rs`,
  `3d/generate_custom_mesh.rs`, `3d/fog.rs`

## Do NOT

- Change anything in `tbg-core`, the protocol, the server, or the lane
  geometry (x = 0..16, 3 wide) and Stronghold positions.
- Add textures for the 3D scene or post-processing passes (colors stay
  vertex colors; ambient occlusion is baked into vertex colors).
- Build features the lobby mock shows but the game does not have (see
  "Owner decisions"); show them as specified there.
- Put props inside the lane or between the camera and the lane.
- Commit the reference images or anything derived from them.

## Steps

### 1. Archer v2 becomes the default
Skip if the reviewer already did it. Remove the `?archer=v2` flag: the v2
spec, proportions, bow lengths, and clips serve every Archer form on branch
A; delete the P15 Archer form, clips, and code paths they replace. Keep the
`&lineup` gallery sheet. Budget test and reach test stay green.
- **Done when** `?gallery=archer&lineup` and a practice match show v2.

### 2. Island, cliffs, and water
Split `arena.rs` into an `arena/` folder: `ground.rs` (grass top with two
greens, irregular polygon outline, sand lane tiles with grout lines and a
darker lane border), `cliffs.rs` (gray stone cliff walls with faceted
strata, darker toward the base, visible on the front edge), `water.rs`
(turquoise sea plane to the frame edges, a foam band hugging the cliff
base, 8–12 rocks in the water with foam rings). Placement from a fixed
seed; merged into few meshes.

### 3. Props
`props.rs` split as needed (`trees.rs`, `rocks.rs`, `camp.rs`):
- Back edge: a dense ring of low-poly pines (2–3 stacked cones, two
  greens), large gray boulders, bushes.
- Lane sides: two fence runs per side near the center with team banners
  (blue on the own half, red on the enemy half), stone-based braziers at
  the lane ends and fence ends.
- Strongholds: crates, barrels, and a brazier beside each tower; hanging
  team banners and a flag on the tower (keep the three damage states).
- Front edge (camera side, **low props only**): boulders, round bushes, a
  fallen log, a barrel, white flowers, grass tufts, small pines.

### 4. Light and life
Warm key light from the upper left (ART-DIRECTION §2), soft shadows, baked
AO at prop bases. Animated cheaply: brazier flames (scale flicker + emissive
color), banners (2–3 rigid segments, sine sway), foam (slow slide). Total
animated entities ≤ 60; zero allocation per frame.

### 5. Desktop camera
For landscape aspect ≥ 1.5, frame the whole island like the reference:
lane horizontal through the center, own Stronghold left, enemy right, sea
visible on all sides. Other aspects keep the current framing.
- **Done when** a 1920×1080 capture of an empty arena and of a busy match
  (30 units) matches the reference composition and every unit stays
  readable (team ring, HP bar, name visible over the new ground).

### 6. Lobby shell (desktop)
Per D-27, the desktop lobby is an HTML/CSS layer over the canvas (no
framework, no CDN, fonts self-hosted under `assets/fonts/` with an OFL
license file). Behind it the live arena with a slow ±8° orbit; panels use
`backdrop-filter: blur()` and a vignette, as in the reference. Layout:
- **Top bar:** logo, player chip (shield icon, name, rating), tabs
  Lobby · Arcanas · Troops · Ranking · Settings, sound toggle on the right.
- **Left panel:** four mode buttons with icon, title, subtitle: Play
  Online (Ranked, highlighted blue), Casual (Quick Match), Play with a
  Friend (create or join room), Practice vs Bot.
- **Bottom left:** player card: emblem, rating, progress bar, Wins /
  Losses / Win rate (P13 stats API).
- **Bottom center:** three feature cards: Arcanas, Troops, Ranking.
- **Bottom right:** News carousel (static list in the page, ≤ 5 items).
Guests see `Guest` with Create account / I have a key in the player chip;
Play Online shows the lock and the call to action, as today.

### 7. Wire the actions
Every button calls an existing `UiAction` through the overlay bridge
(D-20); no new protocol messages. Below 1280×720 or in portrait the HTML
lobby hides and the current Bevy menu shows. Keyboard: Tab order, Enter
activates, Esc closes sub-screens.

### 8. Lobby art
Logo, icons, emblem, and card illustrations as SVG or WebP per the owner's
choice (decision 3). Every file ≤ 200 KB; lobby art total ≤ 2 MB; loaded
after the menu is interactive (lazy), never blocking time-to-menu.

### 9. Measure and capture
Startup log: arena triangles per layer and total, draw calls in a busy
match. Captures at 1920×1080 and 1280×800: lobby (guest and logged in),
empty arena, busy match → `docs/media/p16a/`. Side-by-side comparisons with
the references → `.local/references/p16a/compare/` only. Phone 390×844:
one capture each of menu and match proving no regression.

## Owner decisions (defaults apply until the owner answers)

1. **UI language.** The mock is in Portuguese. Default: English labels as
   in the rest of the game; Portuguese is a later localization pass.
2. **Mocked features that do not exist.** Arcanas (P32+, after P31):
   card and tab shown locked, "Coming later". Troops (unit codex): card and
   tab shown locked. Rank tiers ("Bronze I, next level 1200") have no rule
   in GAME-DESIGN: show the global rank `#N` and rating instead, progress
   bar hidden. Settings: sound, log out, account key only.
3. **2D art source.** Default: code-drawn SVG icons and CSS panels; card
   illustrations as owner-supplied WebP (images the owner holds rights to)
   dropped in later without code changes.
4. **Overlap.** P17 and P20 keep their scope for every other ratio and
   reuse this work; their specs are updated after this phase's review.

## Acceptance checklist

- [x] All verification commands from `AGENTS.md` pass; `trunk build --release` passes
- [x] Desktop captures in `docs/media/p16a/`; comparisons kept in `.local/` only
- [x] Arena ≤ 60 000 triangles (D-26); draw calls recorded, with no fixed maximum (D-28)
- [ ] 60 fps on the desktop dev machine with 30 units; phone not regressed
- [x] WASM brotli ≤ 6 MB; first-load assets and lobby art within D-26
- [ ] Every lobby action works (guest and account), including a ranked queue
- [ ] Owner approves lobby and arena against the references (report)

## Risks and what to do

| Problem | Action |
|---------|--------|
| Dense trees hurt FPS or readability | Merge per layer; desaturate the back ring 10–15 % |
| HTML layer steals canvas input | `pointer-events: none` on the layer, `auto` on panels only |
| Code-built look stays below the reference | Record it; D-22's Blender fallback is the owner's call |

## Phase report

Implementation completed locally on 2026-10-28, branch
`phase/p16a-desktop-lobby-arena`. Review and owner look approval remain open.
The owner's subsequent desktop battle HUD request is separate follow-up work.

### Implemented

Archer v2 is the default for base, Longbow, and Marksman; the replaced P15
native form, clips, and opt-in switch were removed. Existing reach, release,
and budget tests remain green (1,252 / 1,348 / 1,480 triangles including bow).
The lineup and a practice match were inspected in the browser.

The island has an irregular two-tone meadow, sand tiles and grout, faceted
stone strata, turquoise sea, coastal foam, and ten offshore rocks. Seeded
pines, boulders, bushes, flowers, grass, fences, camp supplies, eight braziers,
and team banners dress the edges. The Strongholds retain their positions
and three HP-driven damage states. There are 27 ambient animated entities;
flame/cloth/foam updates allocate no collections per frame. Full character
shadows are retained. Static layers are merged and flag meshes shared without
removing detail. Desktop unit nameplates avoid overlapping one another.

The HTML desktop lobby includes the reference layout, locked future features,
authenticated/guest states, live P13 stats, private-room create/join, settings,
and three static news items. All game actions use the existing UiAction queue.
Fonts are self-hosted OFL subsets (35,248 bytes); original SVG illustrations
load after the menu is interactive and can be replaced using `art.json`.
The existing Bevy menu and framing remain below the desktop breakpoint.

### Measurements

| Arena layer | Triangles |
|---|---:|
| Meadow | 192 |
| BattleLine | 444 |
| Cliffs | 384 |
| Sea | 12 |
| Coastal foam | 384 |
| Offshore rocks and foam rings | 2,000 |
| Forest, near props, fences and camp | 31,828 |
| Two Strongholds, all damage-state pieces | 1,344 |
| Animated fire and cloth | 624 |
| **Total** | **37,212** |

Mixed 30-unit stress at 1280x800: **60.0 rendered FPS**, **320.8 actual WebGL
calls/frame, maximum 322**, including shadows. Counts are diagnostic under
owner-approved D-28; they are no longer an acceptance ceiling. The 1920x1080
stress capture also recorded 60.0 FPS. A 390x844 practice viewport showed
60.0 FPS with seven live units. This is desktop hardware at phone dimensions,
not a physical-phone result. Exact samples are in
`docs/media/p16a/browser-metrics.json`; layer logs in `arena-startup.json`.

WASM is **5,556,563 bytes brotli at quality 9**, below the 6 MB hard ceiling;
the aspirational 4 MB target is not met. All asset files together are
209,308 uncompressed bytes; the conservative other-first-load upper bound,
including JS and HTML, is 342,023 bytes. Lobby SVGs total 21,756 bytes; every
individual art file is below 200 KB. See `build-metrics.json`.
Total browser/GPU memory and cold 4G time-to-menu were not established.

### Verification and behavior

- `cargo fmt --all -- --check`: exit 0.
- `cargo clippy --workspace -- -D warnings`: exit 0.
- `cargo test --workspace`: **95 passed, 0 failed**.
- `cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown`: exit 0.
- `trunk build --release`: exit 0, success at `2026-10-29T00:22:10.596238Z`
  (2026-10-28 in America/Cuiaba).
- `node --check crates/tbg-client/assets/lobby/lobby.js`: exit 0.
- `git diff --check`: exit 0.

An isolated server on 127.0.0.1:18080 used a temporary SQLite database.
Both browser clients entered Ranked through the new lobby; reloading one
client resumed the active match. Surrender finished the match, both clients
showed their results, and the lobby refreshed the rating, rank and W/L stats.
Private create/join (including lowercase-code normalization), Casual queue
and Cancel, Practice, guest registration, key-save, auto-login, two-step
logout, sound, news, Ranking, Enter, and Esc were observed. No server, core,
or protocol files changed.

The key Download button dispatched its existing action, but the in-app
browser's download observation returned `Timed out after 5000ms waiting for
download.` No console error was reported. A manual download check remains
pending; no account key screenshot or database is committed.

All required desktop/phone captures and the default Archer lineup are in
`docs/media/p16a/`; comparisons containing the owner's images are only in
`.local/references/p16a/compare/`. Full workspace test output is saved in
`docs/media/p16a/workspace-tests.txt`.

### Owner decisions and deviations

- English labels; Arcanas/Troops locked; global rank and rating, no invented
  rank tiers or progress; original SVG/CSS art. Defaults 1–4 were applied.
- D-28 records the owner's explicit removal of the fixed draw-call ceiling.
  The temporary small-limb shadow shortcut was removed as part of that request.
- The branch starts from the current P15 checkout (`ae01420`), since main did
  not contain the P15/Archer candidate required by this phase. Nothing was
  pushed or merged, and earlier owner checkpoints were not marked approved.
- `?arena` is a presentation-only inspection view without a menu/orbit;
  `&stress=30` reuses the existing renderer stress scene. It changes no
  SlotCount or simulation rule.
- Initial socket-restricted tests failed with `Operation not permitted`;
  rerunning with localhost sockets enabled passed. A Bevy text-layout API
  mismatch (`no associated function or constant named new_with_justify`)
  was corrected using the installed 0.20 source before the passing checks.
  Early builds interrupted during editing were replaced by a stable release build.

### Remaining human/reviewer checks

- Owner approval against the references; the SVG card illustrations are
  flatter than the reference artwork, and the owner may replace them through
  the manifest or decide on D-22's fallback. No subjective approval is claimed.
- Physical Android/iPhone FPS, touch, total memory and cold-network startup.
- Manual key-download confirmation, independent code review, and the still-open
  P14/P15 owner checkpoints. P17/P20 spec reuse is updated after review.
