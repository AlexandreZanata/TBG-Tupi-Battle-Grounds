# TBG emblem and platform icons

The owner selected the second fortress emblem with banners, a silver/gold
spear and a blue jewel. This supersedes the earlier plain spear emblem.
The refinement preserves that supplied composition and removes the screenshot
background for launcher use. The final refinement used built-in `imagegen`
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

## Earlier generation prompt (superseded)

```text
Use case: precise-object-edit. Asset type: final desktop/mobile game app icon master.
Edit the referenced refined Tupi Battle Grounds emblem only to make it an immaculate professional raster app icon. Preserve its identity, geometry, scale relationships and premium faceted finish: blue fortress LEFT, red fortress RIGHT, one gold spear centered, full upper/lower spear tips, two arched windows, mirrored shield-like lower shapes.
Critical cleanup: remove every isolated pixel, colored speck, scratch, dark mark, halo and fringe outside the actual emblem. The outside silhouette must be perfectly clean and smoothly antialiased. Preserve the internal jewel-metal gradient shading and crisp facets. Solid clean surfaces, no grain/no distressed texture.
Center the entire complete emblem on a SQUARE canvas with at least 9% transparent padding on every side. Preserve truly transparent alpha outside the emblem, including through genuine gaps; no painted checkerboard or solid backdrop. Emblem should be balanced and legible as a 48px icon. Highest available resolution/quality, no text, no initials, no added objects, no frame or app tile.
```
## Replacement generation prompt (current default)

Built-in `imagegen`, reference edit, transparent background. Source: the
owner's `Screenshot_2026-10-31_08-25-11.png`. The generated raster has
1254 × 1254 pixels; the supplied screenshot had 785 × 738 pixels. This is
an AI restoration preserving the design, not a pixel-identical resampling.
Runtime web references use `?v=fortress-2` to refresh cached icons.

```text
Use the attached image as the exact approved game emblem. Perform a faithful high-resolution restoration / super-resolution edit, not a redesign. Reproduce the emblem EXACTLY: identical silhouette, proportions, position and perspective of every element; left sapphire-blue fortress with its blue waving banner, right ruby-red fortress and its red banner, central tall silver spearhead framed in gold with a golden diamond inset, dark metal shaft with golden bands, gold ornamental swept base and the bright cyan-blue diamond jewel centered at the bottom, steel-gray angular leaves/blades behind it, all existing gold trim. Preserve the same polished faceted low-poly fantasy game rendering, the exact relative sizes and color placement, asymmetrical details and geometry. Do not add, remove, simplify, reinterpret, mirror or change any ornament. Increase edge clarity and material detail cleanly without changing shapes or colors. Remove only the nearly black blurred screenshot background and tiny edge debris outside the emblem to create a clean fully transparent alpha background. No black rectangle, no text, no watermarks, no shadows outside the emblem. Center the complete unchanged emblem on a square high-resolution transparent canvas with a small clear margin, including its full top spear tip and bottom point. Highest available resolution and professional icon quality. The reference is the design authority; preserve its exact appearance.
```
