# TBG emblem and platform icons

The owner supplied the twin fortress / golden spear reference and requested
a professional raster redraw. The final refinement used built-in `imagegen`
(reference-image edit, transparent background). No SVG conversion is used.
The original generated raster is preserved as `tbg-emblem-master.png`.

`scripts/export-icons.py` requires Pillow and exports lossless PNG/WebP and
a multi-resolution Windows ICO. Run it with the project's local image-tools
Python or another Pillow installation. Exporting adds transparent padding
and downsamples with Lanczos; it does not invent extra source detail.

- `desktop/`: transparent PNGs from 16 to 1024 px.
- `../../crates/tbg-client/assets/branding/tbg.ico`: Windows/browser ICO,
  containing 16, 24, 32, 48, 64, 128 and 256 px images.
- Runtime `tbg-icon-512.png`: Linux launchers and AppImage.
- Runtime `tbg-emblem.webp`: lossless game UI emblem.
- `ios/AppIcon.appiconset`: square, opaque 1024 px default icon and Xcode
  catalog metadata. No rounded corners are baked in. Future iOS builds may
  add Icon Composer appearance variants; no iOS device validation is claimed.
- `android/res`: adaptive foreground/background, monochrome and legacy
  launcher resources across five densities. The foreground fits inside a
  central 64 dp box within the 108 dp layer, inside the documented 66 dp
  safe zone. The launcher supplies its mask. `play-store-512.png` is separate.

Mobile resources are preparation assets, not an Android/iOS application or
a claim of store acceptance. Owner visual sign-off remains open.

Platform references checked on 2026-10-31:
[Android adaptive icons](https://developer.android.com/develop/ui/compose/system/icon_design_adaptive),
[Apple app icons](https://developer.apple.com/design/human-interface-guidelines/app-icons/),
[Xcode icon catalogs](https://developer.apple.com/documentation/xcode/configuring-your-app-icon).

## Final generation prompt

```text
Use case: precise-object-edit. Asset type: final desktop/mobile game app icon master.
Edit the referenced refined Tupi Battle Grounds emblem only to make it an immaculate professional raster app icon. Preserve its identity, geometry, scale relationships and premium faceted finish: blue fortress LEFT, red fortress RIGHT, one gold spear centered, full upper/lower spear tips, two arched windows, mirrored shield-like lower shapes.
Critical cleanup: remove every isolated pixel, colored speck, scratch, dark mark, halo and fringe outside the actual emblem. The outside silhouette must be perfectly clean and smoothly antialiased. Preserve the internal jewel-metal gradient shading and crisp facets. Solid clean surfaces, no grain/no distressed texture.
Center the entire complete emblem on a SQUARE canvas with at least 9% transparent padding on every side. Preserve truly transparent alpha outside the emblem, including through genuine gaps; no painted checkerboard or solid backdrop. Emblem should be balanced and legible as a 48px icon. Highest available resolution/quality, no text, no initials, no added objects, no frame or app tile.
```
