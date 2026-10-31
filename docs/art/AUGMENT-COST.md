# Augment cost badge alignment

## Result

Augment cost badges now display a white spoils-of-war chest icon and the authoritative numeric cost. Insufficient funds use the existing gray card state with the same icon/number geometry. Visible cost text contains no Energy suffix or insufficient-funds sentence. Accessible button names retain the canonical Energy cost and explain insufficient funds. Rules, prices, affordability, selection and confirmation remain unchanged.

Each frame has a slightly different lower recess: the badge center is at 90.2% of card height for Bronze, 91.4% for Silver and 90.7% for Gold/Diamond, horizontally centered at 50%. Both children share a single flex line with a fixed anchor. The description has a bounded region above it, preventing copy from invading the badge.

## Original icon and conversion

Generated with the built-in image tool on 2026-10-30, using transparent alpha. Original retained at `/home/iiii/.codex/generated_images/01a1876d-fc5b-7971-a521-c09d577887a3/exec-46330d99-161e-46b9-ba5c-62fb61b761ce.png` and private `.local/augment-cost/war-spoils-original.png`.

Exact prompt:

Use case: logo-brand. Asset type: a single minimal monochrome game UI icon, intended for conversion to SVG and display at 18 to 22 pixels. Primary request: represent medieval spoils of war with one small open treasure chest and two simple round coins inside. Straight front view, centered square composition with generous empty transparent margins. Flat solid pure white shapes only, crisp clean smooth edges, simple broad geometric silhouette, open lid angled back but readable from front, a small dark negative-space keyhole and separation between lid and base, two circular coin silhouettes visible above the chest rim. No more than a few broad shapes and negative-space cuts. Genuine transparent background. No gray, black ink, color, gradients, shading, texture, sparkle, glow, shadows, border, lettering, words, digits, watermark, or decorative detail. The image should read immediately as loot at very small size. Only the single original white icon on transparent alpha.

The existing Pillow/vtracer toolchain converts the alpha silhouette to a compact vector: alpha threshold 128, crop to alpha bounds, fit within 80x80, two-pixel padding, binary spline trace, speckle filter 4, corner threshold 60, length threshold 3, splice threshold 45 and path precision 2. The single white path preserves transparent negative space; the SVG contains no embedded raster. No new dependency was added. The HUD uses it as an alpha mask, inheriting the cost line color.

Output: `crates/tbg-client/assets/augments/war-spoils.svg`, 2,957 bytes raw / 1,297 gzip, viewBox `0 0 84 79`, SHA-256 `16ce6896278e51473d6ed0bc9d6cfc486fb6495a36a580c9e93947deaecc73a0`. The complete Augment set including the icon totals 3,830,703 transport bytes, below its 4,000,000-byte budget.

## Verification

- `node --test crates/tbg-client/tests/*.test.js`: 21 passed, 0 failed, including the complete art budget check.
- `node --check crates/tbg-client/assets/battle/hud.js`: pass.
- `cargo fmt --all -- --check`: pass.
- `cargo clippy --workspace -- -D warnings`: pass (1.31 s).
- `cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown`: pass (1.10 s).
- `trunk build --release`: pass (cached Rust release stage 0.51 s; WASM optimization and staging completed). Built SVG SHA-256 equals the source. WASM remains `af370582afe9815c26bee7a49f1e762800712d7a98cf0ce297751c88450cef57`, as Rust source is unchanged.
- Browser fixture: actual production HUD HTML/CSS/JS at 1920x1080 and 1366x768; all four rarity frames inspected with sufficient and insufficient funds, plus the owner-reference selection and the four longest catalog descriptions. Zero console errors. No game/player database is used by the fixture.
- Measured horizontal centering error <= 0.0078125 px and icon/number vertical center difference 0 px. Mixed-funds and all-affordable bounds are exactly identical. Long descriptions clear the cost region by at least 7.89 px at 1920x1080. [Measurements](../media/augment-cost/layout.json).

Gallery: [reference selection 1920](../media/augment-cost/reference-1920.jpg), [reference selection 1366](../media/augment-cost/reference-1366.jpg), [four white badges](../media/augment-cost/rarities-white-1920.jpg), [four gray badges](../media/augment-cost/rarities-gray-1920.jpg), [white 1366](../media/augment-cost/rarities-white-1366.jpg), [gray 1366](../media/augment-cost/rarities-gray-1366.jpg).

The existing local online persistence test remains stopped after its repeated failure; see [the ranking delivery report](../RANKING-UI.md#online-verification-blocker). This visual fix does not change Rust source. No further attempt of that test, merge or VPS deployment is included without the pending owner exception.

Observed in the final WASM build during real local Practice at 1:00: Stone Mend (6), Siege Arrows (6), Shield Step (6) and Deathless Legion (15), with the generated chest and all artwork loaded inside the frame recesses. Cost labels contain digits only; error-level browser console entries: 0. [Real Practice capture, 1920x1080](../media/augment-cost/practice-1920.jpg). This supplements the controlled white/gray comparisons above. No production account or database was accessed.
