# Faction Shop frames and future Synergy emblems

## Result

The four canonical Factions each have a separate colored Shop mold and a distinctive generated emblem. The existing ten 3D troop portrait files are byte-for-byte unchanged. Only the portrait becomes grayscale when unavailable; the frame keeps its Faction colors. Prices, authoritative availability, five offers, purchase intentions, inspect actions and competitive rules are unchanged. The emblems are saved separately for future use and are not loaded or added to the match HUD.

## Canon and palettes

Classification follows `Faction` / `FACTIONS` in `crates/tbg-core/src/types.rs`, the Synergy definitions in `content.rs`, GAME-DESIGN section 5 and ART-DIRECTION section 3. Archetypes are roles, not extra Factions. Bonuses below describe existing rules, not new values.

- Humans — The Crown: cobalt blue `#2F6DC3`, antique gold `#D9B44A`. Crowned kite shield and sunburst. Max HP +20% / +40% at 2 / 4 living conventional units. Shieldbearer and Archer.
- Undead — The Barrow: amethyst `#76519D`, ivory bone `#D8D2C0`, restrained soul-green. Crown-ridged skull with swept soul flames. Attack speed +20% / +40%. Bone Brute and Hexer.
- Machines — The Ironworks: forge orange `#D47A36`, charcoal iron `#4A5058`, brass. Six-tooth forge cog, anvil and hammers. Damage +5% / +20%. Sawbot, Siege Walker and Rampart.
- Plants — The Wildwood: forest green `#3F8C50`, oak `#6B4F2E`, small rose/amber accents. Emerald seed, three leaves and root heart. Regenerate 0.75% / 2% max HP per second. Bloomkeeper, Rootcaller and Nester. Beetling belongs to Plants but summons do not count toward Synergy (BR-U08).

## Production and normalization

Built-in image generation tool, 2026-10-30; no external image API or new dependency. [Exact prompt set](prompts/factions.json). Private originals and normalized masters remain in `.local/art-masters/factions/`; no large PNG master is committed. `scripts/faction_art.py` reproduces the existing vtracer/SSIM pipeline. Frames are contained without stretching/cropping within 720x1024 masters, emblems within 1024x1024, with a consistent 2% transparent margin.

The real SVG derivatives preserve vector paths, but tracing painterly texture/alpha fails the full-size SSIM >= .95 and display SSIM >= .97 / alpha MAE <= 1 gates. The manifest explicitly selects the approved WebP fallback: Q90 method 6 at twice the 260-pixel frame inspection size; lossless at four times the 72-pixel emblem inspection size. The secondary SVGs are retained but never silently substituted for the higher-fidelity raster. All eight served assets pass display gates.

Versions: Pillow 12.2.0, vtracer 0.6.15, numpy 2.4.6, scipy 1.18.0. Full trace, reduced vector and served metrics and SHA-256 values are in [quality.json](../media/factions/quality.json).

- `frame-humans`: 98,528 WebP bytes; SSIM 0.976823, alpha MAE 0.062017; SVG gzip 31,060 bytes.
- `emblem-humans`: 91,418 WebP bytes; SSIM 0.988093, alpha MAE 0.132909; SVG gzip 32,780 bytes.
- `frame-undead`: 97,244 WebP bytes; SSIM 0.976060, alpha MAE 0.081289; SVG gzip 30,016 bytes.
- `emblem-undead`: 94,652 WebP bytes; SSIM 0.993330, alpha MAE 0.182099; SVG gzip 37,514 bytes.
- `frame-machines`: 94,788 WebP bytes; SSIM 0.983453, alpha MAE 0.062349; SVG gzip 32,271 bytes.
- `emblem-machines`: 96,602 WebP bytes; SSIM 0.995101, alpha MAE 0.110147; SVG gzip 40,025 bytes.
- `frame-plants`: 125,286 WebP bytes; SSIM 0.978036, alpha MAE 0.089200; SVG gzip 28,779 bytes.
- `emblem-plants`: 79,576 WebP bytes; SSIM 0.992588, alpha MAE 0.107639; SVG gzip 30,627 bytes.

Total SVG gzip + WebP: **1,041,166 bytes**, budget **2,000,000 bytes**. Only four frame WebPs preload when the Shop first displays; no emblem or full-resolution SVG preload.

## Integration and visual verification

`factions/art.js` validates a four-Faction manifest, safe local paths, color and normalized visual coordinates. Per-frame cost anchors follow the actual painted recess: Humans 85.6%, Undead 84.3%, Machines 82.9%, Plants 85.1%. Names/roles remain live text. The top seal is deliberately blank, reserving future emblem integration.

- JavaScript: `node --test --test-isolation=none --test-reporter=tap crates/tbg-client/tests/*.test.js` — **24 passed, 0 failed**. The new scenarios check complete/canonical assets and budget, four-frame-only preload, stale-frame clearing and unsafe manifest failure.
- All ten original portrait SHA-256 checks pass; [hash inventory](../media/factions/portraits.sha256).
- Browser: production HUD HTML/CSS/JS projected by a private local fixture, 1920x1080 and 1366x768. Every Faction and sufficient/mixed/insufficient funds inspected. No horizontal overflow; all five cards and their costs remain inside the viewport. Parent/frame computed filter is `none`; only unavailable portraits are grayscale. [State measurements](../media/factions/factions-layout.json). Error-level browser console entries: 0.
- `cargo fmt --all -- --check`: pass. `cargo clippy --workspace -- -D warnings`: pass (1.17 s). `cargo check -p tbg-client -p tbg-core --target wasm32-unknown-unknown`: pass.
- `trunk build --release`: pass. The first invocation completed Rust compilation in 2m48s but was externally terminated (exit 143) before packaging; one cached retry completed the release pipeline. No compiler error.
- Real local release Practice: purchased Shieldbearer, Army increased to 1 and all five offers refreshed. All four Faction frames loaded successfully in the refreshed Shop, with zero error-level browser logs. [Actual Practice, 1920x1080](../media/factions/factions-practice-1920.jpg).

Gallery: [catalog](../media/factions/factions-catalog-1366.jpg), [available 1920](../media/factions/factions-affordable-1920.jpg), [unavailable 1920](../media/factions/factions-unaffordable-1920.jpg), [mixed 1366](../media/factions/factions-mixed-1366.jpg), [available 1366](../media/factions/factions-affordable-1366.jpg), [unavailable 1366](../media/factions/factions-unaffordable-1366.jpg).

## Delivery boundary

Owner explicitly requested branch commits and GitHub push only. No merge or VPS change. Existing unrelated online persistence test remains stopped under the repeated-error rule; see [ranking delivery blocker](../RANKING-UI.md#online-verification-blocker). This task changes no Rust simulation, server or protocol source.
