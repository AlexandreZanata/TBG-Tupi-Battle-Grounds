# Missing art assets — implementation and verification

Status: **art implementation complete; prior acceptance blockers resolved**. Branch:
`art/missing-assets`. Base: `544579c` (`origin/phase/p17-strongholds`),
selected because Stronghold PR #10 was still open when work started and
remains open at the final check. Merging this art PR into main also brings the Stronghold D-46 changes from PR #10.
The owner explicitly authorized that combined merge and the front deployment on 2026-10-30.

The phase integrates 69 newly generated assets: nine Arcana icons, four
Augment frames and 40 illustrations, plus four Stronghold portraits,
four banners and eight branch icons. All 81 entries including the 12
approved Arcana sources use WebP fallbacks after measured vector quality
checks. No game rules, official Augment texts, costs, simulation values,
Cargo dependencies or existing contract tests changed.

- [Arcana provenance and quality](art/ARCANA.md): 21 assets, **2,751,068 B / 3,000,000 B**.
- [Augment provenance and quality](art/AUGMENTS.md): 44 assets, **3,829,406 B / 4,000,000 B — PASS**.
- [Stronghold provenance and quality](art/STRONGHOLDS.md): 16 new assets plus the compact HUD sprite, **1,733,333 B / 2,000,000 B**.
- [Actual command output and browser evidence](art/VERIFICATION.md).

Totals count gzip SVG plus WebP. Original generated PNGs, normalized
masters, full-resolution traces, comparison renders and the two rejected
Stronghold portraits with reasons are retained under ignored
`.local/art-masters/<set>/`; they are not committed. Exact selected and
rejected prompts and all measured full/display SSIM, RGB/alpha MAE,
SVG and fallback bytes are recorded in the per-set documents.

## Integration

Each of the nine Arcana child/supreme nodes owns its art. Existing
approved Arcana WebPs remain byte-identical; unused vectors were retuned
to fit the same total budget. Stronghold art uses no source URL until
the selector opens, and every existing stat/branch data field is
unchanged. The actual 3D Stronghold gallery was inspected before drawing.

The Augment dialog shows four vertical rarity cards, art, gem, rarity
band, canonical name/description, Energy cost, projected clock/progress,
Confirm and Forgo. The loader fetches its catalog only when a projected
round opens and preloads only the four current illustrations and their
rarity frames. Selection and confirmation still emit the existing
PlayerIntentions. They do not change Energy or round resolution before
the authoritative view. A Rust test checks all 40 IDs, names, rarities,
descriptions and round timing against core.

The three catalog URLs carry the art release revision so cached pre-art
JSON cannot retain old picture references or the old Stronghold schema.

The fallback encoding keeps Q90 and alpha. Augment illustration display
comparison is 200 px, frames 300 px; small icons and tower portraits use
lossless WebP. Detailed VTracer approximations that miss the unchanged
quality thresholds remain documented and are never selected in the UI.

## Verification

`cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`,
`cargo test --workspace` (**267 passed, 0 failed, 1 ignored**),
`cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown`
and `trunk build --release` passed again after synchronizing with main
(`Already up to date.`). The required three existing Node
suites passed (**13 tests**). Two new lazy-loader/rarity tests also passed.
The complete Node command including the unchanged strict asset budget contract
now passes (**16 Node tests**). The final illustrations are 400 px WebPs displayed at no more
than the measured 200 px width; quality stays Q90 and the dialog is 1120 px wide.
Full actual output is in the linked verification transcript.

Real lobby checks at **1920×1080** and **1366×768** showed all nine new
Arcana icons and all Stronghold art, with no generated pictures loading
before the relevant selector opens. Separately labeled component
fixtures checked the four Augment rarities and all 40 official texts at
both sizes. They verified affordability, pick/confirm/Forgo behavior,
no optimistic Energy deduction, clock/progress projection, and no
browser or art request errors. A title-spacing issue for six long names
at 1920×1080 was corrected and both fixture checks then passed.
No phone layout work was done.

## Resolved acceptance blockers

The owner instructed completion before merge/deploy on 2026-10-30.
The budget now passes at 3,829,406 B. The real in-app browser reached
Practice clock `1:00` and displayed all four generated cards, with every
illustration and frame loaded. Both desktop sizes were captured after
image loading and viewport paint completed. The actual round offered
Second Wind, Desperate Assault, Hardy Sap and Legacy. Normal Shop clicks
defended the Stronghold early; Energy was saved before the round. No
fixture, simulation speed change or direct state edit was used.

The following notes preserve the initial blocked attempts as history:

> blocked: [resolved 2026-10-30] Augment transport budget. Tried the initial full card encoding
> (5,835,510 B), then Q90 frames with alpha and 210 px illustration display
> comparison (4,067,598 B). Both conversion runs returned the exact error
> `Art set exceeds its total budget` against the 4,000,000 B ceiling.
> Cause: high-detail frame/illustration raster bytes plus retained SVGs.
> The step initially stopped under AGENTS.md's two-error guard; the owner
> later instructed completion, and a smaller measured display width resolved it. The 67,598 B excess is not accepted
> by changing the budget or weakening tests.

> blocked: [resolved 2026-10-30] live Practice Augment observation in the software-rendered browser.
> Two runs returned `page.waitForFunction: Timeout 240000ms exceeded.`
> The second used reduced internal raster resolution while waiting and
> restored it before capture; the last actual view was still at `0:07`,
> `council: closed`. Suspected cause: the headless renderer advances the
> fixed-step local simulation far slower than wall time. Per the same
> AGENTS.md guard, the step initially stopped. The owner later instructed
> completion; the real in-app browser resolved the observation requirement. Explicitly labeled fixtures independently verify component
> layout and controls; they do not satisfy the real Practice requirement.

## Desktop screenshots

The Augment images visibly identify themselves as fixtures. The gallery
and selector images are from the real running client. No master or
raster/vector comparison images are included in git.

- [arcanas-1366x768.jpg](media/art/arcanas-1366x768.jpg) — 177,689 B.
- [arcanas-1920x1080.jpg](media/art/arcanas-1920x1080.jpg) — 277,394 B.
- [augments-layout-1366x768.jpg](media/art/augments-layout-1366x768.jpg) — 159,484 B.
- [augments-layout-1920x1080.jpg](media/art/augments-layout-1920x1080.jpg) — 158,861 B.
- [augments-practice-1366x768.jpg](media/art/augments-practice-1366x768.jpg) — 168,198 B.
- [augments-practice-1920x1080.jpg](media/art/augments-practice-1920x1080.jpg) — 224,901 B.
- [stronghold-gallery-1920x1080.jpg](media/art/stronghold-gallery-1920x1080.jpg) — 150,939 B.
- [strongholds-1366x768.jpg](media/art/strongholds-1366x768.jpg) — 205,967 B.
- [strongholds-1920x1080.jpg](media/art/strongholds-1920x1080.jpg) — 279,847 B.

The task server was bound only to `127.0.0.1:18795` with an isolated local
database. Only the recorded server process owned by this worktree is
stopped at handoff. Main and other agents' worktrees are unchanged.
